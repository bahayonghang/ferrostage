//! One pipeline and one audio owner, running only in the isolated helper.
use ferrostage_core::BgraFrame;
use ferrostage_media::{
    MAX_SOURCE_PIXELS, MediaError, output_size,
    protocol::{Open, Packet},
};
use gstreamer as gst;
use gstreamer_video::prelude::*;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
    },
    time::{Duration, Instant},
};
fn failure(error: impl std::fmt::Display) -> MediaError {
    eprintln!("media pipeline: {error}");
    MediaError::Unsupported
}
fn check_decoder(message: &gst::MessageRef) -> Result<(), MediaError> {
    if matches!(message.view(), gst::MessageView::Error(_))
        || (matches!(message.view(), gst::MessageView::Element(_))
            && message.structure().is_some_and(|structure| {
                structure.name() == "missing-plugin"
                    && structure.get::<&str>("type").ok() == Some("decoder")
            }))
    {
        // playbin can otherwise continue with audio when video decoding is
        // unavailable, reporting progress/EOS without ever publishing a frame.
        return Err(MediaError::Unsupported);
    }
    Ok(())
}
pub fn play(
    open: Open,
    commands: Receiver<Packet>,
    mut emit: impl FnMut(Packet) -> Result<(), MediaError>,
) -> Result<(), MediaError> {
    gst::init().map_err(|_| MediaError::RuntimeUnavailable)?;
    let uri = gst::glib::filename_to_uri(&open.path, None).map_err(failure)?;
    let sink = gstreamer_app::AppSink::builder()
        .caps(
            &gst::Caps::builder("video/x-raw")
                .field("format", "BGRA")
                .field("width", gst::IntRange::<i32>::new(1, 1920))
                .field(
                    "height",
                    gst::IntRange::<i32>::new(1, open.max_height as i32),
                )
                .field("pixel-aspect-ratio", gst::Fraction::new(1, 1))
                .build(),
        )
        .max_buffers(2)
        .drop(true)
        .sync(true)
        .build();
    let audio = gst::ElementFactory::make(if open.silent {
        "fakesink"
    } else {
        "autoaudiosink"
    })
    .build()
    .map_err(failure)?;
    if open.silent {
        audio.set_property("sync", true);
    }
    let scale = gst::ElementFactory::make("videoconvertscale")
        .build()
        .map_err(failure)?;
    scale.set_property_from_str("method", "nearest-neighbour");
    let pipeline = gst::ElementFactory::make("playbin3")
        .property("uri", uri)
        .property("video-sink", &sink)
        .property("video-filter", &scale)
        .property("audio-sink", &audio)
        .build()
        .map_err(failure)?;
    let invalid = Arc::new(AtomicBool::new(false));
    let invalid_caps = invalid.clone();
    // Encoded caps reach the decoder before its first buffer. Refuse enormous
    // declared geometry before native decoding, as well as checking raw output.
    pipeline.connect("element-setup", false, move |values| {
        if let Ok(element) = values[1].get::<gst::Element>()
            && element
                .factory()
                .and_then(|f| f.metadata("klass").map(str::to_owned))
                .is_some_and(|s| s.contains("Decoder/Video"))
            && let Some(pad) = element.static_pad("sink")
        {
            let invalid = invalid_caps.clone();
            let validated = Arc::new(AtomicBool::new(false));
            pad.add_probe(
                gst::PadProbeType::EVENT_DOWNSTREAM | gst::PadProbeType::BUFFER,
                move |_, info| {
                    if let Some(gst::PadProbeData::Event(event)) = &info.data
                        && let gst::EventView::Caps(caps) = event.view()
                    {
                        let dimensions = caps.caps().structure(0).and_then(|s| {
                            Some((s.get::<i32>("width").ok()?, s.get::<i32>("height").ok()?))
                        });
                        let accepted = dimensions.is_some_and(|(w, h)| {
                            w > 0
                                && h > 0
                                && w <= 8192
                                && h <= 8192
                                && w as u64 * h as u64 <= MAX_SOURCE_PIXELS
                        });
                        validated.store(accepted, Ordering::Release);
                        if !accepted {
                            invalid.store(true, Ordering::Release);
                            return gst::PadProbeReturn::Drop;
                        }
                    }
                    if matches!(&info.data, Some(gst::PadProbeData::Buffer(_)))
                        && !validated.load(Ordering::Acquire)
                    {
                        invalid.store(true, Ordering::Release);
                        return gst::PadProbeReturn::Drop;
                    }
                    gst::PadProbeReturn::Ok
                },
            );
        }
        None
    });
    let result = (|| {
        let bus = pipeline.bus().ok_or(MediaError::Process)?;
        pipeline.set_state(gst::State::Paused).map_err(failure)?;
        let (result, current, _) = pipeline.state(gst::ClockTime::from_seconds(5));
        result.map_err(failure)?;
        if current != gst::State::Paused {
            return Err(MediaError::Timeout);
        }
        if invalid.load(Ordering::Acquire) {
            return Err(MediaError::Limit);
        }
        for message in bus.iter_filtered(&[gst::MessageType::Error, gst::MessageType::Element]) {
            check_decoder(&message)?;
        }
        let duration = pipeline
            .query_duration::<gst::ClockTime>()
            .map(|t| t.mseconds())
            .unwrap_or(0);
        let remaining = duration.saturating_sub(open.start_ms);
        let length = open.duration_ms.unwrap_or(remaining).min(if duration == 0 {
            u64::MAX
        } else {
            remaining
        });
        if length == 0 || length == u64::MAX {
            return Err(MediaError::Unsupported);
        }
        let stop = open.start_ms.checked_add(length).ok_or(MediaError::Limit)?;
        let seek = || {
            pipeline
                .seek(
                    1.,
                    gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE,
                    gst::SeekType::Set,
                    gst::ClockTime::from_mseconds(open.start_ms),
                    gst::SeekType::Set,
                    gst::ClockTime::from_mseconds(stop),
                )
                .map_err(failure)
        };
        seek()?;
        pipeline.set_state(gst::State::Playing).map_err(failure)?;
        let mut sequence = 0;
        let mut cycle = 0u32;
        let mut paused = false;
        let mut status_at = Instant::now() - Duration::from_secs(1);
        loop {
            loop {
                match commands.try_recv() {
                    Ok(Packet::Stop) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        return Ok(());
                    }
                    Ok(Packet::Pause(value)) => {
                        paused = value;
                        pipeline
                            .set_state(if paused {
                                gst::State::Paused
                            } else {
                                gst::State::Playing
                            })
                            .map_err(failure)?;
                    }
                    Ok(_) => return Err(MediaError::Protocol),
                    Err(std::sync::mpsc::TryRecvError::Empty) => break,
                }
            }
            if invalid.load(Ordering::Acquire) {
                return Err(MediaError::Limit);
            }
            if let Some(sample) = sink.try_pull_sample(gst::ClockTime::from_mseconds(10)) {
                let info = gstreamer_video::VideoInfo::from_caps(
                    sample.caps().ok_or(MediaError::Protocol)?,
                )
                .map_err(failure)?;
                let (w, h) = (info.width(), info.height());
                if output_size(w, h, open.max_height)? != (w, h) {
                    return Err(MediaError::Limit);
                }
                let buffer = sample.buffer().ok_or(MediaError::Protocol)?;
                let mapped =
                    gstreamer_video::VideoFrameRef::from_buffer_ref_readable(buffer, &info)
                        .map_err(failure)?;
                let stride = usize::try_from(mapped.plane_stride()[0]).map_err(failure)?;
                let row = w as usize * 4;
                if stride < row {
                    return Err(MediaError::Protocol);
                }
                let plane = mapped.plane_data(0).map_err(failure)?;
                let mut pixels = Vec::with_capacity(row * h as usize);
                for y in 0..h as usize {
                    let start = y.checked_mul(stride).ok_or(MediaError::Limit)?;
                    pixels.extend_from_slice(
                        plane.get(start..start + row).ok_or(MediaError::Protocol)?,
                    );
                }
                sequence += 1;
                emit(Packet::Frame {
                    sequence,
                    position_ms: buffer
                        .pts()
                        .map(|t| t.mseconds().saturating_sub(open.start_ms))
                        .unwrap_or(0),
                    duration_ms: length,
                    pixels: BgraFrame::new(w, h, row, pixels).map_err(failure)?,
                })?;
            }
            while let Some(message) = bus.pop_filtered(&[
                gst::MessageType::Error,
                gst::MessageType::Eos,
                gst::MessageType::Element,
            ]) {
                check_decoder(&message)?;
                if let gst::MessageView::Eos(_) = message.view() {
                    cycle = cycle.saturating_add(1);
                    if open.plays != 0 && cycle >= open.plays {
                        emit(Packet::End)?;
                        return Ok(());
                    }
                    seek()?;
                    if !paused {
                        pipeline.set_state(gst::State::Playing).map_err(failure)?;
                    }
                }
            }
            if status_at.elapsed() >= Duration::from_millis(50) {
                let position = pipeline
                    .query_position::<gst::ClockTime>()
                    .map(|t| t.mseconds().saturating_sub(open.start_ms))
                    .unwrap_or(0)
                    .min(length);
                emit(Packet::Status {
                    position_ms: position,
                    duration_ms: length,
                    paused,
                })?;
                status_at = Instant::now();
            }
        }
    })();
    pipeline.set_state(gst::State::Null).map_err(failure)?;
    result
}
