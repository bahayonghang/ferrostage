//! Read-only native inventory. Does not open windows or change display state.
use std::{
    process::ExitCode,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use ferrostage_platform::{enumerate_monitors, resolve_roles, watch_topology};

fn main() -> ExitCode {
    println!(
        "platform={} arch={}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    println!(
        "DPI context is not changed or measured by this probe; scale source is process-dependent."
    );
    let mut first = None;
    for sample in 0..10 {
        let start = Instant::now();
        let monitors = match enumerate_monitors() {
            Ok(monitors) => monitors,
            Err(error) => {
                eprintln!("sample {sample}: {error}");
                return ExitCode::FAILURE;
            }
        };
        println!(
            "sample={sample} count={} enumeration_ms={:.3}",
            monitors.len(),
            start.elapsed().as_secs_f64() * 1000.0
        );
        if let Some(first) = &first {
            if first != &monitors {
                eprintln!("topology changed during static probe");
                return ExitCode::FAILURE;
            }
        } else {
            println!("{monitors:#?}");
            println!("no_saved_roles={:#?}", resolve_roles(&monitors, &[]));
            first = Some(monitors);
        }
        if sample < 9 {
            thread::sleep(Duration::from_millis(200));
        }
    }
    let (sink, events) = mpsc::channel();
    let watcher = match watch_topology(sink) {
        Ok(watcher) => watcher,
        Err(error) => {
            eprintln!("watch start: {error}");
            return ExitCode::FAILURE;
        }
    };
    match events.recv_timeout(Duration::from_secs(3)) {
        Ok(ferrostage_platform::TopologyEvent::Changed(monitors))
            if Some(&monitors) == first.as_ref() =>
        {
            println!("watcher_initial_snapshot=PASS")
        }
        event => {
            eprintln!("unexpected initial watcher result: {event:?}");
            return ExitCode::FAILURE;
        }
    }
    if std::env::args().any(|arg| arg == "--watch") {
        println!("Observing for 30 seconds; record physical unplug/replug times independently.");
        let start = Instant::now();
        while let Some(remaining) = Duration::from_secs(30).checked_sub(start.elapsed()) {
            match events.recv_timeout(remaining) {
                Ok(event) => println!(
                    "watch_elapsed_ms={:.3} event={event:#?}",
                    start.elapsed().as_secs_f64() * 1000.0
                ),
                Err(mpsc::RecvTimeoutError::Timeout) => break,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    eprintln!("watcher disconnected");
                    return ExitCode::FAILURE;
                }
            }
        }
        println!("Observations require manual comparison; no hotplug timing PASS is inferred.");
    }
    if let Err(error) = watcher.stop() {
        eprintln!("watch stop: {error}");
        return ExitCode::FAILURE;
    }
    println!(
        "static_enumeration_and_cancellation=PASS; physical_replug_rotation_port_change=NOT_RUN"
    );
    ExitCode::SUCCESS
}
