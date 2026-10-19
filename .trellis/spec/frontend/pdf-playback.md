# PDF transitions and authored playback

## 1. Scope / Trigger

Applies to `/Trans` and `/Dur` parsing, compiled Beamer overlays, public raster
publication, Audience composition/acknowledgements and automatic navigation.
The talk timer is independent. Original PDF Notes remain private rasters.

## 2. Signatures

- Core `PagePresentation { transition: Option<PageTransition>, dwell_seconds: Option<f64> }`
  contains owned parser-independent data. `PlaybackIdentity { generation, navigation, page }`
  identifies one selected physical content step; `AudiencePlayback` carries that identity
  and an explicitly authorized optional transition.
- `LoadedDocument::page_presentation(PhysicalPage)` returns metadata only for mapped
  content pages. Independent notes return `None`; navigation uses `SlideMap::next_step`.
- `Shell::{select_playback, sync_playback, attach_playback, playback_event, poll_timed_playback}`
  bridge committed state, exact public pixels, source window and the pure `Autoplay` clock.
- Audience emits `PlaybackEvent { identity, phase: FirstVisible | Settled, at: Instant }`.
  `Autoplay::{first_visible, settled}(identity, at)` accept matching events;
  `next_turn(now)` returns at most one eligible identity, never a backlog.

## 3. Contracts

Parse indirect transition dictionaries/fields with PDF defaults: absent `S` is Replace,
`D` is one second, `Dm` is Horizontal, `M` is Inward, `Di` is Right, `SS` is 1 and `B`
is false. Apply parameters to their relevant styles. Invalid/unknown supported metadata
uses a bounded structural diagnostic and immediate/manual fallback. `/Dur` is separate
from `/Trans /D`: missing is manual, zero is valid, and accepted dwell must be finite,
nonnegative and representable as `std::time::Duration`.

Sequential Next/Previous authorize the destination's transition. Start, direct Jump,
reload, document change and audience replacement use immediate display. Allocate a new
navigation identity for a selected destination, but attach it only when generation,
physical page, crop, Audience priority and current target pixels match the assembled
public image. While rasterization is pending, retained old pixels retain their old
identity. The scheduler rejects their events for the newly selected destination.

Audience starts an effect only after successful incoming atlas upload. FirstVisible
acknowledges a successfully painted effect frame (which may be its outgoing zero-progress
endpoint), not native Present completion. Settled acknowledges the completed destination
paint. Commit delivered-phase flags inside the deferred callback after verifying current
Live/unblanked snapshot and playback identity. A blank may discard a queued event while
retaining the identity; such a discarded event must not consume its future acknowledgement.
Shell additionally checks the current source audience window, active generation/navigation,
session stage and reload/placement gates before accepting it.

Positive dwell begins at `Settled.at`, after the incoming transition, as in PDF Reference
Figure 8.1. Zero dwell still requires FirstVisible and Settled, then a later Shell poll
turn. No recursive navigation from paint, skipping unpainted steps, catch-up queue or
final-step wrap. Manual navigation invalidates the prior dwell and preserves explicit
pause. Temporary focus/modal/contextual-menu/blank/render-error suspension freezes remaining
dwell; resume continues that remainder. Pointer/drawing settles the effect and explicitly
pauses until Resume. The single automatic-playback action exists only for timed documents
and does not control the talk timer.

Stop, generation/window changes and reload invalidate playback. Reload blocking includes
awaiting stable bytes, structure parsing, pending candidate work, an unattempted replacement
and the `failed_worker` recovery latch. A failed rollback remains blocked after the pending
transaction disappears: retained old pixels are not authorization to resume timed navigation.
Recovery requires a fresh identity and successful paint acknowledgements.

All 12 standard styles have separate composition paths. Split/Blinds use authored line
orientation; Box uses centered edge/center reveal; Push/Cover/Uncover preserve their distinct
moving and stationary layers. Dissolve/Glitter use bounded deterministic spatial cells.
Fly prepares changed pixels once; `B` fills their bounding rectangle. Centered scale greater
than one requires scale-aware travel to keep directional endpoints offscreen; `/Di /None`
remains scale-only. Test just inside endpoints because exact-endpoint replacement hides pops.

Old/new/Fly images share the existing texture cache and budget with staging/media. Do not
rerasterize PDFs or copy full images each tick. Failed destination upload retains the last
valid public pixels and must not acknowledge the destination. Blanking/Curtain always cover
all layers. Media and drawing cancel effects. Native visual acceptance remains separate
from geometry/scene/atlas tests.

Known JavaScript/SetOCGState action requirements use existing warning UI without executing
scripts or logging their contents. Action inspection is bounded (128 nodes, depth 8,
32 additional-action entries, 256 annotations per page); static optional-content layers
alone are not classified as animations. The separate `animate` package runtime is outside
support; built-in Beamer animate/animatevalue are ordinary compiled page/timing sequences.

## 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Destination raster missing or wrong target/crop/priority | Keep previous public image/identity; no destination dwell |
| Incoming upload fails | Retain old image; no false FirstVisible/Settled |
| Same-identity blank before deferred delivery | Drop callback without consuming the phase; acknowledge after visible repaint |
| Old window/generation/navigation callback | Reject before clock mutation |
| Positive `/Dur` plus transition | Full authored dwell after settlement |
| Zero `/Dur` | Paint and settle every step; advance on a later poll only |
| Pause then manual navigation | New target selected, explicit pause retained |
| Modal/focus/blank suspension | Preserve remaining dwell without catch-up |
| Reload rollback failure, pending transaction gone | Playback remains cancelled until recovery latch clears |
| Enlarged Fly, cardinal direction | Changes approach/leave a fully offscreen endpoint without popping |

## 5. Good / Base / Bad Cases

Good: a new page waits for its exact image, paints its transition, settles, then waits the
full authored dwell. Base: absent timing stays manual and does not schedule idle effects.
Bad: tag old retained pixels with a new page identity, count transition time as dwell,
consume an undelivered phase, or resume because a failed reload's transaction was removed.

## 6. Tests Required

Retain `transitions::tests`, actual `audience::playback::tests` scene/atlas coverage,
`autoplay::tests`, Shell `timed_*` regressions and reload playback tests. Assert exact
destination matching, note exclusion, source-window rejection, no recursive zero advance,
positive post-settlement dwell, pause/remainder preservation, deferred blank race and
failed-worker latch recovery. `ferrostage-document/tests/overlay_timing.rs` uses the compiled
fixture with a label oracle; its PDFium feature additionally verifies real labels and
distinct content/notes rasters. Normal tests must not require a TeX installation.

Run native all-style recordings, dual-display interruption and source-deck checks separately.
Record NOT_RUN honestly when native evidence is unavailable; do not infer standing readability
or real GPU presentation from passing headless tests.

## 7. Wrong vs Correct

Wrong: `remaining = Dur - transition_elapsed`, flag a phase before deferred eligibility,
or let a retained image restart scheduling while `reload.failed_worker` remains true.
Correct: arm from successful `Settled.at`, commit phase flags on eligible delivery, and
require the same recovery/identity/paint boundary before every new timed navigation.
