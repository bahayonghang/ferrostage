#![forbid(unsafe_code)]
use gstreamer as gst;
use gst::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    gst::init()?;
    let uri = gst::glib::filename_to_uri(std::env::args().nth(1).ok_or("input required")?, None)?;
    let video = gstreamer_app::AppSink::builder().max_buffers(2).sync(false).build();
    let audio = gst::ElementFactory::make("fakesink").property("sync", false).build()?;
    let pipeline = gst::ElementFactory::make("playbin3").property("uri", uri)
        .property("video-sink", &video).property("audio-sink", &audio).build()?;
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        pipeline.set_state(gst::State::Paused)?;
        pipeline.state(gst::ClockTime::from_seconds(5)).0?;
        let duration = pipeline.query_duration::<gst::ClockTime>().ok_or("no audio duration")?;
        if duration != gst::ClockTime::from_seconds(12) { return Err(format!("duration {duration}").into()); }
        pipeline.set_state(gst::State::Playing)?;
        pipeline.state(gst::ClockTime::from_seconds(5)).0?;
        let message = pipeline.bus().ok_or("no bus")?.timed_pop_filtered(
            gst::ClockTime::from_seconds(10), &[gst::MessageType::Error, gst::MessageType::Eos],
        ).ok_or("no audio EOS")?;
        if !matches!(message.view(), gst::MessageView::Eos(_)) { return Err(format!("audio error: {message:?}").into()); }
        if video.try_pull_sample(gst::ClockTime::ZERO).is_some() { return Err("unexpected video frame".into()); }
        println!("audio_duration={duration} bus_EOS=true video_frames=0 audio_output=fakesink");
        Ok(())
    })();
    pipeline.set_state(gst::State::Null)?;
    result
}
