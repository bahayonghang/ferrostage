# Independent check results

## Findings (fixed)

None. Independent review found no actionable code issue, and this reviewer changed no product code.

## Findings (not fixed)

None within this task's scope. Existing monitor-discovery work was preserved. Release packaging remains the main session's next step and is not claimed complete here.

## Reviewed contracts

- Settings retain schema version 1 and the string path list. `schema.rs:186` records and bounds MRU paths and associated timestamps. `parse/sections.rs:69` validates individual timestamp leaves, then prunes them to effective deduplicated/limited paths. `merge.rs:43` replaces the complete known timestamp-map leaf while preserving unknown general siblings and other sections. Parse/store regressions cover old settings, mixed invalid values, pruning, clear, unknown preservation, future read-only versions and failed save/retry.
- `shell.rs:791` obtains descriptor/core acceptance before replacing the active document and recording a successful-open time. The existing writable guard encloses MRU/time mutation. Read/structure/candidate/core failures preserve previous successful history; reload does not record another successful-open event. Read-only success leaves history and disk bytes unchanged. Failed persistence retains the effective successful event and Unsaved/Retry.
- `shell.rs:48` preserves typed IO/structure errors through asynchronous work and selects actionable catalog copy. Windows sharing/lock codes take precedence over permission-kind classification. Deterministic truncated-PDF coverage reaches the actual asynchronous structure boundary and keeps the active document/history intact. Candidate/renderer/core diagnostics are logged, without interpolating raw parser text into import messages.
- `presenter.rs:52` captures matching metadata for the existing first three paths in source order. `presenter/home_view.rs` uses existing activation/focus/tooltip paths and a shared header/row time-column width. Date conversion handles missing/negative/unrepresentable values without inventing file times or reading filesystem metadata. Actual GPUI input/geometry regressions cover full-row activation, Enter/Space, known/unknown values, font-size 18 and aligned columns. Existing focus, full-path disclosure, settings and workbench regressions remain in the 179-test suite.
- Root DESIGN and backend database/error plus frontend component contracts have been synchronized with the implementation. No new dependency, PDF write, install, commit or push was introduced.

## Verification

All reviewer commands used Rust 1.99.0 explicitly through process-local `RUSTUP_TOOLCHAIN`, `RUSTC`, `RUSTDOC` and toolchain-first `PATH`. This avoids the separate maintained GPUI checkout selecting Rust 1.98.1. No repository toolchain/dependency waiver or cache deletion was used.

| Check | Result | Evidence |
| --- | --- | --- |
| `just ci` | PASS: fmt, workspace Clippy with `-D warnings`, 207 default tests including 48 settings tests, default dependency policy | `check-ci.log` |
| Product Clippy: `cargo clippy -p ferrostage-app -p ferrostage-ui --features gpui,pdfium --all-targets --locked -- -D warnings` | PASS, 59.64s | `check-product-clippy.log` |
| Native PDF regression: `cargo test -p ferrostage-pdf -p ferrostage-render -p ferrostage-document --features pdfium --all-targets --locked` | PASS, 40 tests including actual metadata/notes/raster and isolated missing-library process | `check-native-pdf.log` |
| Full dependency policy: `cargo deny --all-features --locked --offline check` | PASS: advisories, bans, licenses, sources | `check-all-features-policy.log` |
| `git diff --check` | PASS | Reviewer command exit 0; only existing line-ending conversion notices |
| Product build (implementation agent) | PASS, debug `gpui,pdfium` desktop binary rebuilt | `product-build.log` |
| Full GPUI library suite (implementation agent) | PASS, 179 tests, 0 failed | `ui-tests.log` |
| Native normal-window home smoke (main session) | PASS: real WenYuan Serif SC 18, complete aligned known/unknown dates, Chinese partial-PDF guidance, actual mouse open, MRU/time persistence, clean close | `native-results.md` |

Native PDF tests used `FERROSTAGE_TEST_PDFIUM` resolved to the verified staged `target/debug/pdfium.dll`. No test was waived or ignored. Product build, lint and the GPUI library suite together cover the `check-shell` recipe; successful existing build/test evidence was inspected rather than repeated without code changes. Test counts above describe separate suites and overlap; they are not a count of unique tests.

The native observation was performed by the main session and is distinct from this reviewer's source/headless checks. Native keyboard disclosure and Enter/Space are supported by GPUI input tests; they are not claimed as separately observed native OS input. No size/DPI matrix, installation, signing, clean-machine qualification or publishing was performed. Release packaging is recorded separately after these gates.
