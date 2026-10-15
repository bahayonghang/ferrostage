# Task06 implementation verification — 2026-10-14

**Ready for independent check.** Core content navigation, document-local notes/layout/overlay mapping, read-only media metadata and checked fixtures are implemented. All commands have returned. Task remains in_progress pending independent review. No task switch, commit, push, archive or journal operation occurred. Branch main, HEAD `ab78a84d96c7cf4a0bbc8dfd4f8e45da180bafb3`; prior tasks' uncommitted work is preserved.

## Shared API and change scope

- Core adds validated `SlideMap`, TOP/BOTTOM crops and checked `DocumentDescriptor::{identity,mapped}` with mandatory private map/crop. `LoadMappedDocument` loads Ready, `ReconfigureDocument` requires equal generation/count and preserves stage/timer, `CommitDocument` remains strictly newer. Preview never projects; all movement/clamping stays on content. `StageShadowDocument` retains the current mapped navigation/crop. Existing core call sites/tests migrated; no audience snapshot field or BGRA storage change.
- Document adds `LoadedDocument::{load,build,reload,descriptor}`, original-label candidates, ordered per-content `NotesMapping`, five layouts/source/warnings and document-local overrides. `notes_for` retains Markdown semantics. Native metadata and structure geometry must agree; invalid/empty maps are typed errors. `project` still delegates only to core audience_snapshot.
- PDF adds owned lopdf structure/media extraction in `structure` modules. Exact field paths, primary tables, supported subset and unresolved-value policy: `media-field-contract.md`. No native raster algorithm, scheduler, media backend, UI or compiler integration was changed.
- Document's optional pdfium test feature forwards task05; existing native CI/just commands now include document. Cargo.lock only gains internal document dependency edges; no external pin or license policy changed.

## Actual fixtures / PoC3

| Case | Actual versus expected |
| --- | --- |
| A overlays.pdf | Actual Beamer16:9 with pause, metadata453.543x255.118pt; labels1/1/2; groups[0,1],[2], three steps for two frames. PASS. |
| B notes-right.pdf | Actual second-screen907.087x255.118pt -> Right, slide LEFT_HALF/notes RIGHT_HALF. Both rendered512x288 crops match respective1024x288 full-page halves within1 BGRA byte, with nonempty/distinct content. PASS. |
| B producer limitation | Two source frames actually emit labels1/1, so heuristic one group is expected; physical_pages produces two. No forced labels or postprocessing. Explicit regression checks this limitation. |
| C independent.pdf | Authored labels produce content1/4/6/8/9/11/12/13; seven groups; notes1->[2,3],4->[5]; orphans0/7/10; exact01 versus1/non-numeric labels and noncontinuous repeated Intro preserved. PASS. |
| D media.pdf | Inherited CropBox[20,30,380,270],90deg -> visible240x360; first rect[0,0,.5,.5]. Movie and Screen2s/8s/Forever/autostart; embedded bytes match fixture-not-video. RichMedia Video asset/activation extracted, times/repeat unresolved; unscaled120 ticks retained, zero scale/Palindrome/Flash diagnosed. PASS. |
| Actual movie.pdf | Beamer emits Start[256,128],Duration[1024,128],Repeat; Widget/AA/PO Movie action identifies unique title. Recover2s/8s/Forever/autostart. Ambiguous-title/action-override regressions pass. |
| E geometry/counterexamples | bottom-wide8:9, bottom-classic2:3 and rotated top fixture -> Bottom candidate; wide/portrait ordinary content intentionally collides with heuristic; explicitNone restoresfull. Ordinary16:9/tallportrait outside bands ->None; mixed sizes/rotation ->None warning. Five manual overrides and new-load/reset versus same-file/reload preservation pass. |

`fixtures/beamer/expected.json`, native test assertions and the primary field table contain expectations. Actual PDFium metadata came through the task05 single render worker, not fabricated metadata. Full13-PDF stdout is in native-movie.log (before the final activation support); final assertions include the activation path and pixel comparisons in native-final.log. Initial logs are retained as historical checkpoints, not presented as final behavior.

## Commands and actual results

| Command | Result / evidence |
| --- | --- |
| `just ci` | PASS exit0: fmt, workspace/all-targets clippy -D warnings,124 default tests, cargo deny. ci-final.log. Existing duplicate dependency warnings remain nonfatal. |
| `cargo test -p ferrostage-core -p ferrostage-pdf -p ferrostage-render -p ferrostage-document --features pdfium --all-targets --locked --offline` | PASS exit0:59 tests=core26+document5+document-native1+pdf9+render16+render-native1+isolated-missing1. native-final.log. Zero ignored. |
| `cargo clippy -p ferrostage-core -p ferrostage-document -p ferrostage-pdf -p ferrostage-render --features pdfium --all-targets --locked --offline -- -D warnings` | PASS exit0, clippy-final.log. |
| `cargo check --workspace --all-targets --all-features --locked --offline` | PASS exit0, typecheck.log. |
| `cargo tree -p ferrostage-core --locked --offline` | PASS exit0, core-tree.log: thiserror/tracing production only, proptest dev; no IO/GPUI/PDF dependency. |
| Normal document native suite with nonexistent configured DLL | Expected exit101 with Pdf(LibraryUnavailable), native-missing.log. Explicit log audit confirmed the typed reason. No missing-runtime success/ignore. |
| Development-only pdflatex, three sources | PASS exit0; beamer-compile.log/beamer-movie-compile.log. Compilation alone was not the model acceptance. |
| `git diff --check` | PASS exit0 (line-ending notices only); artifact audit and manifest record final files. |

The initial sandbox Cargo attempt failed to open the existing build lock (core-initial.log,exit101). Approved existing-cache runs reached compilation. Implementation compile/style failures and corrected successful runs are preserved in core-retry/model-initial/clippy-initial and later logs. The first negative-test log check used incorrect Select-String positional arguments; the corrected explicit -LiteralPath/-Pattern audit confirmed the expected failure. A documentation script initially used Windows' default encoding and stopped before editing design.md; the UTF-8 correction completed it. These are not missing tests or silently discarded failures.

## Provenance and boundaries

`fixture-provenance.json` records exact source/PDF/generator/tool/font/spec/DLL hashes. `verification-manifest.json` hashes the reviewed source/docs/evidence and records the whole current compilation context; previous tasks remain uncommitted, so HEAD alone is not this implementation's identity.

TeX Live2025 pdfTeX3.141592653-2.6-1.40.27, Beamer3.72(2025-02-04), pgfpages0.02, multimedia0.02. Actual TeX PDFs embed subset CMSS10/CMSS12 and CMSS9 for notes-right. pdffonts and source-font headers identify AMS Type1 Computer Modern Sans; installed AMSFonts OFL1.1 covers these fonts and explicitly excludes created documents from font-license inheritance. Source-font/notice hashes are recorded; no standalone fonts or TeX implementation are added. Synthetic PDFs have no fonts. This is separate from task05's original Type3 fixture.

No external media path is opened, PDF action/script executed or user PDF written. Native tests read repository fixtures and structure tests mutate only in-memory copies of authored fixtures. Embedded fixture bytes are not playable media. Exact producer labels, heuristic false positives and unsupported structures remain visible; missing timing is not guessed. Full reload publication/retirement belongs to10; real-import/UI reconfiguration belongs to08; playback belongs to11. Native UI/screenshots/GPU upload, physical monitor actions, non-Windows runtime, media playback, font-substitution quality and performance claims are NOT_RUN/UNVERIFIED for this task. Task03 manual unplug and GateB remain pending and were not required for these model checks.
