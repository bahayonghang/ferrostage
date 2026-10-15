# Task07 bounded GPUI maintenance evaluation

The authorized technical evaluation is complete. The isolated Windows candidate builds, type-checks and passes warnings-denied lint and 43 distinct targeted/upstream tests. The unchanged full `cargo deny` check returns **exit4 / FAIL**: advisories, bans and sources pass, but three licenses remain rejected. This is a reviewable candidate, not an approved product pin, policy exception, shell implementation or Gate B result.

Only `cache/gpui-maintenance-candidate`, task07 `research/maintenance-*` and the task07 implementation checkpoint were changed by this evaluation. Product Cargo files, lock, crates, CI and `deny.toml` are unchanged. The final comparison against06 remains **117/117 inputs unchanged**, including **92/92 classified product inputs**, with no unlisted crate files. Prior01–06 delivery evidence was not rerun or relabeled.

## Candidate and exact change boundary

The base is Zed tag `v1.23.3`, commit `96a1b1b5389efff473fc2306f7f8c690f2e1a417`, with its Apache-2.0 GPUI package0.2.2. This is not crates.io GPUI0.2.x. Only the GPUI crate and two OFL fonts were copied from that checkout; no Zed GPL application/UI or Pympress source was used. The other tagged GPUI/platform/support dependencies remain at this exact tag through Git dependencies.

| Input | Candidate change | Behavior and ownership |
| --- | --- | --- |
| GPUI `Cargo.toml` | resvg/usvg exact0.48.1, skrifa exact0.44.0; isolated workspace inheritance/path rewriting | Retains all inherited dependencies/features except the specified text-stack migration.62 inherited entries and12 original path-package manifests are recorded; the12 are Apache-2.0. |
| GPUI `src/svg_renderer.rs:43` | `ttf_parser::Face::parse(...).glyph_index(ch)` becomes `skrifa::FontRef::from_index(...).charmap().map(ch)` | Keeps the original face index, failure-as-false behavior, font selection and fallback. Seven targeted tests are appended under `cfg(test)`. |
| resvg/usvg0.48.1 | Maintained skrifa/harfrust/fontdb stack | Preserves `text`, `system-fonts`, `memmap-fonts`, `raster-images`; explicitly enables `svgz`, newly feature-gated in0.48, to preserve previous gzip decoding. No image codec was removed. |
| pulp0.22.3, MIT | Manifest only: dependency alias `paste` now selects the real published `pastey =0.2.3` | Keeps EXR/pulp APIs and all Rust source. No upgrade to pulp0.23, no source rename, no advisory-identity disguise. |
| metal0.33.0, MIT OR Apache-2.0 | Same dependency-only pastey replacement | Keeps the complete Apple graph. Actual Apple/Objective-C compilation and runtime remain NOT_RUN. |
| cosmic-text0.19.0, MIT OR Apache-2.0 | Manifest only: fontdb0.23 → exact0.24.0 | Necessary full-graph edge through `gpui_platform → gpui_web → gpui_wgpu → cosmic-text`. Original Rust source and features remain unchanged. |
| GPUI `src/geometry.rs:3723` | Old literal `1.618_034` → `std::f32::consts::GOLDEN_RATIO` | Original tag triggers Rust1.99 `clippy::approx_constant`. One-shot proof shows both f32 values are `0x3fcf1bbd`; no geometry behavior, lint policy or API changes. |

`maintenance-source.patch` contains the exact generated diff, including evaluation-only workspace setup and tests. `maintenance-preparation.json` lists original inherited entries. The preparer copies only into the ignored candidate; shared Cargo registry/Git sources are never edited. Standalone `[workspace]` declarations isolate copied packages from FerroStage's workspace. Retained resolved locks restore the same candidate and validation graphs when preparing again.

The candidate root has four image/macro tests and three font/cosmic tests. A separate `validation/wgpu-consumer` directly depends on the tagged `gpui_wgpu` to compile its actual cosmic-text/fontdb caller on the Windows host; it does not change the candidate's default or audited graph. The fontdb validation copy is unmodified except an isolated workspace declaration.

## Maintained sources and parser ancestry

The official [resvg0.48.1 source/changelog](https://github.com/linebender/resvg/blob/68b14c4c3bccdb60344c777406486b54c36ec1a4/CHANGELOG.md) records the0.48 text-stack migration and0.48.1 patch fixes. Both resvg/usvg published archives identify that commit. Their exact archive checksums, source manifests and licenses are in `maintenance-source-validation.json` and the retained root lock. The GPUI call is checked against published skrifa0.44's `MetadataProvider`, `FontRef::from_index` and `Charmap::map` APIs, then exercised through the actual GPUI renderer, not only a port of its helper.

[pastey0.2.3](https://github.com/as1100k/pastey/tree/c377d07c9d1a418e9922af59dc932736752a95cc) is the actual maintained MIT OR Apache-2.0 fork; its published README documents the `paste = { package = "pastey", ... }` migration. Its changelog records0.2.3 coverage improvements and0.2.2 fixes for None-delimited groups. The full published source/archive is checksum verified. Pulp's upstream maintained branch also uses pastey, but the candidate keeps pulp0.22.3 and replaces only its macro dependency to avoid a broader EXR API migration. Metal still requires the local manifest replacement. Windows pulp/EXR tests and a representative Metal identifier/type-position macro test pass; the latter is not an Apple build substitute.

[fontdb0.24.0](https://github.com/RazrFalcon/fontdb/tree/ecb707e6096b59146b750f66a32c125a43515192), published under MIT, deliberately **inlines a minimal subset of ttf-parser0.25.1**. Its `src/ttf_parser/mod.rs` states the MIT/Apache provenance and limits the subset to font table records, `name` and parts of `OS/2`; the module set is `mod.rs`, `language.rs`, `name.rs`, `os2.rs`, `parser.rs`. The changelog records this decision. The exact archive SHA256 is `2660c5e9157bf76d2db1294e4a9feba604ef610819a3b591088d0d8392a3290f`.

These are two separate findings: (1) the audited graph contains no `ttf-parser` package node and has no associated RustSec advisory failure; (2) a maintained upstream metadata subset still has ttf-parser ancestry. This evaluation does not claim all old parser code is gone or prove comprehensive parser security. The subset is maintained by fontdb upstream; candidate adopters must continue tracking that source, not rely on package-node absence alone. Existing upstream fontdb tests cover one font-load integration case; the additional tests exercise malformed/truncated data, out-of-bounds TTC offsets, collection indices, family/style/weight/monospace metadata and real cosmic shaping/fallback/rasterization. There is no fuzz-campaign or non-Windows system-discovery evidence.

All12 relevant published input archives match checksums from the original route or candidate locks, and all2,004 archive files match the cached registry source bytes. Five copied source packages are compared file by file; differences are only the listed source/manifests, validation locks and maintenance notices. No `.cargo-checksum.json` exists in this Cargo registry cache, so the original `.crate` archive plus lock checksum is the verification source. Exact-tag `git diff --exit-code <revision>` returned0 over GPUI, workspace metadata, the12 inherited manifests and both font directories; corresponding status returned0 with an empty result. HEAD/ref identity alone was not treated as byte verification. See `maintenance-tag-source-check.json`, `maintenance-source-validation.json` and raw logs.

## Final unchanged-policy audit

Exact command: `cargo deny --manifest-path cache/gpui-maintenance-candidate/Cargo.toml --config deny.toml --locked --offline check`. The approved cache-lock retry completed with **exit4**, `advisories ok, bans ok, licenses FAILED, sources ok`; raw log `maintenance-deny-final.log`. No policy file, ignore, target filter, all-features override or source identity was used to hide a rejection. Git-source and duplicate-version warnings remain warnings under the existing policy.

| Remaining rejection | Required dependency path | Attribution | Exact retained notice |
| --- | --- | --- | --- |
| arrayref0.3.9 — BSD-2-Clause | GPUI → resvg0.48.1 → tiny-skia0.12 / tiny-skia-path0.12 → arrayref; also usvg → tiny-skia-path | Required Windows runtime source as well as other platforms | `maintenance-retained/notices/arrayref-0.3.9/LICENSE` |
| libbz2-rs-sys0.2.5 — bzip2-1.0.6 | GPUI → tagged http_client → async-compression0.4.50 → compression-codecs0.4.45 → bzip2 0.6.1 → libbz2-rs-sys | Required Windows runtime source | `maintenance-retained/notices/libbz2-rs-sys-0.2.5/LICENSE` |
| cbindgen0.28.0 — MPL-2.0 | gpui_platform → gpui_macos → gpui_apple → **build dependency** cbindgen | Apple build tool in the preserved full graph; absent from Windows normal/build graph | `maintenance-retained/notices/cbindgen-0.28.0/LICENSE` |

MPL-2.0 is not described as a permissive license. This report requests no exception and applies none; Root owns the separately reviewable policy proposal. Remaining notices are exact published package copies with source/hash records. Neither native compile nor an advisory-only pass converts this full FAIL into delivery PASS.

`maintenance-tree-all.txt` records all-target normal/build/dev edges; `maintenance-tree-windows.txt` omits dev edges to show the actual candidate Windows graph; `maintenance-tree-wgpu-consumer.txt` separately records the API validation graph. The root lock still includes inactive AVIF/rav1e-related `paste1.0.15` metadata. The actual full-policy resolved graph has no active `paste`, `rustybuzz` or `ttf-parser`; no optional feature was disabled to obtain that result. Enabling additional future codecs/features requires a fresh audit and is not pre-approved by this evaluation.

## Verification and limitations

Commands and actual exits are chronological in `maintenance-status.txt`. Every long operation was marked RUNNING before execution; completion came from the returned process. The command below is deliberately left as an expected FAIL until a separate policy decision.

| Check | Actual result | Raw log |
| --- | --- | --- |
| Windows native candidate `cargo build --manifest-path cache/gpui-maintenance-candidate/Cargo.toml --locked --offline` after final source change | exit0 | `maintenance-final-source-build.log` |
| Candidate `cargo clippy --manifest-path cache/gpui-maintenance-candidate/Cargo.toml --all-targets --locked --offline -- -D warnings` | exit0 | `maintenance-final-source-clippy.log` |
| Direct GPUI `cargo clippy --manifest-path cache/gpui-maintenance-candidate/vendor/gpui/Cargo.toml --target-dir cache/gpui-maintenance-candidate/target --no-default-features --lib --locked --offline -- -D warnings` | exit0 | `maintenance-gpui-clippy-final.log` |
| Candidate `cargo check --manifest-path cache/gpui-maintenance-candidate/Cargo.toml --all-targets --locked --offline` | exit0, before the bit-identical constant fix; final build/clippy subsequently compiled it | `maintenance-typecheck.log` |
| WGPU direct consumer `cargo check --manifest-path cache/gpui-maintenance-candidate/validation/wgpu-consumer/Cargo.toml --target-dir cache/gpui-maintenance-candidate/target --offline` | exit0,36.41s; resolved lock retained | `maintenance-wgpu-consumer-typecheck.log` |
| Actual GPUI `cargo test --manifest-path cache/gpui-maintenance-candidate/vendor/gpui/Cargo.toml --target-dir cache/gpui-maintenance-candidate/target --no-default-features --lib svg_renderer --locked --offline` | exit0,14 tests:7 existing +7 new;351 unrelated tests filtered | `maintenance-final-source-svg-tests.log` |
| Candidate `cargo test --manifest-path cache/gpui-maintenance-candidate/Cargo.toml --offline --tests` | exit0,7 tests:4 image/macro +3 font/cosmic | `maintenance-harness-tests.log` |
| Copied pulp upstream `cargo test --manifest-path cache/gpui-maintenance-candidate/vendor/pulp/Cargo.toml --target-dir cache/gpui-maintenance-candidate/target --lib` | exit0,17 tests | `maintenance-pulp-upstream-tests-retry.log` |
| Unmodified fontdb upstream `cargo test --manifest-path cache/gpui-maintenance-candidate/validation/fontdb/Cargo.toml --target-dir cache/gpui-maintenance-candidate/target --tests` | exit0,1 integration test; library has0 | `maintenance-fontdb-upstream-tests-retry.log` |
| Copied cosmic upstream `cargo test --manifest-path cache/gpui-maintenance-candidate/vendor/cosmic-text/Cargo.toml --target-dir cache/gpui-maintenance-candidate/target --lib` | exit0,4 tests; one reads system font discovery | `maintenance-cosmic-upstream-tests-retry.log` |
| Modified Rust/harness `rustfmt --check --edition 2024 ...` | exit0; exact file arguments in status | `maintenance-format.log` |
| Unchanged full `cargo deny ... check` | **exit4 / FAIL, three licenses** | `maintenance-deny-final.log` |

The43 distinct tests count each test once; repeated image/SVG runs are not added. SVG tests execute the actual CPU rasterizer and verify BGRA channel order, alpha, exact geometry pixels, gzip equivalence, real glyph fallback, mixed-font/ligature/combining output and collection indexing. EXR tests generate131×67 images with arbitrary SIMD tails and check every decoded float channel for Uncompressed, RLE, ZIP16 and PIZ; malformed EXR and PNG RGBA paths are also exercised. They do not cover every codec or rendering mode. No GPU display, window creation, focus, fullscreen, multi-monitor transaction or FerroStage UI was exercised.

All initial failures remain: cache/network fetch/build failures; cache-lock deny failure; first upstream tests finding outer workspace membership, followed by unavailable offline dev dependencies; original-tag clippy literal rejection; the mistaken one-shot `PHI` identifier (the correct Rust name is `GOLDEN_RATIO`); and a preparation Git permission error followed by a shell-continuation lint run on unchanged geometry. The status explicitly corrects that latter run; it is not patched-code evidence. A direct `cargo check -p gpui_wgpu` while the package was inactive for the host triggered a Cargo1.99 feature-resolver panic, preserved in `maintenance-wgpu-typecheck.log`; the separate direct-consumer check then passed. The panic is a harness/Cargo selection failure, not a GPUI API error. No failure was relabeled or suppressed. Incremental-cache hard-link warnings used Cargo's copy fallback.

## Fonts, retention and reproduction

The exact two font licenses and copyright notices are retained under `maintenance-retained/notices/fonts`. IBM Plex Sans: Copyright2017 IBM Corp., **Reserved Font Name "Plex"**, SIL OFL1.1. Lilex: Copyright2019 The Lilex Project Authors, SIL OFL1.1; its supplied header names no specific reserved font name. The complete original OFL text, including modification/distribution conditions, is retained for both. Original font paths and hashes are recorded; no Windows proprietary font was copied into review artifacts.

The two-font TTC is deterministically generated by `maintenance-prepare.py`, adjusts SFNT offsets and is used only in ignored local cache. Its original source font hashes and generated hash are retained; the derivative binary is not included in the reviewable task files. This avoids claiming redistribution approval for a format-changed derivative that retains an RFN. Fontdb's original Tuffy test font remains in the ignored validation copy; its original public-domain/fallback-grant notice and hash are also retained. The cosmic upstream system-font test depends on the local Windows font environment and is not a portable golden-output assertion.

Run from the FerroStage root, with the exact recorded Cargo caches available:

```powershell
python -X utf8 .trellis/tasks/10-14-07-gpui-shell/research/maintenance-prepare.py
python -X utf8 .trellis/tasks/10-14-07-gpui-shell/research/maintenance-source-validation.py
```

The preparer restores retained resolved locks for the candidate, GPUI/pulp/cosmic/fontdb tests and WGPU consumer. It requires the recorded tagged source and published archive bytes, never obtains an arbitrary latest branch, edits a shared cache or launches a window. Run the exact checks above; for reproduction with dependencies already cached add `--locked --offline` to the originally online upstream commands. Package-check network/cache access may need the same tool approval as the recorded initial attempts.

`maintenance-capture.py` retains current manifests/locks/main stubs, hashes441 candidate source/fixture files and all maintenance artifacts, captures toolchain identity, generated EXR/TTC/proof/app hashes, and rechecks the117-file product baseline. `maintenance-evidence-manifest.json` is the final integrity index; it excludes its own bytes. The main `implement.md` checkpoint is outside this new evidence index and remains a live recovery note. Original A/B/C spikes, their99-record evidence index and original failures were not replaced.

Next: independent check of this candidate/evidence and a concrete candidate/license decision coordinated by Root. Only after approval and a passing product integration gate may task07 I-2 feature/CI wiring and I-3 onward resume. Product native feature commands, product `just ci`, actual shell/theme runtime, Apple/Linux/Web native runs, Gate B physical unplug/port changes and real single/zero-screen behavior remain **NOT_RUN**. User's unavailable physical operations remain required but unexecuted, never simulated as PASS.
