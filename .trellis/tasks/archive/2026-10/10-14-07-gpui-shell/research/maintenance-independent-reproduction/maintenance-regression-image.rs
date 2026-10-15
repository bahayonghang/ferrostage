#![forbid(unsafe_code)]

use exr::prelude::*;
use std::io::Cursor;

fn expected(x: usize, y: usize) -> (f32, f32, f32, f32) {
    (x as f32 / 128.0, y as f32 / 64.0, -0.25, 1.0)
}

fn fixture(compression: Compression) -> Vec<u8> {
    let layer = Layer::new(
        (131, 67),
        LayerAttributes::named("FerroStage maintenance regression"),
        Encoding {
            compression,
            ..Encoding::SMALL_LOSSLESS
        },
        SpecificChannels::rgba(|position: Vec2<usize>| expected(position.x(), position.y())),
    );
    let mut bytes = Cursor::new(Vec::new());
    Image::from_layer(layer)
        .write()
        .to_buffered(&mut bytes)
        .unwrap();
    bytes.into_inner()
}

#[test]
fn exr_lossless_decoding_preserves_all_float_channels_and_simd_tails() {
    for compression in [
        Compression::Uncompressed,
        Compression::RLE,
        Compression::ZIP16,
        Compression::PIZ,
    ] {
        let encoded = fixture(compression);
        let artifacts = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("artifacts");
        std::fs::create_dir_all(&artifacts).unwrap();
        std::fs::write(artifacts.join(format!("{compression:?}.exr")), &encoded).unwrap();
        let decoded = image::load_from_memory_with_format(&encoded, image::ImageFormat::OpenExr)
            .unwrap()
            .into_rgba32f();
        assert_eq!(decoded.dimensions(), (131, 67));
        for (x, y, pixel) in decoded.enumerate_pixels() {
            let (r, g, b, a) = expected(x as usize, y as usize);
            assert_eq!(pixel.0, [r, g, b, a], "{compression:?} at ({x}, {y})");
        }
    }
}

#[test]
fn truncated_exr_is_an_error() {
    let encoded = fixture(Compression::PIZ);
    assert!(
        image::load_from_memory_with_format(
            &encoded[..encoded.len() / 2],
            image::ImageFormat::OpenExr
        )
        .is_err()
    );
}

#[test]
fn png_decode_retains_rgba_channel_order() {
    let pixels = image::RgbaImage::from_fn(7, 3, |x, y| {
        image::Rgba([x as u8 * 32, y as u8 * 64, 17, 128])
    });
    let mut bytes = Cursor::new(Vec::new());
    pixels
        .write_to(&mut bytes, image::ImageFormat::Png)
        .unwrap();
    let decoded = image::load_from_memory(bytes.get_ref())
        .unwrap()
        .into_rgba8();
    assert_eq!(decoded, pixels);
}

#[test]
fn maintained_macro_supports_metal_identifier_type_positions() {
    // Exact token-pasting forms used by metal's foreign_obj_type macro. This
    // checks expansion on Windows, not Objective-C ABI or a Metal device.
    struct BufferRef(u32);
    macro_rules! getter {
        ($owned:ident) => {
            impl paste::paste! { [<$owned Ref>] } {
                fn value(&self) -> u32 {
                    self.0
                }
            }
        };
    }
    getter!(Buffer);
    assert_eq!(BufferRef(7).value(), 7);
}
