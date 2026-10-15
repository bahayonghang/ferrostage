# Task06 independent check — 2026-10-14

**Technical delivery ready.** No unresolved finding in the reviewed task06 implementation. The main session approved the coordinated crop-setter API repair described below. Review used the actual task/spec artifacts, primary PDF tables, implementation checkpoint and cross-crate paths. No task switch, commit, push, archive or journal operation occurred.

## Findings (fixed)

- `crates/ferrostage-pdf/src/structure/source.rs`: a present malformed Filespec `/EF` was ignored and the extractor returned an external filename. The new regression observed `Some(External("clip.mp4"))` instead of an unresolved source. Invalid or dangling `/EF` now yields a Malformed diagnostic and no source; PDF null still behaves as an absent entry. Both Screen and RichMedia source paths are covered.
- `crates/ferrostage-pdf/src/structure/media.rs`: unsupported Movie `/Rate` and `/FWScale` were diagnosed only in the annotation activation dictionary, omitting the selected Movie action whose parameters override it. Both dictionaries now receive the same check. This adds diagnostics, not support or execution of those parameters.
- `crates/ferrostage-pdf/src/structure/tests.rs`: both defects failed before the fixes (`independent-media-red.log`, exit101) and pass afterward (`independent-media-green.log`). The initial sandbox attempt failed before compilation on the existing Cargo build lock; the approved existing-cache retry reached the two intended assertions.
- `crates/ferrostage-core/src/state.rs`, `state/lifecycle_tests.rs`, `state/mapping_tests.rs`: the pre-existing public `set_slide_crop` bypassed checked descriptor geometry. After the main session approved the API change, an invalid-crop regression failed first with a NaN crop installed (`independent-crop-red.log`, exit101). The method now returns `Result<(), CoreError>`, uses the existing `is_inside_unit_square` predicate, and returns `InvalidDocumentCrop` before mutation. The sole existing caller handles the result; valid behavior is unchanged. The core suite then passed all28 tests (`independent-crop-green.log`), and final full/native gates were rerun. Parent/04/06/08 contract coordination belongs to the main session.
- `crates/ferrostage-core/src/state/mapping_tests.rs`: added direct coverage for reconfiguration while a preview is active, with paused Presenting and frozen Ready timers. A mismatched generation preserves the complete state; successful reconfiguration clears preview, preserves the committed content page, stage, pause state and elapsed time.
- `docs/beamer-notes.md`: removed the stale opening claim that the model was not implemented, retaining the UI/playback integration boundary. Backend specs and the media field contract now record the two diagnostic rules.

## Findings (not fixed)

None. The initially deferred public crop setter was subsequently approved for a coordinated task06 repair and is now fixed and verified.

Native UI/screenshots, playback, physical monitor actions, non-Windows runtime and hosted CI execution remain NOT_RUN/UNVERIFIED. Task03 manual acceptance and Gate B remain pending. These are not model failures and are not inferred from native CPU fixtures. Tasks08/10/11 retain import/publication/reload/playback integration ownership.

## Verification

| Check | Result | Evidence |
| --- | --- | --- |
| Lint and default workspace gate | PASS, `just ci` exit0: fmt, warnings-denied clippy, 128 tests, deny; zero ignored | `independent-ci-closeout.log` |
| Native clippy | PASS, core/document/pdf/render, pdfium feature, all targets, locked/offline, warnings denied, exit0 | `independent-native-clippy-closeout.log` |
| TypeCheck | PASS, workspace/all-targets/all-features/locked/offline, exit0 | `independent-typecheck-closeout.log` |
| Native tests | PASS, exit0; 63 tests: core28 + document5 + document-native1 + pdf11 + render16 + render-native1 + isolated-missing1 | `independent-native-closeout.log` |
| Normal native document suite without DLL | Expected cargo exit101 with `Pdf(LibraryUnavailable)`; explicit negative harness PASS exit0 | `independent-native-missing.log` |
| Core production dependencies | PASS, only thiserror/tracing and their transitive graph; proptest is dev-only | `independent-core-tree.log` |
| Diff whitespace | PASS, `git diff --check` exit0; existing line-ending notices only | `independent-diff-check.log` |
| Source/fixture/runtime applicability | PASS; details and final hashes in independent manifest | `independent-artifact-audit.log`, `independent-check-manifest.json` |

The implementation's original logs and manifest remain an immutable checkpoint. Their original counts were 124 default / 59 native. Independent tests add four cases, giving 128 / 63; no existing test was ignored or removed. The dependency graph is unchanged by this review. The previous all-features dependency-license result remains applicable, and current `just ci` deny also passed.

## Evidence and contract review

Before review edits, all 117 source and 24 evidence hashes matched implementation manifest SHA256 `ace3cfd20730468bd37b9d20d925abc4d2d9dfe2278200aa984ff70c76894222`. All 19 fixture/source hashes and 10 external input hashes matched provenance, including the two cached primary PDF references, TeX/font inputs and PDFium DLL SHA256 `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`. The final manifest captures current source and evidence separately. Whole-workspace dirty state includes prior tasks; HEAD alone is not the reviewed source identity.

The 16 task05 frame/native/raster/render source files in its independent manifest are unchanged. This review does not rerun the 1200-sample task05 performance protocol or make a new timing claim: these fixes affect read-only structure diagnostics and a direct state crop setter, which is not on the task05 worker raster/performance path. Native raster and actual Beamer crop/full pixel regressions were rerun.

Checked actual flow: owned PDFium metadata plus lopdf geometry/annotations -> document labels/layout/notes/media -> validated content-only SlideMap and crop descriptor -> core navigation/snapshot. Core still has no document/PDF/GPUI dependency. Audience snapshot has no labels, map, notes, source paths, parser handles or cache access. `project` delegates directly to core. Reconfigure preserves generation/count and timer/stage; commit requires a strictly newer generation; both install only validated content maps and clear preview. Invalid commands validate before mutation.

The source-authored fixtures and primary PDF1.7 tables9.9/9.13–16/9.20–24/9.30–31, Movie action table8.59 and Adobe Extension3 tables9.50a/9.51–9.51c agree with the declared subset. All rotations, inherited visible bounds, clipping, layout overrides, exact notes suffix matching, ordered multi-notes and noncontinuous label groups were checked. Actual Beamer second-screen labels `1,1` remain unmodified; `physical_pages` is the documented recovery. Unsupported/unscaled values remain explicit. No file path is opened, action executed, user PDF written or compiler integrated by product code. Fixture and font provenance are independent of viewer implementation source.

All verification commands have returned; no active command remains.
