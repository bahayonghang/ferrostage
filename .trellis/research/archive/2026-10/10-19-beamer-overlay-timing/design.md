# Overlay mapping and page-duration scheduler

## Existing path and metadata
`ferrostage-document/src/builder.rs` already groups compatible repeated content labels and maps `notes:` pages; `ferrostage-core/src/slide_map.rs` exposes next_step. Exercise this path using generated Beamer fixtures, including no-label/ambiguous-label cases and inserted independent notes. Correct proven gaps only. LaTeX commands are not parsed at runtime: the app consumes the compiled PDF.

Add optional finite nonnegative dwell duration from page /Dur beside transition metadata. Missing is distinct from zero. Do not overload slide transition /D or the user's elapsed talk timer. Duration is tied to physical content step, not logical slide-group index. Reject invalid/unrepresentable values with manual fallback and a diagnostic.

## Scheduler contract
One pending action keyed by document generation, physical content step, navigation identity and audience window. The transition child reports identity-fenced first-visible and settled events. With positive /Dur, the deadline is successful settlement time plus authored duration. PDF Reference 1.4 Figure 8.1 (printed page 487, https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.4.pdf) places display duration after the incoming transition; this corrects the initial first-visible timing assumption. Pausing during a transition retains the full dwell until settlement and resume. Without a transition, settled coincides with first visible. With /Dur=0, dispatch on a later UI/event-loop opportunity after first paint and settlement, never recursively in render or before a real displayed frame.

The next action uses existing next-step navigation to skip notes while preserving overlays. If raster/texture upload is late, wait for its own visible evidence; do not skip intermediate compiled frames or queue a backlog. Prefetch bounded upcoming content using the existing worker/priorities and budget. Stop at missing /Dur or the final content step.

## User control and lifecycle
Show one contextual auto-play Pause/Resume action only when the document contains authored timing; no speed editor or global animation settings. The existing talk timer controls remain independent. Pause stores remaining duration and invalidates the scheduled callback. Resume re-arms from the same public page; for zero duration it schedules only the next eligible rendered step.

Manual Next/Previous/Jump cancels the previous callback, chooses one destination, and arms from its successfully painted settlement if automatic playback is not explicitly paused. Modal opening or presenter focus loss suspends elapsed dwell; closing/refocusing resumes the remainder, not missed deadlines. Blackout suspends and unblank resumes the latest visible destination with remaining dwell; navigation while blank starts a fresh destination dwell on unblank. Drawing/pointer use settles transition and pauses automatic playback until explicit Resume, allowing annotation without surprise navigation. Stop/reload/topology/window replacement cancel the epoch; recovery never fires a retired callback. Reload requires a fresh presentation snapshot/first paint before scheduling.

## Unsupported active content
Use bounded PDF structure inspection for recognized JavaScript/actions and optional-content animation patterns, add a document diagnostic through existing warning surfaces. Do not execute scripts, disable existing supported media, dump script/note bytes, or call all optional-content PDFs animated. Static display continues; release compatibility docs distinguish known detection from complete interpretation.

## Validation
Fake-clock tests trace event order and epoch invalidation; integration tests exercise actual next_step and note mapping. Native fixtures prove at least one visible paint per zero-duration step, no notes, pause responsiveness and final frame stability. Throughput is bounded by rendering, not a promise of arbitrary FPS. Revert scheduler independently while ordinary manual stepping remains functional.
