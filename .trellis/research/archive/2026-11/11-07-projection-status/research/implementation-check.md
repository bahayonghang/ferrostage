# Projection status implementation

## Boundary

The presenter owns a status projection and the last accepted audience Settled receipt. No core state, audience snapshot, settings schema, monitor allocation or PDF bytes were changed. The existing stage capsule and timer keep their roles; a status row beneath the command bar shows drawn/pending content steps, configured versus observed output location, blank state and an explicit Restore action. The location control opens the existing Displays entry.

## Receipt and readiness contracts

- `settled_playback()` returns the last accepted Settled identity only for the current Presenting audience WindowId and document generation. FirstVisible and snapshot publication do not update it. Pending navigation and failed destination output preserve the previous complete receipt.
- `selected_playback()` returns the current navigation identity only when stage, generation, page, crop and audience source still match.
- `public_frame_ready()` means exact current Audience-priority raster pixels with matching target size, without a placement transaction, loading, graphics error or reload/recovery block (including failed_worker). It remains valid under a covering blank. It is not a visible-paint receipt. The authored-playback scheduler uses this existing readiness boundary.
- Audience commits phase delivery only while its current snapshot is Live, unblanked and identity-matched. Shell retains the existing source/generation/navigation/session/placement/reload/snapshot-identity checks. A subsequently arriving blank cannot discard an already delivered historical paint; the independent review added a real-event race/liveness regression for this boundary.
- Restore toggles off the current blank through existing core commands and does not change page, timer pause, authored-playback pause or private notes.

Location uses the current `observed_display` → unique MonitorId mapping, never requested WindowRecord.monitor. Missing readback/topology stays explicitly unknown. Ready labels the configured target separately; a Start transaction labels preparing. Same-screen fullscreen/windowed mode is explicit. The audience receives none of this presenter UI.

## Validation

The full UI-library command `cargo test -p ferrostage-ui --features test-support --lib --locked` passed **256 tests**, including six new status/readiness regressions. Log: `runs/projection-full-ui.log`.

New coverage: ordinary page actual audience draw, FirstVisible versus Settled, pending/failed targets retaining the old receipt, stale navigation/window/generation, real Restore button for both blanks with timer/pause and public metadata preservation, configured versus unknown observed location, preparing state, and exact readiness rejection for recovery/loading/placement/graphics failure. The existing suite covers authored transitions and topology safe-stop paths. Product build/Clippy, final CI and native acceptance are owned by the independent checker/main session after this handoff.

The failure prefix deliberately says Output problem rather than claiming pixels were retained: RenderStatus can also report a transition fallback that successfully drew the destination. Drawn and pending identities state what is actually known.
