# Parent integration checkpoint — 2026-10-14

This is a blocked implementation checkpoint, not parent completion. Branch `main`, HEAD `ab78a84d96c7cf4a0bbc8dfd4f8e45da180bafb3`; all work remains uncommitted. No commit, push, archive, journal/add_session or second task tree was created. Actual task directories remain `.trellis/tasks/10-14-<name>` below; parent is `.trellis/tasks/10-14-presenter-v0.3`.

## Child delivery and evidence

| Child name | Implementation / check | Remaining acceptance / recovery |
| --- | --- | --- |
| 01-docs-sync | Documentation independently reviewed; `research/review-2026-10-14.md` | Await review and separately authorized commit/archive; do not restart |
| 02-settings-schema | Settings/schema/NTFS persistence independently verified; `research/check-review-2026-10-14.md` | UI integration belongs to09; implementation need not be repeated |
| 03-monitor-topology | Pure resolution, complete Windows enumeration and polling independently verified; `research/independent-check.md` | Physical unplug/replug, port/same-model/missing-identity, DPI/rotation changes and latency remain NOT_RUN; user cannot operate hardware this turn |
| 04-session-lifecycle | Core lifecycle, injected timer, preview isolation and snapshot boundaries independently verified; `research/independent-check.md` | Native window transaction belongs to07; later checked crop/map refinements are covered by06 |
| 05-pdf-raster | Single PDFium worker, immutable BGRA, candidate/cache protocol, native crop and CPU timing independently verified; `research/independent-check.md` | GPU upload, real font-substitution quality and hosted/non-Windows runtime evidence are not inferred |
| 06-beamer-document-model | Content maps, notes/layout/media metadata, actual Beamer fixtures and independent fixes verified; `research/independent-check.md` | Native UI/controls/playback remain downstream work |
| 07-gpui-shell | I-1 complete; authorized isolated maintenance candidate independently reviewed, two compatibility regressions repaired, final526 files reproducible | I-2 final candidate/three exact exceptions were approved; actual product build/full audit now pass. Original-policy failure is retained historically. Product I-3 onward and theme/window tests NOT_RUN. See final independent maintenance report and parent gpui-license-review.md |
| 08-presenter-workbench | Planning retained; not started | Requires07 and Gate B; real PDF dual-window integration/P1 remain in scope |
| 09-settings-ui | Planning retained; not started | Requires07/08 consumers and window coordinator |
| 10-hot-reload | Planning retained; not started | Requires07/08 and Gate C; P2 remains in scope |
| 11-media-playback | Planning retained; not started | Requires07/08/09 and Gate C; P2 remains in scope |

Trellis states are unchanged by this report: parent planning,01–07 in_progress,08–11 planning. These are distinct from technical verification and remaining manual acceptance. Current root-session task is07; a child session with a null task pointer is not evidence that the root task was never started.

## Technical checks and applicability

Environment: Windows x86_64 MSVC, Rust/Cargo1.99.0, native PDFium151.0.7881.0. Machine inventory is `native-environment.json`; exact PDFium checksum, native notices and fixture/font provenance are in05/06 evidence. Static observation found three displays; it does not prove physical topology transitions.

| Check / command | Actual result | Evidence |
| --- | --- | --- |
| `just ci` | PASS exit0,128 default tests, zero ignored, fmt/clippy/deny PASS | 06 `research/independent-ci-closeout.log` |
| Native core/pdf/render/document suite with `--features pdfium --all-targets --locked --offline` and verified `FERROSTAGE_TEST_PDFIUM` | PASS exit0,63 tests, zero ignored | 06 `research/independent-native-closeout.log`; command family in06 `research/verification.md` |
| Native Clippy `-D warnings` | PASS exit0 | 06 `research/independent-native-clippy-closeout.log` |
| `cargo check --workspace --all-targets --all-features --locked --offline` | PASS exit0 for currently linked product features | 06 `research/independent-typecheck-closeout.log` |
| `cargo tree -p ferrostage-core --edges normal` | PASS pure core boundary | 06 `research/independent-core-tree.log` |
| Native document suite without DLL | Expected exit101 / `LibraryUnavailable`; negative harness PASS | 06 `research/independent-native-missing.log` |
| PDFium CPU render protocol | 1200 measured samples; four4K P95 values12.3451–13.9641ms meet25ms; crop/FULL median ratios0.465884/0.457172 meet0.8 | 05 `research/poc-2-timing.json`, `research/independent-check.md` |
| Two allowed GPUI isolated builds | exit0 each; no executable launched | 07 `research/route-spike-build.log`, `kit-route-build.log` |
| Tagged, kit/component and minimal pre/platform policy audits | exit5 each, advisories/licenses FAILED | 07 `research/route-spike-deny-final.log`, `kit-route-deny.log`, `pre-minimal-deny.log` |
| `task.py validate` for parent and11 children | 12/12 exit0;11 unique children and matching parent links | `checkpoint-context-validation.log/json` |

The final06 manifest SHA256 is `6d00530903376138990099497bf797ee847aac6e6b233905a7da76dbfdcadf47`, with source snapshot `e4bef9aaf6b86d859e6d79023b81ed13bf48d58de601af1bda4649dc49e7e46c`. Task07 implement comparison found117/117 inputs unchanged, including92/92 product/build inputs, and no additional crate files. The independent07 audit confirms all117 inputs,43 prior evidence files and10 external inputs match, and all86 current crate files are covered. It also passed incremental fmt/clippy/typecheck and reproduced minimal full-policy exit5; see07 `research/independent-check.md` and `independent-hash-audit.json` (SHA256 `7ab294c93f0beb5ddcb9b6b8d80a6f214a5d30f927b483d5e333587dc4801262`). Existing product checks were not needlessly rerun after documentation-only work, and are not relabeled as task07 product acceptance.

## Gates and parent acceptance

| Gate | Status | Reason / required next evidence |
| --- | --- | --- |
| A | PASS, technical contract scope | Latest128-test CI includes90 core/settings/platform integration cases; details in `gate-a.md` |
| B | BLOCKED / native workflow NOT_RUN | GPUI product dependency gate fails; no shell windows. Physical unplug/port changes unavailable. Actual placement, focus, Start/Stop, recovery and timing still required |
| C | NOT_RUN, blocked byB/08 | Real≥80-page Beamer,≥30-minute presentation, notes isolation, frame latency and GPU/VRAM measurements not performed |
| D | NOT_RUN, blocked by08–11 | Settings UI, replacement reload, media lifecycle/performance and final integration not performed |

All parent AC-1…AC-10 remain unchecked. AC-4 has01's semantic audit; AC-5 has core/document isolation; AC-6 has settings storage; AC-7 has current default/PDFium checks; AC-10 has design-token contrast calculations. These subsets do not satisfy the full end-to-end criteria. AC-1/2/3 still require actual UI/topology behavior; AC-8/9 require reload/playback; M1 and M2 are not achieved.

Root `DESIGN.md` records the requested Claude style and51 passing contrast pairs. Tasks07/08/09 have shared ownership and native visual criteria. No Claude-themed native screen, screenshot or runtime control-state acceptance is delivered yet. A browser mockup is not substituted.

## Recovery

The bounded ADR-008 maintenance evaluation was approved and its final independent review is complete. The user approved the concrete final candidate and three exact exceptions in `research/gpui-license-review.md`; this subsequent approval is distinct from the earlier maintenance-only scope. Actual product I-2 build and full all-features dependency audit pass; no advisory/lint ignore is approved. Only the approved exact exceptions were added, with other policy values unchanged. Resume07 after the passed I-2 product policy/build checks, continuing I-3–I-5/I-7; preserve pending asynchronous activation/fullscreen confirmation requirements from the source review. Do not restart01–06.

Even if07 can subsequently be implemented, the explicit unavailable physical operations keep Gate B incomplete. Follow its declared blocking scope; do not start08 real dual-window integration or silently remove10/11. Save every new check and source fingerprint before a later resume. Commit/archive remain separately unauthorized closeout actions.

The user explicitly approved the concrete repaired candidate/three exceptions. The existing shell_implement agent is active on07 I-3–I-5 after actual I-2 gates returned0; no other child is being implemented. Root alone handles later native UI verification. Current source and running commands must be re-audited before any resume.

Final coordination checks: `checkpoint-final-checks.json` records task07 context validation and `git diff --check` exit0, seven valid local document links, no trailing whitespace in the checked root documents, matching07 final audit hash, unchanged branch/HEAD and current07. The earlier full-tree validation remains12/12. Dirty path/stat snapshots are retained under `checkpoint-final-*.log`; later writes only append these result records.

## Maintenance review checkpoint — 2026-10-15T00:23:40.256033+00:00

The [final07 report](../../10-14-07-gpui-shell/research/maintenance-independent-report.md) defines the reviewed candidate. Its only reconstruction entry includes both GPUI helper and HarfRust repairs; the original maintenance patch alone is historical/defective. Final526 source files reconstruct identically and753 manifest records match Root read-back. Review manifest SHA256 is `4cc87ec3a4f2fe7bbad8757eefd78869be8ae62d7f73e3a6e027e3201ea23bba`.

Actual GPUI16, root7, HarfRust205 tests and separate4 probes pass; probes overlap coverage and are not summed. Root/GPUI strict lint, typecheck, Windows WGPU host compile and Windows build pass. Direct HarfRust strict lint still exits101 with the same six messages/locations as untouched upstream; no unrelated source repair or allow was added. Final full original deny exits4 only for arrayref0.3.9 BSD-2-Clause, libbz2-rs-sys0.2.5 bzip2-1.0.6 and cbindgen0.28.0 MPL-2.0. Exact proposal and attribution are ready; policy SHA remains unchanged.

Current product inputs still match06, so the earlier128/63 suites remain applicable and were not rerun for isolated maintenance/doc work. Parent+11 task contexts validated12/12;07 was revalidated after adding the final report (33 implement/30 check entries). Gate A PASS; B BLOCKED; C/D NOT_RUN; all AC-1…10 remain unchecked. No physical screen operations, product UI/theme/GPU acceptance, commit/push/archive or journal was performed.

Product-adoption follow-up: I-2 actual build and full all-features deny returned0; exact policy delta was independently read back with every other parsed policy field unchanged (`gpui-license-application-check.json`). Runtime shell/theme implementation and final product regressions are in progress. Historical candidate128/63 baseline statements above do not establish checks for newly changed product files.
