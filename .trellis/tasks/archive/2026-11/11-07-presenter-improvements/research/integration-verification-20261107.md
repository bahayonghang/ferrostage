# Presenter improvements integration verification

Status: complete for all six approved child tasks. Each child has its own implementation, directly affected regressions and scoped native evidence. The later suggestions in item-review.md remain explicitly deferred; they are not unfinished members of this task tree.

## Completed child evidence

- Performance and airy-glass appearance were already committed and archived before this continuation. Their original scoped native receipts remain the owning evidence; this session does not repeat their benchmarks or material/platform matrices.
- Crop confirmation: feature commit `968c261`, archive `6e5587f`. Final recheck passed 243 GPUI tests, 249 workspace tests, 62 native PDFium tests and cargo deny. Original native cancellation/window-creation and private-pixel evidence retained.
- Notes-free layout: feature commit `5dfb995`, archive `742d0ae`. Final checks passed 250 GPUI tests and 249 workspace tests plus cargo deny. Native two previews, divider/reset, end placeholder, mixed-note stability and final narrow visible-layout reset observed.
- Projection status: feature commit `1c2f3d5`; final checks passed 257 GPUI tests, 249 workspace tests and cargo deny. Native named status, selected-versus-settled three-second transition, actual display mapping, covers/restore, paused timer and compiled Beamer step2 observed. An actual deferred-delivery/blank race was reproduced and fixed.
- Return position: final checks passed **269 GPUI tests**, **251 workspace tests**, **55 settings tests**, strict Clippy, fmt, cargo deny and **9 directly affected native-PDFium UI reload tests**. Real compiled-Beamer A-step2→B→C→return, cover readiness, paused timer/autoplay/tool retention, Stop/Resume, setting-editor rebind and document-replacement lifetime were observed. The initial native run found footer clipping; a two-line wrap/max-width fix and a real GPUI regression resolved it, followed by final-source native retesting.

## Final integration checklist

- [x] Return from A step2 after B→C; consume only after correct output; cover/stop/resume/reload and keybinding paths.
- [x] Final-version same-page crop Start→Escape; no audience-window creation. Fresh per-process successful-creation logs contain **zero** audience creations, cross-checked against a one-presenter native window inventory. See `native/final-crop-no-audience.json`, `final-crop-gate.png`, `final-crop-cancelled.png` and `final-crop-stdout.zip` (raw log member).
- [x] Final product checks and directly affected native-PDFium UI reload regressions. `just ci` then `just check-shell` passed after the final source change. Original gate logs are copied into the Return task's `research/checks/`; no source changed afterward.
- [x] Review scoped coverage and complete each task checklist. Archive link/manifest reconciliation is part of the final metadata commit and is verified against the resulting archive tree.

## Final binary and fixture provenance

Final GPUI/PDFium executable SHA256: `09fa7c5a1297e6d2b487f892d3ff63fe7562b2c7c7987c1c8a9cef0d89c4fd6b`. The final integration crop-cancel scenario uses checked-in compiled Beamer `fixtures/beamer/notes-right.pdf`, SHA256 `a0b857f6fd1009db87b17317139c603518af2ad27a0be8ee9b67d009e29a94a7`. Its source PageLabels intentionally expose the existing grouping limitation; the crop gate does not claim to repair that exporter mapping.

Return acceptance uses compiled `fixtures/beamer-timing/overlays-timing.pdf`, SHA256 `8e40489ef4fe594585b19e64c20fd22e958aeb94b801e4b5a7ad7016c59dedd9`. Only an owned copy under `runs/return-native` was externally corrupted/restored for hot-reload checks; both files finish with the original hash. Ordinary, mixed-geometry and independent-note layout checks use repository synthetic fixtures. The four-direction crop privacy checks use generated colored fixtures, supplemented by real compiled Beamer crop previews. Performance uses the owning task's fixed 100-page Beamer A/B sample. These fixture categories are not an exporter-compatibility matrix.

## Evidence owners and limits

The archived child directories retain their own verification documents, native screenshots/data and test receipts. Final Return visual review was independent and returned **ship**. Its mechanical scanner reports zero failures but zero scanned files because Rust is unsupported; screenshots and source review supply that evidence.

Actual Settings recording/save/readback and runtime rebind passed with **Ctrl+R**, and the default **Alt+Left** passed. Sky-injected **Alt+R** did not record in two attempts; its native input cause remains unlocated after source tracing. It is recorded as an unresolved observation, not a passed Alt-letter check or a proven platform/tool defect. No maintained dependency was changed to mask it.

No physical projector receipt, physical cable hotplug or peripheral display matrix is implied by these application-level checks.

## Archived evidence links

- [Crop verification](../../11-07-notes-crop-confirm/research/verification.md).
- [Projection-status verification](../../11-07-projection-status/research/verification.md).
- [Exact-return verification](../../11-07-return-position/research/verification.md), feature `85ba283`, child archive `36799e0`.
- [Notes-free layout verification](../../11-07-notes-free-layout/research/verification.md).
- [Airy-glass verification](../../11-07-presenter-airy-glass/research/verification.md).
- [Performance A/B results](../../11-07-presenter-performance/research/native-results.md).

Removed only the two empty black/white UIA captures in the projection task; real screenshots and all non-empty receipts remain. The task links and JSONL context use their final archive locations.
