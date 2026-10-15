# Task05 independent check — 2026-10-14

**Technical delivery ready.** No unresolved task05 code finding; all commands returned. Review used the task manifests, parent contracts, implementation evidence and actual code. No task switch, commit, push, archive or journal operation occurred.

## Findings (fixed)

- `render/src/protocol.rs`, `worker/engine.rs`: `CandidateReady` only identified generation/epoch/per-worker batch nonce. A token surviving shutdown could commit a replacement worker's different candidate when those numbers repeated. Regression failed first with an unexpected successful `GenerationAck` (`independent-token-before.log`, exit101). A private shared `Arc<()>` identity and `Arc::ptr_eq` commit guard now bind tokens to their issuing worker while retaining all existing checks. Public consumer API is unchanged. Targeted regression passed (`independent-token-after.log`); final suites include it.
- `worker/tests/contracts.rs`, `tests/native.rs`: closed concrete coverage gaps for issued-token epoch/reprepare/abort invalidation, failed prerender cleanup preserving active, pixel cache keys versus echoed priority/epoch, all four independent rotation/CropBox corners and typed invalid page/crop/target requests. New tests were split into their own module per the backend guideline.
- Backend specs and task design now state worker-scoped readiness and current CPU-cache ownership. No other task implementation changed.

## Findings (not fixed)

None within task05. Hosted CI execution, real font-substitution quality, non-Windows native runtime, native UI/GPU upload and physical monitor actions remain NOT_RUN/UNVERIFIED. They are not inferred from CPU raster tests. Task10 still owns the main-thread publication/Commit-ack transaction; this review did not implement it. Commit rejects invalid tokens before changing active state, and successful replacement retains the old native document until explicit Retire.

## Verification

| Check | Actual result | Evidence |
| --- | --- | --- |
| Lint / default workspace gate | PASS, `just ci` exit0; 109 tests, none ignored; fmt/clippy/deny pass | `independent-ci-closeout.log` |
| Native clippy, warnings denied | PASS, core/pdf/render feature suite exit0 | `independent-native-clippy-closeout.log` |
| TypeCheck | PASS, workspace/all-targets/all-features/locked/offline exit0 | `independent-typecheck-closeout.log` |
| Native tests | PASS, 42 tests: core21 + pdf3 + render16 + integration2 | `independent-native-closeout.log` |
| Normal native suite without DLL | Expected exit101, `LibraryUnavailable`; harness PASS | `independent-native-missing.log` |
| Artifact/statistical audit | PASS | `independent-artifact-audit.log` |

The first sandbox `just ci` attempt could not open the existing build lock (`independent-ci-before.log`). Approved execution with the existing Cargo cache resolved it. No checks were skipped to avoid that error. All-features dependency-deny evidence is reused from the unchanged dependency graph (`deny-all-features.log`). The capacity and narrow-thumbnail regressions were independently confirmed in their preserved red log and pass in current suites.

## Evidence applicability and ownership review

Before edits, all 88 source, 34 evidence and 3 local-artifact hashes matched `verification-manifest.json`. Five exact pdfium-render0.9.4 source hashes and all 16 native notice hashes also matched. That original manifest/report remain the implementation checkpoint; `independent-check-manifest.json` records current reviewed files and new evidence.

All six 20-warmup/200-sample timing cases recompute exactly: full4K P95 13.7167ms; previous-frame-retained 13.9641ms; wide/tall crops 13.0335/12.3451ms. Crop/FULL median ratios are 0.4658842877 and 0.4571721034. The four 4K cases meet 25ms and both ratios meet 0.8. Earlier failures remain preserved. No benchmark rerun occurred: the worker identity guard changes candidate commit only; native raster, frame constructor and uncached Audience timing path are unchanged. The original whole-tree benchmark snapshot limitation remains explicit in `verification.md`.

PDFium owns byte vectors and documents inside the single worker; safe borrowed bitmap destruction precedes moving the Vec into immutable BgraFrame. No mutable-pixel/native-handle escape, unsafe code, RGBA conversion or user-PDF write path was introduced. Core dependencies remain pure and the audience snapshot has no notes/path/cache access. Scheduling, in-flight stale suppression, candidate nonce isolation and global capacity-based L2/L3 budgets were checked against their actual paths. Runtime pin, explicit path, feature forwarding, checksum/notice acquisition and fail-on-missing native CI configuration remain consistent.
