# Reviewed GPUI maintenance source

These six dependency copies are excluded from the FerroStage workspace. They
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
