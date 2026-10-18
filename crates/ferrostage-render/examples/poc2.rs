//! Controlled CPU raster benchmark; writes only the explicitly supplied report.
use ferrostage_core::{DocumentGeneration, NormalizedRect, PhysicalPage};
use ferrostage_pdf::{PixelSize, RuntimeConfig};
use ferrostage_render::*;
use std::{env, error::Error, fs, path::PathBuf, time::Duration};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("usage: poc2 <pdfium library file> <report.json>".into());
    }
    let worker = RenderHandle::spawn(
        RuntimeConfig {
            library_path: PathBuf::from(&args[0]),
            font_directories: vec![],
        },
        CacheBudgets::default(),
    )?;
    worker
        .load_active(
            DocumentGeneration(1),
            include_bytes!("../../../fixtures/pdf/raster.pdf").to_vec(),
        )?
        .recv()??;
    let cases = [
        ("full_4k", 8, NormalizedRect::FULL, 3840, 2160),
        (
            "full_4k_previous_frame_retained",
            8,
            NormalizedRect::FULL,
            3840,
            2160,
        ),
        (
            "wide_full_same_density",
            1,
            NormalizedRect::FULL,
            7680,
            2160,
        ),
        ("wide_crop_4k", 1, NormalizedRect::LEFT_HALF, 3840, 2160),
        (
            "tall_full_same_density",
            2,
            NormalizedRect::FULL,
            3840,
            4320,
        ),
        (
            "tall_crop_4k",
            2,
            NormalizedRect {
                x: 0.0,
                y: 0.0,
                w: 1.0,
                h: 0.5,
            },
            3840,
            2160,
        ),
    ];
    let mut report = String::from(
        "{\n  \"warmups\":20,\n  \"samples_per_case\":200,\n  \"cache_bypass\":true,\n  \"time_boundary\":\"backend render plus raw owned BGRA export; no queue/upload\",\n  \"cases\":[\n",
    );
    for (index, (name, page, crop, width, height)) in cases.into_iter().enumerate() {
        let request = RenderRequest {
            generation: DocumentGeneration(1),
            page: PhysicalPage(page),
            crop,
            target_px: PixelSize { width, height },
            priority: RenderPriority::Audience,
            epoch: 0,
        };
        let retain_previous = name.ends_with("retained");
        let mut previous: Option<ferrostage_core::BgraFrame> = None;
        let mut samples = Vec::new();
        for iteration in 0..220 {
            let id = worker.submit_uncached(request)?;
            let event = worker.results().recv_timeout(Duration::from_secs(30))?;
            if event.id != id || event.request != request {
                return Err("benchmark identity mismatch".into());
            }
            let result = event.outcome?;
            if result.cache_hit {
                return Err("benchmark unexpectedly hit cache".into());
            }
            let ms = result
                .render_duration
                .ok_or("missing actual render time")?
                .as_secs_f64()
                * 1000.0;
            if retain_previous {
                if let Some(old) = previous.as_ref()
                    && old.pixels_bgra() != result.frame.pixels_bgra()
                {
                    return Err("retained deterministic frame changed".into());
                }
                previous = Some(result.frame);
            }
            if iteration >= 20 {
                samples.push(ms);
            }
        }
        let mut sorted = samples.clone();
        sorted.sort_by(f64::total_cmp);
        let median = (sorted[99] + sorted[100]) / 2.0;
        let p95 = sorted[189];
        if index != 0 {
            report.push_str(",\n");
        }
        report.push_str(&format!("    {{\"name\":\"{name}\",\"page\":{page},\"width\":{width},\"height\":{height},\"median_ms\":{median},\"p95_ms\":{p95},\"samples_ms\":{samples:?}}}"));
        println!("{name}: median={median:.3} ms p95={p95:.3} ms");
    }
    report.push_str("\n  ]\n}\n");
    fs::write(&args[1], report)?;
    worker.shutdown()?;
    Ok(())
}
