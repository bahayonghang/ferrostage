# Implementation-first acceptance — 2026-10-15

## User decision and precedence

The user explicitly requested committing and archiving completed tasks, providing a prompt for all remaining functionality, and deferring physical unplug/latency, real single/zero-display, other native matrices and size testing until after all functionality is implemented. This policy supersedes earlier blocking language in the parent and child plans and historical execution reports. It changes verification scheduling, not product behavior or recorded test outcomes.

Do not delete existing regression tests, raw evidence or failed receipts. Do not mark deferred checks PASS. The task tree is unchanged. Archive03/07 once their implementation and retained functional checks are independently confirmed; this denotes completion under this revised scope, not full hardware qualification.01/02/04/05/06 are already archived.08–11 remain implementation work, including every P1/P2 feature.

## Checks that remain required

- Before implementing: review actual dependency interfaces and planning artifacts; use task.py start on the existing task. Preserve others' edits and dispatch serial implement then check agents.
- Bug fixes retain meaningful reproducing regressions. Run affected tests, strict lint/typecheck, required feature compilation, cargo deny and just ci after meaningful code changes; reuse prior results only with source applicability evidence.
- Keep core purity, read-only user PDFs, single PDFium thread, settings atomicity/future-version handling, snapshot-only Audience and private notes/media paths. No license, advisory, unsafe or architectural waiver is granted.
- Verify new functionality through automated/integration checks and bounded real smoke tests where the feature requires native components: real nonempty PDF/notes rendering and isolation, actual settings effects, reload commit/abort/retire, actual GIF/video decode and missing-DLL startup. Mocks cannot prove native functionality. Do not launch a full hardware matrix or long benchmark under the name of a smoke test.
- A concrete functional regression, missing dependency API, unsafe loader or license incompatibility remains blocking for the affected work. Continue other dependency-ready work; do not use placeholders to claim implementation.

## Deferred, non-blocking verification backlog

These are DEFERRED_BY_USER / NOT_RUN unless an earlier version-scoped result exists. The old thresholds and protocols are retained for a later validation phase; do not execute them until that phase is requested.

| Verification | Owner / later phase |
| --- | --- |
| Physical unplug/replug/port changes, identity stability across hardware, measured unplug/migration latency, same-model/missing-EDID devices |03/07/09; later hardware acceptance|
| True single-display and zero-display scenarios, effective-DPI/rotation changes |03/07/09; later hardware acceptance|
| Exhaustive theme/size/DPI/font/focus/cursor/hover/disabled/loading matrices, transient Curtain captures, cross-window drag/drop matrix, live OS-theme matrix |07/08/09; later native UX acceptance|
| Full80-page/30-minute M1 presentation, repeated startup/Start sampling,60-second pointer/grid benchmarks,4K alignment and dedicated-VRAM measurements |08; later performance/soak acceptance|
| Full1080p60/15-minute media PoC, frame pacing/drop-rate/VRAM and audio-stop/background-exit timing thresholds |11; later performance/soak acceptance|

Functional support for hotplug, single/zero displays, drag/drop, themes, layout and media is **not removed**. Keep existing automated coverage and implement all planned consumers. A bug observed incidentally must still be repaired; deferral is not permission to disregard known functional failures.

## Revised gates

- **Gate A — retained contract checks:** unchanged, technical PASS supported by current source-bound tests.
- **Gate B — implementation gate:**07 shell, role selection/Identify, lifecycle/actions, automated safety paths, product build/policy and bounded core-native evidence. Current V16 evidence satisfies this revised gate; B-implementation PASS. The original full hardware/UX gate remains DEFERRED, not retrospectively passed.08 is unblocked.
- **Gate C — functional workbench gate:**08 P0 real current/next/notes rendering, controlled Audience pixels, navigation/timer/blank/jump and functional isolation, plus affected automatic/feature checks and bounded native smoke. Long-run/performance/matrix work above does not block09–11.08 P1 remains required before all functionality is declared complete.
- **Gate D — functional integration gate:**08 P1 plus09 complete settings consumers,10 reload transaction,11 GIF/video/native-dependency behavior; cross-task privacy, read-only, generation/resource retirement and final build/test/policy checks. Deferred matrices remain a separate ledger and do not block implementation completion.

Parent AC-1–AC-10 retain their full original wording as later qualification targets. At closeout, report each implemented functional subset separately from its deferred physical/performance/visual subset; do not tick a combined full-qualification criterion because only its functional subset passed. "All functionality implemented and checked; extended acceptance deferred by user" is distinct from "all original acceptance passed".

## Submission boundary

This user turn authorizes commits and archival of the completed03/07 work, including the reviewed V16 repair, evidence and this scheduling revision. It does not request a push. The follow-up prompt defaults to leaving new08–11 changes uncommitted and unarchived until a later explicit closeout request. No new task tree is needed.
