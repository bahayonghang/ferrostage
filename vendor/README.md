# Reviewed GPUI maintenance source

The original six dependency copies are excluded from the FerroStage workspace. They
retain upstream package licenses and upstream lint configuration; FerroStage
crates retain `unsafe_code = forbid` and warnings-denied lint.

- GPUI0.2.2: Zed v1.23.3, revision96a1b1b5389efff473fc2306f7f8c690f2e1a417,
  Apache-2.0. Maintained resvg/usvg0.48.1 font stack, compatible Unicode cmap
  coverage, and bit-identical Rust1.99 GOLDEN_RATIO constant. The task07 V8
  retirement repair adds an owning-Window lifetime token and weak active-status
  callback guard in `src/window.rs`; live callback errors retain their logger.
- pulp0.22.3 (MIT), metal0.33.0 (MIT OR Apache-2.0): published pastey0.2.3
  replaces the old paste dependency under its existing alias. Rust code unchanged.
- cosmic-text0.19.0 (MIT OR Apache-2.0): dependency-only fontdb0.24 migration.
- harfrust0.12.0 (MIT): reviewed format2 selection/mapping in two source files.
  The free function `shape` compiles only with `experimental_font_api`, matching
  its existing re-export. `Shaper::shape` is unchanged.
- accesskit_windows0.34.0 (MIT OR Apache-2.0): approved F14 four-file patch
  propagates actual ClientToScreen failure through all four coordinate callers.
  Only ERROR_INVALID_WINDOW_HANDLE maps to UIA_E_ELEMENTNOTAVAILABLE; live
  values, other errors and empty rectangles/ranges retain upstream semantics.
  Original production manifest, tests, examples and notices are preserved.
  `product-v16-accesskit-provenance.json` records archive/VCS/patch hashes and
  exact source inventory; `product-v16-accesskit.py verify` reconstructs it.
  Both files are in the task07 research directory. The local patch is tracked
  by ADR-008/F14; no upstream issue or PR has been submitted. Remove it only
  after separately approving an upstream replacement and rerunning all four
  caller regressions and native product close checks. This bounded error
  repair does not establish HWND reuse safety or complete provider retirement.

Authoritative source review and preserved failing/passing regressions:
`.trellis/tasks/archive/2026-10/10-14-07-gpui-shell/research/maintenance-independent-report.md`.
The final526-source snapshot is
`cc9a8ded6bfeeed1c0286ba5bf2e3ecf750b6ce34987f11d4b6e23d85ec8a960`.
`product-vendor-adoption.json` and `product-vendor-adaptation.patch` in that
research directory record candidate-to-product copying and test path changes.
Product source is not represented as byte-identical to the evaluation snapshot.

The later V8 lifetime delta is bounded by ADR-008 and backed by the task07
`product-v8-retirement-red-activation.log` / `product-v8-retirement-green.log`
real-GPUI regressions. New product-v8 source/build and independent review records
govern that revision; the original526-source snapshot remains historical. This
does not change `gpui_windows`, add a maintained package or expand policy waivers.

`fixtures/fonts` contains only unchanged original OFL fonts and complete notices;
IBM's Reserved Font Name is Plex. TTC derivatives are generated in memory by
tests, never stored or redistributed. `fixtures/svg` contains authored regression
shapes. No evaluation application or generated modified font is included.

Fontdb0.24 retains an upstream-maintained minimal metadata subset derived from
ttf-parser0.25.1. Package advisory absence is not removal of all parser ancestry.
Direct HarfRust strict lint has six reproduced upstream diagnostics; no waiver
or unrelated algorithm change was introduced. Apple native behavior is unverified.

## Jiff migration

FerroStage uses Jiff 1.0.1 for wall time. The [maintenance scope](../.trellis/spec/frontend/maintained-dependencies.md) records the approval for #10. The application stores recent-open times as Unix seconds. The GPUI feature enables system zone discovery and the platform time-zone database, including Windows.

| Package and source | Local changes | Regression coverage |
| --- | --- | --- |
| GPUI 0.2.2, scheduler 0.1.0 and zlog 0.1.0, [Zed v1.23.3](https://github.com/zed-industries/zed/tree/96a1b1b5389efff473fc2306f7f8c690f2e1a417/crates), Apache-2.0 | GPUI's standalone workspace supplies the tagged dependencies and lint settings to both extracted packages. The application patches all Git references to those same sources. `platform_scheduler.rs` and scheduler's `clock.rs` share `jiff::Timestamp`. The scheduler still measures elapsed time with `Instant`. GPUI's `window/a11y/debug.rs` emits a numeric local offset and three fractional digits. All three zlog sinks use the same seconds-and-offset format in `sink.rs`. | Scheduler tests cross midnight at nanosecond precision and reset UTC independently of elapsed time. The zlog test checks its local timestamp prefix. GPUI tests the timestamp in the serialized accessibility frame. |
| pdfium-render 0.9.4, [registry release](https://crates.io/crates/pdfium-render/0.9.4), MIT OR Apache-2.0 | Both manifests replace the date dependency and reflect Jiff's Rust 1.85 minimum. `src/utils.rs` formats PDF dates. Under `src/pdf/document/page/`, `annotation.rs`, `annotations.rs`, `annotation/private.rs` and `field/private.rs` use UTC timestamps for creation and modification. The public setters take `jiff::Timestamp` and the getters still return the PDF strings. | Format tests cover UTC, signed offsets, leap day, pre-epoch time and truncation to seconds. The application's native `annotation_dates` test writes and reads both fields on an in-memory PDF with the pinned DLL. |

The PDF wrapper archive has SHA256 `8948a803616a9e936b15a6637af2cd48c5fb8ae0fcdeb3c32eac3a540e255a19` at upstream commit `6cee8b9a3951832ac0ff62ce4c32800278001cb8`. The copy retains its VCS metadata, license, headers and generated ABI bindings. Cargo resolves it through the application lockfile. The 0.9.5 release still requires chrono. The upstream proposal targets a different wrapper revision and Jiff 0.2. At the time of migration, Zed main also still required chrono in both packages. [PDFium upstream proposal](https://github.com/ajrcarey/pdfium-render/pull/287) · [Zed scheduler](https://github.com/zed-industries/zed/blob/b51ffab0153f3372b71d1bd565d48d5203fefe6a/crates/scheduler/Cargo.toml) · [Zed zlog](https://github.com/zed-industries/zed/blob/b51ffab0153f3372b71d1bd565d48d5203fefe6a/crates/zlog/Cargo.toml)

The wrapper also carries two build-warning fixes. The grayscale scanline helper compiles with `image_api` or in unit tests. The public `PdfPageObjectMark` retains its opaque handle and lifetime, with the unused private constructor and accessor removed.

Run the targeted checks from the repository root. The wrapper's upstream unit tests also compile image tests, so that invocation enables `image_025`. The product uses only `pdfium_7881`.

```sh
cargo test --manifest-path vendor/gpui/Cargo.toml --locked -p scheduler -p zlog --lib jiff_
cargo test --manifest-path vendor/gpui/Cargo.toml --locked -p gpui --no-default-features --lib jiff_capture
cargo test --manifest-path vendor/pdfium-render/Cargo.toml --no-default-features --features pdfium_7881,image_025 --lib test_date_time_to_pdf_date_string
```

Run `just test-native` with the verified PDFium library, `just check-shell` and `just ci`. Check all targets in the product graph with `cargo metadata --locked --all-features --format-version 1`, `cargo tree --locked --workspace --all-features --target all` and `cargo deny --all-features --locked check`.
