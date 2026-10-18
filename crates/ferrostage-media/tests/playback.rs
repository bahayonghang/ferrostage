use ferrostage_core::{DocumentGeneration, NormalizedRect, PhysicalPage};
use ferrostage_media::{Controller, Identity, MediaError, Runtime, Source, Start, animation};
use std::{
    path::PathBuf,
    thread,
    time::{Duration, Instant},
};
// The product owns one native pipeline. Serialize process fixtures to avoid
// testing several independent owners competing for the shared plugin registry.
#[cfg(feature = "native-tests")]
static NATIVE: std::sync::Mutex<()> = std::sync::Mutex::new(());
fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/media")
}
fn request(name: &str) -> Start {
    Start {
        identity: Identity {
            generation: DocumentGeneration(1),
            page: PhysicalPage(0),
            slot: 0,
            epoch: 0,
        },
        source: Source::External(fixtures().join(name)),
        max_height: 1080,
        source_crop: NormalizedRect::FULL,
        aspect_ratio: None,
        start_ms: 0,
        duration_ms: None,
        plays: Some(0),
    }
}
#[track_caller]
fn wait(
    controller: &Controller,
    predicate: impl Fn(&ferrostage_media::Update) -> bool,
) -> ferrostage_media::Update {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(update) = controller.latest() {
            if predicate(&update) {
                return update;
            }
            assert!(update.error.is_none(), "{update:?}");
        }
        assert!(
            Instant::now() < deadline,
            "media timeout: {:?}",
            controller.latest().map(|u| (
                u.position_ms,
                u.duration_ms,
                u.paused,
                u.ended,
                u.error,
                u.frame.map(|f| f.sequence)
            ))
        );
        thread::sleep(Duration::from_millis(10));
    }
}
#[test]
fn actual_animation_delays_bgra_limits_pause_and_epoch_cancellation() {
    for name in ["synthetic-three-frames.gif", "synthetic-three-frames.webp"] {
        let decoded =
            animation::decode(&fixtures().join(name), 1080, NormalizedRect::FULL, || false)
                .unwrap();
        assert_eq!(
            decoded
                .frames
                .iter()
                .map(|(_, d)| d.as_millis())
                .collect::<Vec<_>>(),
            [120, 240, 360]
        );
        assert_eq!(decoded.frames[0].0.width(), 160);
        assert_eq!(decoded.frames[0].0.pixels_bgra()[3], 255);
        assert_eq!(
            animation::decode(&fixtures().join(name), 1080, NormalizedRect::FULL, || true).err(),
            Some(MediaError::Cancelled)
        );
    }
    let mut controller = Controller::new(Runtime::default()).unwrap();
    let first = controller
        .start(request("synthetic-three-frames.gif"))
        .unwrap();
    let update = wait(&controller, |u| u.frame.is_some());
    let pixels = update.frame.unwrap().pixels;
    controller.pause(true);
    let paused = wait(&controller, |u| u.paused);
    thread::sleep(Duration::from_millis(80));
    assert_eq!(controller.latest().unwrap().position_ms, paused.position_ms);
    controller.stop();
    assert!(controller.latest().is_none());
    let second = controller
        .start(request("synthetic-three-frames.gif"))
        .unwrap();
    assert!(second.epoch > first.epoch);
    let next = wait(&controller, |u| u.frame.is_some());
    assert_eq!(next.identity, second);
    assert_eq!(pixels.width(), next.frame.unwrap().pixels.width());
    controller.shutdown();
    let deadline = Instant::now() + Duration::from_secs(2);
    while !controller.has_exited() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(5));
    }
}
#[test]
fn actual_missing_runtime_is_typed_and_does_not_poison_animation() {
    let mut controller = Controller::new(Runtime {
        helper: fixtures().join("missing-helper.exe"),
        directory: fixtures().join("absent-runtime"),
        silent: true,
    })
    .unwrap();
    controller
        .start(request("synthetic-ball-tone-12s.mp4"))
        .unwrap();
    let error = wait(&controller, |u| u.error.is_some());
    assert_eq!(error.error, Some(MediaError::RuntimeUnavailable));
    controller
        .start(request("synthetic-three-frames.gif"))
        .unwrap();
    assert!(wait(&controller, |u| u.frame.is_some()).error.is_none());
}
#[cfg(feature = "native-tests")]
#[test]
fn actual_helper_video_seek_repeat_pause_audio_and_reap() {
    let _native = NATIVE.lock().unwrap();
    let runtime = Runtime {
        helper: PathBuf::from(
            std::env::var_os("FERROSTAGE_TEST_MEDIA_HELPER")
                .expect("verified built helper required"),
        ),
        directory: PathBuf::from(
            std::env::var_os("FERROSTAGE_TEST_MEDIA_RUNTIME").expect("verified runtime required"),
        ),
        silent: true,
    };
    for name in [
        "synthetic-ball-tone-12s.mp4",
        "synthetic-ball-tone-12s.webm",
        "synthetic-ball-tone-12s.ogg",
    ] {
        let mut controller = Controller::new(runtime.clone()).unwrap();
        let mut start = request(name);
        start.start_ms = 2000;
        start.duration_ms = Some(800);
        start.plays = Some(2);
        controller.start(start).unwrap();
        let first = wait(&controller, |u| u.duration_ms == 800 && u.position_ms > 50);
        assert!(first.position_ms < 800);
        if !name.ends_with("ogg") {
            assert!(
                wait(&controller, |u| u.frame.is_some())
                    .frame
                    .unwrap()
                    .pixels
                    .width()
                    <= 1920
            );
        }
        controller.pause(true);
        let paused = wait(&controller, |u| u.paused);
        thread::sleep(Duration::from_millis(120));
        assert!(
            controller
                .latest()
                .unwrap()
                .position_ms
                .abs_diff(paused.position_ms)
                < 80
        );
        controller.pause(false);
        let ended = wait(&controller, |u| u.ended);
        assert!(ended.error.is_none());
        if name.ends_with("ogg") {
            assert!(ended.frame.is_none());
        }
        controller.shutdown();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !controller.has_exited() {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(5));
        }
    }
}

#[cfg(feature = "native-tests")]
fn software_only_subprocess(test: &str, disable_software: bool) {
    let _native = NATIVE.lock().unwrap();
    // The selected runtime's hardware H.264 factories are D3D11 adapters.
    // Disable every adapter slot, including machines with multiple GPUs.
    let mut ranks = vec!["d3d11h264dec:0".to_owned()];
    ranks.extend((0..64).map(|n| format!("d3d11h264device{n}dec:0")));
    if disable_software {
        ranks.push("openh264dec:0".to_owned());
    }
    let registry = std::env::temp_dir().join(format!(
        "ferrostage-software-registry-{}-{test}.bin",
        std::process::id()
    ));
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test, "--nocapture"])
        .env("FERROSTAGE_TEST_SOFTWARE_ONLY_CHILD", "1")
        .env("GST_REGISTRY_1_0", &registry)
        .env("GST_PLUGIN_FEATURE_RANK", ranks.join(","))
        .output()
        .unwrap();
    if registry.exists() {
        std::fs::remove_file(registry).unwrap();
    }
    assert!(
        output.status.success(),
        "software-only subprocess failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(feature = "native-tests")]
#[test]
fn software_h264_video_seek_repeat_pause_audio_and_reap_without_hardware() {
    if std::env::var_os("FERROSTAGE_TEST_SOFTWARE_ONLY_CHILD").is_some() {
        actual_helper_video_seek_repeat_pause_audio_and_reap();
    } else {
        software_only_subprocess(
            "software_h264_video_seek_repeat_pause_audio_and_reap_without_hardware",
            false,
        );
    }
}

#[cfg(feature = "native-tests")]
#[test]
fn missing_h264_decoder_reports_unsupported_instead_of_audio_only_success() {
    if std::env::var_os("FERROSTAGE_TEST_SOFTWARE_ONLY_CHILD").is_none() {
        software_only_subprocess(
            "missing_h264_decoder_reports_unsupported_instead_of_audio_only_success",
            true,
        );
        return;
    }
    let mut controller = Controller::new(Runtime {
        helper: std::env::var_os("FERROSTAGE_TEST_MEDIA_HELPER")
            .unwrap()
            .into(),
        directory: std::env::var_os("FERROSTAGE_TEST_MEDIA_RUNTIME")
            .unwrap()
            .into(),
        silent: true,
    })
    .unwrap();
    controller
        .start(request("synthetic-ball-tone-12s.mp4"))
        .unwrap();
    let update = wait(&controller, |u| u.error.is_some());
    assert_eq!(update.error, Some(MediaError::Unsupported));
    assert!(update.frame.is_none());
    controller.shutdown();
}

#[test]
fn unnamed_embedded_gif_webp_use_pure_decoder_and_output_bounds() {
    for name in ["synthetic-three-frames.gif", "synthetic-three-frames.webp"] {
        let mut controller = Controller::new(Runtime {
            helper: fixtures().join("missing"),
            directory: fixtures().join("missing"),
            silent: true,
        })
        .unwrap();
        let mut start = request(name);
        start.source = Source::Embedded {
            name: "media.bin".into(),
            bytes: std::fs::read(fixtures().join(name)).unwrap().into(),
        };
        start.max_height = 45;
        start.plays = None;
        controller.start(start).unwrap();
        let update = wait(&controller, |u| u.frame.is_some());
        let pixels = update.frame.unwrap().pixels;
        assert_eq!((pixels.width(), pixels.height()), (80, 45));
        thread::sleep(Duration::from_millis(800));
        assert!(
            !controller.latest().unwrap().ended,
            "intrinsic infinite animation loop is retained"
        );
        controller.shutdown();
    }
}

#[test]
fn contain_letterbox_then_crop_is_bounded_and_never_stretches_private_half() {
    let pixels = ferrostage_core::BgraFrame::new(
        4,
        2,
        16,
        vec![
            1, 2, 9, 255, 1, 2, 9, 255, 8, 9, 1, 255, 8, 9, 1, 255, 1, 2, 9, 255, 1, 2, 9, 255, 8,
            9, 1, 255, 8, 9, 1, 255,
        ],
    )
    .unwrap();
    let public =
        animation::project_frame(&pixels, NormalizedRect::LEFT_HALF, Some(1.), 1080).unwrap();
    assert_eq!((public.width(), public.height()), (2, 4));
    assert_eq!(&public.pixels_bgra()[..8], &[0; 8]);
    assert!(
        public
            .pixels_bgra()
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[3] > 0)
            .all(|p| p[2] > p[1])
    );
    assert!(animation::project_frame(&pixels, NormalizedRect::FULL, Some(1e-20), 1080).is_err());
}

#[cfg(feature = "native-tests")]
#[test]
fn partial_runtime_real_helper_and_embedded_video_are_independent_failures() {
    let _native = NATIVE.lock().unwrap();
    let helper = PathBuf::from(
        std::env::var_os("FERROSTAGE_TEST_MEDIA_HELPER").expect("verified built helper required"),
    );
    let directory = PathBuf::from(
        std::env::var_os("FERROSTAGE_TEST_MEDIA_RUNTIME").expect("verified runtime required"),
    );
    let partial =
        std::env::temp_dir().join(format!("ferrostage-media-partial-{}", std::process::id()));
    std::fs::create_dir_all(partial.join("bin")).unwrap();
    std::fs::copy(
        directory.join("bin/gstreamer-1.0-0.dll"),
        partial.join("bin/gstreamer-1.0-0.dll"),
    )
    .unwrap();
    let mut missing = Controller::new(Runtime {
        helper: helper.clone(),
        directory: partial.clone(),
        silent: true,
    })
    .unwrap();
    missing
        .start(request("synthetic-ball-tone-12s.mp4"))
        .unwrap();
    assert_eq!(
        wait(&missing, |u| u.error.is_some()).error,
        Some(MediaError::RuntimeUnavailable)
    );
    missing.shutdown();
    std::fs::remove_dir_all(partial).unwrap();
    let mut controller = Controller::new(Runtime {
        helper,
        directory,
        silent: true,
    })
    .unwrap();
    let mut start = request("synthetic-ball-tone-12s.mp4");
    start.source = Source::Embedded {
        name: "video.mp4".into(),
        bytes: std::fs::read(fixtures().join("synthetic-ball-tone-12s.mp4"))
            .unwrap()
            .into(),
    };
    start.duration_ms = Some(250);
    start.plays = Some(1);
    controller.start(start).unwrap();
    assert!(wait(&controller, |u| u.frame.is_some()).frame.is_some());
    assert!(wait(&controller, |u| u.ended).error.is_none());
    controller.shutdown();
}

#[cfg(feature = "native-tests")]
#[test]
fn unresponsive_child_stderr_is_drained_then_killed_and_reaped_off_caller() {
    let _native = NATIVE.lock().unwrap();
    let helper = PathBuf::from(
        std::env::var_os("FERROSTAGE_TEST_MEDIA_HELPER").expect("verified built helper required"),
    );
    let helper = helper.parent().unwrap().join("examples/ipc_stall.exe");
    assert!(
        helper.is_file(),
        "build ipc_stall with native-tests before this suite"
    );
    let directory = PathBuf::from(
        std::env::var_os("FERROSTAGE_TEST_MEDIA_RUNTIME").expect("verified runtime required"),
    );
    let mut controller = Controller::new(Runtime {
        helper,
        directory,
        silent: true,
    })
    .unwrap();
    controller
        .start(request("synthetic-ball-tone-12s.mp4"))
        .unwrap();
    thread::sleep(Duration::from_millis(150));
    let before = Instant::now();
    controller.shutdown();
    assert!(
        before.elapsed() < Duration::from_millis(100),
        "shutdown must not join on caller"
    );
    assert!(controller.latest().is_none());
    let deadline = Instant::now() + Duration::from_secs(3);
    while !controller.has_exited() {
        assert!(
            Instant::now() < deadline,
            "child must be killed/reaped despite stalled protocol"
        );
        thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(feature = "native-tests")]
#[test]
fn helper_parent_pipe_eof_terminates_process_during_startup() {
    let _native = NATIVE.lock().unwrap();
    use ferrostage_media::protocol::{self, Open, Packet};
    use std::process::{Command, Stdio};
    let helper = PathBuf::from(
        std::env::var_os("FERROSTAGE_TEST_MEDIA_HELPER").expect("verified helper required"),
    );
    let runtime = PathBuf::from(
        std::env::var_os("FERROSTAGE_TEST_MEDIA_RUNTIME").expect("verified runtime required"),
    );
    let paths = std::iter::once(runtime.join("bin"))
        .chain(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ))
        .collect::<Vec<_>>();
    let mut command = Command::new(helper);
    command
        .env("PATH", std::env::join_paths(paths).unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut child = command.spawn().unwrap();
    let mut stdin = child.stdin.take().unwrap();
    protocol::write(
        &mut stdin,
        &Packet::Open(Open {
            path: fixtures()
                .join("synthetic-ball-tone-12s.mp4")
                .to_string_lossy()
                .into(),
            max_height: 1080,
            start_ms: 0,
            duration_ms: None,
            plays: 0,
            silent: true,
        }),
    )
    .unwrap();
    drop(stdin);
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(2));
            break;
        }
        if Instant::now() > deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("helper survived parent pipe EOF");
        }
        thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(feature = "native-tests")]
#[test]
fn original_beamer_movie_metadata_drives_real_helper_playback() {
    let _native = NATIVE.lock().unwrap();
    use ferrostage_pdf::structure::{MediaRepeat, MediaSource, MediaTime};
    let bytes = std::fs::read(fixtures().join("movie.pdf")).unwrap();
    let parsed = ferrostage_pdf::extract_structure(&bytes).unwrap();
    let media = parsed.pages.iter().flat_map(|p| &p.media).next().unwrap();
    let Some(MediaSource::External(path)) = &media.source else {
        panic!("external movie required")
    };
    let (Some(MediaTime::Seconds(start)), Some(MediaTime::Seconds(duration))) =
        (media.start, media.duration)
    else {
        panic!("scaled timing required")
    };
    assert_eq!(
        (start, duration, media.repeat, media.autostart),
        (2., 8., Some(MediaRepeat::Forever), Some(true))
    );
    let mut request = request(path);
    request.start_ms = (start * 1000.) as u64;
    request.duration_ms = Some((duration * 1000.) as u64);
    request.plays = Some(0);
    let mut controller = Controller::new(Runtime {
        helper: std::env::var_os("FERROSTAGE_TEST_MEDIA_HELPER")
            .expect("built helper required")
            .into(),
        directory: std::env::var_os("FERROSTAGE_TEST_MEDIA_RUNTIME")
            .expect("verified runtime required")
            .into(),
        silent: true,
    })
    .unwrap();
    controller.start(request).unwrap();
    let frame = wait(&controller, |u| {
        u.duration_ms == 8000 && u.position_ms > 50 && u.frame.is_some()
    });
    assert!(frame.frame.unwrap().pixels.width() > 0);
    controller.shutdown();
    let deadline = Instant::now() + Duration::from_secs(3);
    while !controller.has_exited() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(std::fs::read(fixtures().join("movie.pdf")).unwrap(), bytes);
}
