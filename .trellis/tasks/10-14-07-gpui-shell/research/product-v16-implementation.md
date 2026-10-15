# V16 approved AccessKit adoption — implementation handoff

The user explicitly approved the reviewed four-file AccessKit0.34.0 patch as the sixth maintained package. This implements that bounded F14 repair. **Native product acceptance is NOT_RUN at this handoff; Gate B remains INCOMPLETE.** All changes remain uncommitted. Root owns serial independent review and all desktop/owned-HWND execution.

## Product changes and reconstruction

- `vendor/accesskit_windows/`: complete published 0.34.0 package (19 files), plus the exact upstream MIT/Apache notices at VCS `c88605b96d04431f9c3c792464a0f2f253480e94`. Original normalized/original Cargo manifests, lock, tests, example, changelog and source copyright headers remain. The published archive does not include separate license files, so those two notices were fetched from that exact revision.
- Only four runtime files differ: `src/util.rs`, `src/context.rs`, `src/node.rs`, `src/text.rs`. Actual ClientToScreen failure becomes a normal propagated Result; only ERROR_INVALID_WINDOW_HANDLE maps to UIA_E_ELEMENTNOTAVAILABLE. Live values, other errors, missing bounding boxes and empty rectangle lists keep their existing semantics. All four coordinate callers propagate the result. No public API, native dependency version, accessibility disable, general error masking or waiver was introduced.
- `Cargo.toml` adds the local crates.io patch and workspace exclusion. Final `Cargo.lock` only removes source/checksum from the same-version AccessKit package. Its dependency list and every other package/edge remain identical to the committed lock; `product-v16-baseline-and-lock.json` verifies this structurally.
- `vendor/README.md` records the sixth package and removal/rebase conditions. Root separately updated AGENTS, ADR-001/008, maintained-dependencies and project/task status.

`product-v16-accesskit-provenance.json` records all upstream/product file hashes, archive checksum, VCS, license URLs and patch identity. The archive SHA is `106c2b961215864d1c2e703ee63269c25c4e80a577ffb2c1017b9c17dcdf83a1`; the approved patch SHA is `93563d304909e2bca657dc0136382ba9179e638b71799d9c6823918ad2a9f2f0`. Reconstruction verifies the original patch bytes, normalizes only its CRLF transport to LF for application to the LF archive, applies every original hunk, and compares all21 product files. No textual patch change was made.

Reconstruction command: `python .trellis/tasks/10-14-07-gpui-shell/research/product-v16-accesskit.py verify`. Local tracking remains ADR-008/F14; no upstream issue/PR was submitted. Remove/rebase only with a separately approved upstream replacement that preserves all four caller contracts and passes regression plus native product checks.

## Tests and actual product checks

| Check | Result |
| --- | --- |
| Archive + approved patch reconstruction,21 complete files | PASS |
| Retained-context real Windows/COM regression harness | PASS,8 tests |
| Root-owned native HWND destroy harness | Compile PASS only; NOT_RUN by implementer |
| Actual GPUI UI/test-support suite on final lock | PASS,82 =59 GPUI+23 pure |
| `just ci` on final lock | PASS,151 tests; fmt, strict default Clippy, tests, default deny |
| Strict app/UI/document all-feature/all-target Clippy | PASS |
| App/UI/document all-feature/all-target typecheck | PASS |
| Unchanged full all-feature locked/offline deny | PASS |
| Final app `gpui,pdfium` locked/offline build | PASS |
| Diff whitespace and frozen hash read-back | PASS |

The maintained narrow harness is generated from the exact product package, with a test-only standalone manifest that removes GUI dev dependencies/example and redirects the private test module. These adaptations never enter the production package. The first six tests preserve V15's real leaf and four actual COM caller positive/negative controls with strong Context retained. Two new tests prove zero BoundingRectangle and null GetBoundingRectangles for missing geometry on both live and invalid HWNDs; text remains `AB`. All eight create no native window. Original upstream GUI tests and external UIA interaction remain NOT_RUN. The known HarfRust unused-function warning remains visible in consumer/build logs; no new lint allowance was added.

V15's actual PDFium/document2 evidence is reused, not rerun: every first-party source, PDFium dependency, DLL and notes fixture is unchanged, and the final lock only changes AccessKit's source. The new source-bound regression and product commands do not establish real presenter notes rendering (task08).

## Preserved preparation failures and corrections

- The first license fetch failed due to sandbox DNS resolution; the exact pinned files were then fetched successfully with approved network access. No alternate license text was invented.
- First extraction into the sandbox-default temp directory hit WinError5. Reconstruction now uses a uniquely created task-local temporary directory, verifies it resolves under research before scoped cleanup, and leaves all historical records intact.
- First `git apply` encountered the approved patch's CRLF versus archive LF mismatch. The script now verifies the approved raw patch hash before line-ending normalization; all textual hunks are unchanged. These preparation failures occurred before vendor creation and are recorded in the session tool transcript, rather than claimed as product test failures.
- Initial `cargo update -p accesskit_windows --offline` also rewrote three unrelated dependency edges (gpu-allocator/windows, iana-time-zone/windows-core, tempfile/getrandom). `product-v16-lock-before-tightening.txt` preserves that intermediate resolution. Those edges were restored before the final-lock GPUI and all remaining product checks. The earlier GPUI82 PASS is retained as intermediate evidence only.
- First harness execution failed with external mbx cache access denied; `product-v16-a11y.log/.json` preserves it. The cache-authorized rerun passed all8 tests. No failure is relabeled PASS.

## Frozen identity and handoff

V15 freeze SHA `0d6a9c2a7bbb10fe6f6c7d24e7c4f7faf10340d456fd5185fdfe631acdc4370f` and all622 sources matched before adoption. No old evidence was overwritten.

- V16 freeze SHA: `0e2b1ed8ef9299676ba249e7600bd8ab6734f00c6ae17f677467405ad60402d3`.
- 648 source/contract inputs, aggregate `3e208cf586dcf37ab0420228e8eb3fba8ee07c0b73a8597c318a7f6a219c35b8`;617 unchanged,5 changed,26 added versus V15.
- EXE: `cf228c03655ddd45d25520bd88dab025c219af9bec9195314ce541dbb1cd89d1`.
- Notes PDF: `e939e35f931c184fd93b2e9b95e560db73ba6a6f6dfccbb0963262f0b359dfeb`.
- PDFium DLL: `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`.

Root's `native/v16-core.toml` is an initial config fixture, excluded from immutable source_files so authorized UI Apply/restart can persist changes. The native plan and runner are source-bound. The frozen harness matches all production source except the explicit test-module wiring; its original upstream source and narrow tests are retained for independent review.

After serial independent review, Root may run the compiled owned-HWND test with `python .trellis/tasks/10-14-07-gpui-shell/research/product-v16-check.py root-hidden`. It creates16 hidden test-owned windows, destroys each while Context/providers remain retained on another thread, and checks64 actual COM calls. The implementer has not run it or launched FerroStage. Root then owns actual live accessibility and titlebar/Presenting teardown with full stdout/stderr/backtrace, preserving V14's F14 failure.

This patch does not establish simultaneous destruction/conversion race elimination, HWND reuse safety, complete provider disconnect/drain, all UIA methods or all product-close behavior. Other native errors are preserved by source control flow, without injecting an artificial non-invalid-HWND error. Physical unplug/reconnect/port/true single-zero tests remain NOT_RUN; size matrix remains user-deferred.07/Gate B is incomplete,08–11 remain gated/P2. No commits, push, archive, install or add_session were performed. All command sessions have returned before handoff.
