#![forbid(unsafe_code)]
use image::{AnimationDecoder, ImageDecoder};
use std::{fs::File, io::BufReader, path::PathBuf};

fn check<'a>(decoder: impl AnimationDecoder<'a> + ImageDecoder, max_rgb_error: u8) -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = decoder.dimensions();
    if (width, height) != (160, 90) { return Err("fixture geometry differs".into()); }
    let mut delays = Vec::new();
    let mut colors = Vec::new();
    let mut total = 0;
    for frame in decoder.into_frames().take(4) {
        let frame = frame?;
        delays.push(frame.delay().numer_denom_ms());
        let mut packed = frame.into_buffer().into_raw();
        for pixel in packed.as_chunks_mut::<4>().0 { pixel.swap(0, 2); }
        colors.push(packed[..4].to_vec());
        total += packed.len();
    }
    if delays != [(120, 1), (240, 1), (360, 1)] { return Err(format!("bad delays {delays:?}").into()); }
    let expected = [[40u8,65,210,255], [90,160,40,255], [210,95,60,255]];
    for (color, expected) in colors.iter().zip(expected) {
        if color[3] != 255 || color[..3].iter().zip(expected).any(|(a,b)| a.abs_diff(b) > max_rgb_error) {
            return Err(format!("bad BGRA {colors:?}").into());
        }
    }
    println!("frames={} dimensions={width}x{height} delays={delays:?} corner_bgra={colors:?} packed_bytes={total}", delays.len());
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(std::env::args_os().nth(1).ok_or("missing fixture directory")?);
    check(image::codecs::gif::GifDecoder::new(BufReader::new(File::open(root.join("synthetic-three-frames.gif"))?))?, 0)?;
    // image-webp's documented approximate animation compositing differs from
    // libwebp/Pillow by one RGB level here; alpha and frame delay remain exact.
    check(image::codecs::webp::WebPDecoder::new(BufReader::new(File::open(root.join("synthetic-three-frames.webp"))?))?, 1)?;
    Ok(())
}
