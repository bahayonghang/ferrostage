#![forbid(unsafe_code)]
use gstreamer as gst;
use gst::prelude::*;
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    gst::init()?;
    let uri = gst::glib::filename_to_uri(std::env::args().nth(1).ok_or("input required")?, None)?;
    let sink = gstreamer_app::AppSink::builder().max_buffers(2).sync(true).build();
    let audio = gst::ElementFactory::make("fakesink").property("sync", true).build()?;
    let pipeline = gst::ElementFactory::make("playbin3").property("uri", uri)
        .property("video-sink", &sink).property("audio-sink", &audio).build()?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        for cycle in 0..2 {
            pipeline.set_state(gst::State::Paused)?;
            pipeline.state(gst::ClockTime::from_seconds(5)).0?;
            pipeline.seek(1.0, gst::SeekFlags::FLUSH | gst::SeekFlags::ACCURATE,
                gst::SeekType::Set, gst::ClockTime::from_seconds(2),
                gst::SeekType::Set, gst::ClockTime::from_seconds(3))?;
            pipeline.set_state(gst::State::Playing)?;
            pipeline.state(gst::ClockTime::from_seconds(5)).0?;
            let deadline = Instant::now() + Duration::from_secs(8);
            let mut frames = 0;
            let mut last = None;
            while Instant::now() < deadline {
                if let Some(sample) = sink.try_pull_sample(gst::ClockTime::from_mseconds(100)) {
                    let pts = sample.buffer().and_then(|buffer| buffer.pts()).ok_or("no PTS")?;
                    if !(gst::ClockTime::from_seconds(2)..gst::ClockTime::from_seconds(3)).contains(&pts)
                        || last.is_some_and(|previous| pts <= previous) {
                        return Err(format!("unexpected frame PTS {pts}").into());
                    }
                    last = Some(pts);
                    frames += 1;
                    if frames == 10 {
                        pipeline.set_state(gst::State::Paused)?;
                        let (transition, state, _) = pipeline.state(gst::ClockTime::from_seconds(5));
                        transition?;
                        if state != gst::State::Paused { return Err("did not pause".into()); }
                        let position = pipeline.query_position::<gst::ClockTime>().ok_or("no position")?;
                        std::thread::sleep(Duration::from_millis(150));
                        if pipeline.query_position::<gst::ClockTime>() != Some(position) { return Err("paused position moved".into()); }
                        println!("cycle={cycle} paused_position={position} unchanged=true");
                        pipeline.set_state(gst::State::Playing)?;
                        pipeline.state(gst::ClockTime::from_seconds(5)).0?;
                    }
                } else if sink.is_eos() { break; }
                if let Some(error) = pipeline.bus().ok_or("no bus")?.pop_filtered(&[gst::MessageType::Error]) {
                    return Err(format!("pipeline error: {error:?}").into());
                }
            }
            if !sink.is_eos() || frames != 30 { return Err(format!("incomplete cycle: {frames}").into()); }
            println!("cycle={cycle} resumed=true frames={frames} EOS=true");
        }
        Ok(())
    })();
    pipeline.set_state(gst::State::Null)?;
    let (transition, state, _) = pipeline.state(gst::ClockTime::from_seconds(5));
    transition?;
    if state != gst::State::Null { return Err("did not stop".into()); }
    println!("final_state=Null audio_output=fakesink");
    result
}
