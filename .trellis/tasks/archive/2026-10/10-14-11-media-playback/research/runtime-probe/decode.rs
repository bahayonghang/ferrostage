use gstreamer as gst;
use gstreamer_video::prelude::*;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    gst::init()?;
    let path = std::env::args().nth(1).ok_or("missing input")?;
    let uri = gst::glib::filename_to_uri(path, None)?;
    let sink = gstreamer_app::AppSink::builder()
        .caps(&gst::Caps::builder("video/x-raw").field("format", "BGRA").build())
        .max_buffers(2)
        .sync(false)
        .build();
    let audio = gst::ElementFactory::make("fakesink").property("sync", false).build()?;
    let pipeline = gst::ElementFactory::make("playbin3")
        .property("uri", uri)
        .property("video-sink", &sink)
        .property("audio-sink", &audio)
        .build()?;
    pipeline.set_state(gst::State::Playing)?;
    let (transition, state, pending) = pipeline.state(gst::ClockTime::from_seconds(5));
    transition?;
    eprintln!("state={state:?} pending={pending:?}");
    let started = Instant::now();
    let mut frames = 0usize;
    let mut total = 0usize;
    let mut fingerprint = 0u64;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        while started.elapsed() < Duration::from_secs(20) {
            if let Some(sample) = sink.try_pull_sample(gst::ClockTime::from_mseconds(100)) {
                let info = gstreamer_video::VideoInfo::from_caps(sample.caps().ok_or("no caps")?)?;
                let mapped = gstreamer_video::VideoFrameRef::from_buffer_ref_readable(
                    sample.buffer().ok_or("no buffer")?, &info,
                )?;
                let stride = usize::try_from(mapped.plane_stride()[0])?;
                let row = usize::try_from(info.width())?.checked_mul(4).ok_or("row overflow")?;
                let height = usize::try_from(info.height())?;
                if stride < row { return Err("short stride".into()); }
                let pixels = mapped.plane_data(0)?;
                let mut packed = Vec::with_capacity(row * height);
                for y in 0..height {
                    packed.extend_from_slice(pixels.get(y * stride..y * stride + row).ok_or("short plane")?);
                }
                fingerprint = packed.iter().fold(fingerprint, |n, b| n.wrapping_mul(31).wrapping_add(u64::from(*b)));
                frames += 1;
                total += packed.len();
            } else if sink.is_eos() {
                break;
            }
            if let Some(message) = pipeline.bus().ok_or("no bus")?.pop_filtered(&[gst::MessageType::Error]) {
                return Err(format!("pipeline error: {message:?}").into());
            }
        }
        if frames == 0 || !sink.is_eos() { return Err("no frames or timeout".into()); }
        Ok(())
    })();
    pipeline.set_state(gst::State::Null)?;
    result?;
    println!("native={} frames={frames} packed_bytes={total} fingerprint={fingerprint} elapsed_ms={}", gst::version_string(), started.elapsed().as_millis());
    Ok(())
}
