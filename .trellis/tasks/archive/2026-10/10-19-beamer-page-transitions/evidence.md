# Implementation receipt, 2026-10-19

## Implemented boundary

- Core owns pure `PageTransition`, `PagePresentation`, `AudiencePlayback` and
  `PlaybackIdentity` types. The projection remains notes-free.
- PDF structure parsing resolves indirect transition dictionaries/parameters,
  implements all 12 styles and PDF defaults, separates `/Dur`, and reports invalid
  metadata through existing structure diagnostics.
- `LoadedDocument::page_presentation` gates lookup through actual content mapping,
  excluding independent notes and preserving overlay physical identity.
- Audience composition prepares incoming images before starting its monotonic
  effect clock. GPU image bounds/clips implement geometric and motion effects;
  GPUI element opacity implements Fade. Dissolve and Glitter use a stable bounded
  spatial grid. Fly prepares one changed-pixel image, with optional opaque changed
  bounding rectangle, then translates/scales it over the stationary base.
- Old/new/Fly/staging/media allocations use the same texture cache budget.
  Failure retains the last valid public image; no false destination-visible event
  is emitted after failed upload. The final successful frame frees old/difference
  images. No per-tick PDF raster or full CPU image copy occurs.
- Initially added 43-page deterministic synthetic fixtures, fast and 2-second native
  versions. They contain no private thesis content. `docs/pdf-transitions.md`
  describes semantic compatibility and explicit fallback boundaries.

## Shared integration contract

Shell owns `navigation: u64`, allocated for accepted navigation; initial Start,
Jump, reload, replacement and generation change publish `transition: None`.
Sequential commands select the destination's metadata. Keep the pending identity
private until assembled generation/page/crop and exact Audience target match the
destination; while the assembler holds old pixels retain their old playback.
Never attach incoming metadata to an old retained raster.

`Audience` implements `EventEmitter<PlaybackEvent>`, where the event contains
`identity: PlaybackIdentity`, `phase: FirstVisible | Settled`, `at: Instant`.
FirstVisible means the first successfully painted effect frame, including its
progress-zero outgoing endpoint, after incoming atlas upload. It is not a claim
that the first incoming pixel has nonzero opacity, or that native Present has
completed. Settled means the completed destination was painted. Immediate paths
emit both once. Timer zero-dwell must wait for Settled. Both emission and Shell
consumption must fence the current identity; Shell also fences source window.

Same-identity `transition: None` cancels an active effect and settles the incoming
image, or retains the old fallback if incoming upload failed. Pointer/drawing,
blanking, Curtain, staging and media cancel; Shell handles timing cancellation and
resume policy. Repeated publication does not restart. Latest navigation replaces
the prior effect without queued playback. Frame callbacks only run while animating
plus a final cleanup draw.

Shell wiring and timed-navigation scheduler are assigned to the overlay/timing
worker. That worker also owns the final `/Dur` representability check and bounded
active-content diagnostics; these are not counted as completed in this receipt.

## Verification

- PASS: `cargo test -p ferrostage-core -p ferrostage-pdf -p ferrostage-document -p ferrostage-ui --lib --offline --locked`, 97 tests before the final pure reveal-variant test addition.
- PASS: `cargo test -p ferrostage-ui --features test-support --lib audience::playback --offline --locked`, 2 tests. Actual GPUI scene/atlas path covers every non-Replace family, Fade opacity, spatial masks, frame reuse, exactly-once visibility/settlement, final resource retirement, cancellation, budget fallback and failed atlas insertion retaining old public pixels without a false visible event.
- PASS: default strict Clippy for core/PDF/document/UI all targets.
- PASS: native-feature type check. Native strict Clippy had only unused event
  fields pending the Shell subscriber after local lint fixes; no waiver added.
- NOT_RUN here: native Windows recordings, real GPU output equivalence,
  dual-display cancellation and original thesis inventory replay. Parent owns
  integrated native acceptance. Headless scene assertions are not visual acceptance.

## Owned files

`core/src/transition.rs`, core lib/snapshot/state initializers;
`pdf/src/structure/presentation.rs` and structure hook;
document builder/content metadata and tests;
UI audience/playback modules, transition geometry, texture-cache transfer/access;
UI lib module registration and slide_surface/workbench/shell test initializer
defaults only; fixture generator/PDFs and compatibility documentation.

No dependency, package version, installation, Git commit or private PDF edits.

## Independent review repairs

The parent-authorized reviewer subsequently reproduced and repaired enlarged Fly
endpoint clipping and lost deferred playback acknowledgements during a same-identity
blank/unblank. Eight `/SS 3` cardinal/inward/outward cases were appended to both
fixtures, preserving the original page order; the current fixtures contain 51 pages.
See `../10-19-presenter-defense-experience/research/review-stage2.md` for red/green
regressions and strict native-feature Clippy results. Native recordings remain
parent-owned; these automated checks do not complete that acceptance.
