# Build log

2026-10-15. Product code uses the approved plan. Commands ran through `C:\Users\lyh\AppData\Local\mbx\bin\cargo.exe` unless noted.

## Icon

- Source generation saved a 1024×1024 image. Pillow 12.3.0 wrote `assets/icons/ferrostage.png` and `assets/icons/ferrostage.ico`.
- ICO directory: count 4, sizes 16, 32, 48, 256.
- `Cargo.lock` gained one direct edge, `embed-resource` 3.0.12. No version change.
- `cargo build -p ferrostage-app --features gpui,pdfium --offline --message-format json` compiled `harfrust` and finished with 0 warnings and 0 errors.
- `ExtractIconExW` on `target\debug\ferrostage.exe` returned icons. Drawn 32px image: 68 clay pixels, 0 blue pixels.
- Live presenter window title: `FerroStage — 未打开文档 — FerroStage`. Class icons `GCLP_HICON` / `GCLP_HICONSM` were non-null. Drawn 32px class icon: 71 clay pixels, 0 blue pixels. `PrintWindow` of the top 80px is `titlebar.png`.

## Warning

- Same JSON build, after deleting the previous `harfrust` rlib, recompiled `harfrust` and reported no `shape` warning.
- Final `cargo build -p ferrostage-app --features gpui,pdfium --offline --locked` finished in 1.16s with 0 warnings.
- `cargo check -p harfrust --features experimental_font_api --offline --locked` exits 101: `cannot specify features for packages outside of workspace`. `harfrust` stays excluded.
- A temporary `ferrostage-app` dev-dependency enabled the empty `experimental_font_api` feature. `cargo check -p ferrostage-app --tests --features gpui,pdfium --offline` printed `Checking harfrust` and exited 0, with no `shape` warning. The dev-dependency and its lock hunk were removed. The following locked product build stayed feature-off.

## Gates

- `cargo clippy -p ferrostage-app -p ferrostage-ui --all-targets --features gpui,pdfium --locked --offline -- -D warnings`: exit 0, 0 warnings.
- `cargo deny --all-features --locked --offline check`: exit 0. `advisories ok, bans ok, licenses ok, sources ok`. Duplicate-crate warnings are the existing warn-level report; `embed-resource` stays 3.0.12.
