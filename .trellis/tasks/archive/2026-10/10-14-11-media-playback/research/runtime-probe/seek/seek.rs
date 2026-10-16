#![forbid(unsafe_code)]
use gstreamer as gst;
use gst::prelude::*;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    gst::init()?;
    let uri = gst::glib::filename_to_uri(std::env::args().nth(1).ok_or("input required")?, None)?;
    let sink = gstreamer_app::AppSink::builder()
        .caps(&gst::Caps::builder("video/x-raw").field("format", "BGRA").build())
        .max_buffers(2).sync(false).build();
    let audio = gst::ElementFactory::make("fakesink").property("sync", false).build()?;
    let pipeline = gst::ElementFactory::make("playbin3").property("uri", uri)
        .property("video-sink", &sink).property("audio-sink", &audio).build()?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        for cycle in 0..2 {
            pipeline.set_state(gst::State::Paused)?;
            pipeline.state(gst::ClockTime::from_seconds(5)).0?;
            pipeline.seek(1.0, gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE,
                gst::SeekType::Set, gst::ClockTime::from_seconds(2),
                gst::SeekType::Set, gst::ClockTime::from_seconds(10))?;
            pipeline.set_state(gst::State::Playing)?;
            pipeline.state(gst::ClockTime::from_seconds(5)).0?;
            let deadline = Instant::now() + Duration::from_secs(15);
            let mut frames = 0;
            let mut first = None;
            let mut last = None;
            while Instant::now() < deadline {
                if let Some(sample) = sink.try_pull_sample(gst::ClockTime::from_mseconds(100)) {
                    let pts = sample.buffer().and_then(|buffer| buffer.pts()).ok_or("no PTS")?;
                    if !(gst::ClockTime::from_seconds(2)..gst::ClockTime::from_seconds(10)).contains(&pts) {
                        return Err(format!("out-of-range frame {pts}").into());
                    }
                    first.get_or_insert(pts);
                    last = Some(pts);
                    frames += 1;
                } else if sink.is_eos() { break; }
                if let Some(error) = pipeline.bus().ok_or("no bus")?.pop_filtered(&[gst::MessageType::Error]) {
                    return Err(format!("pipeline error: {error:?}").into());
                }
            }
            if !sink.is_eos() || frames != 240 { return Err(format!("incomplete cycle: {frames}").into()); }
            println!("cycle={cycle} frames={frames} first_pts={first:?} last_pts={last:?} EOS=true");
        }
        Ok(())
    })();
    pipeline.set_state(gst::State::Null)?;
    result
}
