# Packaging

Not implemented yet. The first package is a Windows 11 executable.

- `just install` runs `cargo install --path crates/ferrostage-app --locked`.
- PDFium, when the raster crate starts linking `pdfium-render` 0.9.4, ships beside the binary. It is not part of this skeleton.
- URW Base35 fonts, if required for a fallback, belong under `assets/fonts/urw-base35/` and must carry a license that `cargo deny` accepts. The directory currently holds only a README.
- The app does not embed a telemetry endpoint. Settings stay in a local TOML file.
