# Reviewed GPUI maintenance source

These five dependency copies are excluded from the FerroStage workspace. They
retain upstream package licenses and upstream lint configuration; FerroStage
crates retain `unsafe_code = forbid` and warnings-denied lint.

- GPUI0.2.2: Zed v1.23.3, revision96a1b1b5389efff473fc2306f7f8c690f2e1a417,
  Apache-2.0. Maintained resvg/usvg0.48.1 font stack, compatible Unicode cmap
  coverage, and bit-identical Rust1.99 GOLDEN_RATIO constant.
- pulp0.22.3 (MIT), metal0.33.0 (MIT OR Apache-2.0): published pastey0.2.3
  replaces the old paste dependency under its existing alias. Rust code unchanged.
- cosmic-text0.19.0 (MIT OR Apache-2.0): dependency-only fontdb0.24 migration.
- harfrust0.12.0 (MIT): reviewed format2 selection/mapping in two source files.

Authoritative source review and preserved failing/passing regressions:
`.trellis/tasks/10-14-07-gpui-shell/research/maintenance-independent-report.md`.
The final526-source snapshot is
`cc9a8ded6bfeeed1c0286ba5bf2e3ecf750b6ce34987f11d4b6e23d85ec8a960`.
`product-vendor-adoption.json` and `product-vendor-adaptation.patch` in that
research directory record candidate-to-product copying and test path changes.
Product source is not represented as byte-identical to the evaluation snapshot.

`fixtures/fonts` contains only unchanged original OFL fonts and complete notices;
IBM's Reserved Font Name is Plex. TTC derivatives are generated in memory by
tests, never stored or redistributed. `fixtures/svg` contains authored regression
shapes. No evaluation application or generated modified font is included.

Fontdb0.24 retains an upstream-maintained minimal metadata subset derived from
ttf-parser0.25.1. Package advisory absence is not removal of all parser ancestry.
Direct HarfRust strict lint has six reproduced upstream diagnostics; no waiver
or unrelated algorithm change was introduced. Apple native behavior is unverified.
