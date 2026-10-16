//! Decode and transform off the UI thread, with limits before frame allocation.
use crate::{MAX_ANIMATION_BYTES, MAX_SOURCE_BYTES, MediaError, output_size};
use ferrostage_core::{BgraFrame, NormalizedRect};
use image::{AnimationDecoder, ImageDecoder};
use std::{fs::File, io::BufReader, path::Path, time::Duration};
pub struct Animation {
    pub frames: Vec<(BgraFrame, Duration)>,
    pub duration: Duration,
    pub plays: u32,
}
pub fn decode(
    path: &Path,
    max_height: u32,
    crop: NormalizedRect,
    cancelled: impl Fn() -> bool,
) -> Result<Animation, MediaError> {
    decode_projected(path, max_height, crop, None, cancelled)
}
pub fn decode_projected(
    path: &Path,
    max_height: u32,
    crop: NormalizedRect,
    aspect: Option<f32>,
    cancelled: impl Fn() -> bool,
) -> Result<Animation, MediaError> {
    let file = File::open(path).map_err(|_| MediaError::SourceUnavailable)?;
    if file
        .metadata()
        .map_err(|_| MediaError::SourceUnavailable)?
        .len()
        > MAX_SOURCE_BYTES
    {
        return Err(MediaError::Limit);
    }
    let guessed = image::ImageReader::new(BufReader::new(file))
        .with_guessed_format()
        .map_err(|_| MediaError::Unsupported)?;
    let format = guessed.format();
    let reader = guessed.into_inner();
    match format {
        Some(image::ImageFormat::Gif) => collect(
            image::codecs::gif::GifDecoder::new(reader).map_err(|_| MediaError::Unsupported)?,
            max_height,
            crop,
            aspect,
            MAX_ANIMATION_BYTES,
            cancelled,
        ),
        Some(image::ImageFormat::WebP) => collect(
            image::codecs::webp::WebPDecoder::new(reader).map_err(|_| MediaError::Unsupported)?,
            max_height,
            crop,
            aspect,
            MAX_ANIMATION_BYTES,
            cancelled,
        ),
        _ => Err(MediaError::Unsupported),
    }
}
fn collect<'a>(
    mut decoder: impl AnimationDecoder<'a> + ImageDecoder,
    max_height: u32,
    crop: NormalizedRect,
    aspect: Option<f32>,
    budget: usize,
    cancelled: impl Fn() -> bool,
) -> Result<Animation, MediaError> {
    let (w, h) = decoder.dimensions();
    let (out_w, out_h) = output_size(w, h, max_height)?;
    let (canvas_w, canvas_h) = if let Some(aspect) = aspect {
        canvas_size(out_w, out_h, aspect, max_height)?
    } else {
        (out_w, out_h)
    };
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(budget as u64);
    decoder.set_limits(limits).map_err(|_| MediaError::Limit)?;
    let mut frames: Vec<(BgraFrame, Duration)> = Vec::new();
    let mut allocated = 0;
    let mut duration = Duration::ZERO;
    let plays = match decoder.loop_count() {
        image::metadata::LoopCount::Infinite => 0,
        image::metadata::LoopCount::Finite(n) => n.get(),
    };
    let mut iter = decoder.into_frames();
    loop {
        if cancelled() {
            return Err(MediaError::Cancelled);
        }
        // Reserve room for the decoder's next full source RGBA frame first.
        if frames.len() >= 10_000
            || allocated
                + frames.capacity() * std::mem::size_of::<(BgraFrame, Duration)>()
                + w as usize * h as usize * 12
                + out_w as usize * out_h as usize * 8
                + canvas_w as usize * canvas_h as usize * 8
                > budget
        {
            return Err(MediaError::Limit);
        }
        let Some(frame) = iter.next() else { break };
        let frame = frame.map_err(|_| MediaError::Unsupported)?;
        let (num, den) = frame.delay().numer_denom_ms();
        let delay = Duration::from_secs_f64((f64::from(num) / f64::from(den) / 1000.).max(0.001));
        let buffer = frame.into_buffer();
        let mut pixels = if (out_w, out_h) == (w, h) {
            buffer.into_raw()
        } else {
            image::imageops::resize(&buffer, out_w, out_h, image::imageops::FilterType::Nearest)
                .into_raw()
        };
        for p in pixels.as_chunks_mut::<4>().0 {
            p.swap(0, 2);
        }
        let frame = BgraFrame::new(out_w, out_h, out_w as usize * 4, pixels)
            .map_err(|_| MediaError::Limit)?;
        let frame = project_frame(&frame, crop, aspect, max_height)?;
        allocated += frame.allocation_bytes();
        if allocated > budget {
            return Err(MediaError::Limit);
        }
        duration = duration.checked_add(delay).ok_or(MediaError::Limit)?;
        frames.push((frame, delay));
    }
    if frames.is_empty() {
        return Err(MediaError::Unsupported);
    }
    Ok(Animation {
        frames,
        duration,
        plays,
    })
}
/// Crop decoded source pixels, never squash a complete private/public frame.
pub fn crop_frame(frame: &BgraFrame, crop: NormalizedRect) -> Result<BgraFrame, MediaError> {
    if !crop.is_inside_unit_square() {
        return Err(MediaError::Limit);
    }
    if crop == NormalizedRect::FULL {
        return Ok(frame.clone());
    }
    let x = (crop.x * frame.width() as f32).ceil() as u32;
    let y = (crop.y * frame.height() as f32).ceil() as u32;
    let right = ((crop.x + crop.w) * frame.width() as f32).floor() as u32;
    let bottom = ((crop.y + crop.h) * frame.height() as f32).floor() as u32;
    if right <= x || bottom <= y {
        return Err(MediaError::Limit);
    }
    let (w, h) = (right - x, bottom - y);
    let mut pixels = Vec::with_capacity(w as usize * h as usize * 4);
    for row in y..bottom {
        let start = row as usize * frame.stride() + x as usize * 4;
        pixels.extend_from_slice(&frame.pixels_bgra()[start..start + w as usize * 4]);
    }
    BgraFrame::new(w, h, w as usize * 4, pixels).map_err(|_| MediaError::Limit)
}
/// Contain in the original physical annotation first, then remove private area.
/// Padding is transparent BGRA, so the underlying PDF remains visible.
pub fn project_frame(
    frame: &BgraFrame,
    crop: NormalizedRect,
    aspect: Option<f32>,
    max_height: u32,
) -> Result<BgraFrame, MediaError> {
    let Some(aspect) = aspect else {
        return crop_frame(frame, crop);
    };
    let (w, h) = canvas_size(frame.width(), frame.height(), aspect, max_height)?;
    let scale = (w as f64 / frame.width() as f64).min(h as f64 / frame.height() as f64);
    let fw = (frame.width() as f64 * scale).floor().max(1.) as u32;
    let fh = (frame.height() as f64 * scale).floor().max(1.) as u32;
    let resized = if (fw, fh) == (frame.width(), frame.height()) {
        None
    } else {
        let buffer =
            image::RgbaImage::from_raw(frame.width(), frame.height(), frame.pixels_bgra().to_vec())
                .ok_or(MediaError::Limit)?;
        Some(
            image::imageops::resize(&buffer, fw, fh, image::imageops::FilterType::Nearest)
                .into_raw(),
        )
    };
    let source = resized.as_deref().unwrap_or(frame.pixels_bgra());
    let mut pixels = vec![0; w as usize * h as usize * 4];
    let (x, y) = ((w - fw) / 2, (h - fh) / 2);
    for row in 0..fh as usize {
        let start = ((y as usize + row) * w as usize + x as usize) * 4;
        pixels[start..start + fw as usize * 4]
            .copy_from_slice(&source[row * fw as usize * 4..(row + 1) * fw as usize * 4]);
    }
    crop_frame(
        &BgraFrame::new(w, h, w as usize * 4, pixels).map_err(|_| MediaError::Limit)?,
        crop,
    )
}
fn canvas_size(
    width: u32,
    height: u32,
    aspect: f32,
    max_height: u32,
) -> Result<(u32, u32), MediaError> {
    if !aspect.is_finite() || aspect <= 0. {
        return Err(MediaError::Limit);
    }
    let source_aspect = width as f64 / height as f64;
    let (w, h) = if source_aspect > f64::from(aspect) {
        (width, (width as f64 / f64::from(aspect)).round() as u32)
    } else {
        ((height as f64 * f64::from(aspect)).round() as u32, height)
    };
    output_size(w, h, max_height)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn animation_budget_reserves_contain_canvas_before_decoding() {
        let mut gif = Vec::new();
        image::codecs::gif::GifEncoder::new(&mut gif)
            .encode_frame(image::Frame::new(image::RgbaImage::from_pixel(
                1,
                16,
                image::Rgba([1, 2, 3, 255]),
            )))
            .unwrap();
        let decode = |budget| {
            let decoder = image::codecs::gif::GifDecoder::new(std::io::Cursor::new(&gif)).unwrap();
            collect(
                decoder,
                1080,
                NormalizedRect::FULL,
                Some(1.),
                budget,
                || false,
            )
        };
        // The final 16x16 BGRA frame fits, but canvas/crop plus decoder scratch
        // do not. Enforce the peak before allocating the projected canvas.
        assert_eq!(decode(2048).err(), Some(MediaError::Limit));
        assert_eq!(decode(4096).unwrap().frames[0].0.allocation_bytes(), 1024);

        let mut gif = Vec::new();
        image::codecs::gif::GifEncoder::new(&mut gif)
            .encode_frames((0..2).map(|_| {
                image::Frame::new(image::RgbaImage::from_pixel(
                    1,
                    16,
                    image::Rgba([1, 2, 3, 255]),
                ))
            }))
            .unwrap();
        let iterations = std::cell::Cell::new(0);
        let decode = |budget| {
            let decoder = image::codecs::gif::GifDecoder::new(std::io::Cursor::new(&gif)).unwrap();
            collect(
                decoder,
                1080,
                NormalizedRect::LEFT_HALF,
                Some(1.),
                budget,
                || {
                    iterations.set(iterations.get() + 1);
                    false
                },
            )
        };
        // The first cropped frame fits. Its retained pixels plus the next
        // full contain canvas/crop peak must reject before decoding frame two.
        assert_eq!(decode(2800).err(), Some(MediaError::Limit));
        assert_eq!(iterations.get(), 2);
        let animation = decode(8192).unwrap();
        assert_eq!(animation.frames.len(), 2);
        assert!(
            animation
                .frames
                .iter()
                .all(|(frame, _)| frame.allocation_bytes() == 512)
        );
    }
    #[test]
    fn crop_removes_private_pixels_and_preserves_alpha() {
        let frame = BgraFrame::new(2, 1, 8, vec![1, 2, 3, 128, 9, 8, 7, 255]).unwrap();
        assert_eq!(
            crop_frame(&frame, NormalizedRect::LEFT_HALF)
                .unwrap()
                .pixels_bgra(),
            &[1, 2, 3, 128]
        );
    }
}
