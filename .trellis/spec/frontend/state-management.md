# State Management

There is one presenter state and one audience projection.

| State | Owner | Who may read it |
|-------|--------|-----------------|
| `PresentationState` | `ferrostage-core` | Presenter commands via `apply_at(command, Instant)`; `elapsed_at` is pure and core never samples the clock |
| `AudienceProjectionSnapshot` | Core metadata from `audience_snapshot`, matching visual added by the UI assembler | Audience window only while `Presenting`; the presenter may also read it. Includes `stage: AudienceStage`. `Idle` / `Ready` snapshots are `Curtain` and carry no visual |
| `SlideNotes` | `ferrostage-document` | Presenter window only |
| `Settings` | `ferrostage-settings` | App and presenter-side consumers. Successful runtime edits are saved; failed runtime application rolls back before save. A failed save keeps the runtime value and shows unsaved. A future `schema_version` is read-only and is not reset or written back |
| `MonitorId` + `MonitorRole` | `display.monitors` | The only role authority. Not monitor indexes |

`SwapScreens` is an intent to the app role coordinator. One settings transaction exchanges the two `MonitorRole` bindings and moves the windows. It does not flip an authoritative swap flag, and it does not change `current_page` or `slide_crop`. Core has no swap flag. Direct `apply_at(SwapScreens, now)` returns `ScreenSwapRequiresCoordinator` without changing state; the app handles this intent before core dispatch.

Core `AudienceProjectionSnapshot` includes `stage` and `visual`; its metadata projection always sets `visual=None`. Tasks 08/11 supply matching pixels. Task 02 supplies the sectioned settings schema and `SettingsStore`; task 07 owns runtime display assignment and its single save-status/Retry owner, while task 09 integrates the shared panel into full settings. The audience window is absent outside `Presenting`. A saved Audience that is missing does not fall back to another monitor (see the [frontend quality guidelines](./quality-guidelines.md)).

The audience view receives only the assembled snapshot, never the document, notes, media sources, a full-cache handle, or an arbitrary-page request API. The assembler publishes matching generation/page/crop/target pixels together; it keeps the previous complete projection until the next one is ready. `Curtain` and black/white blank cover every visual layer immediately.

Blank mode is `Option<BlankMode>` on the snapshot so the audience window does not invent a second blank flag.

## Shared display assignment contract

### 1. Scope / Trigger

Task07 G-13/I-8 owns the presenter-side panel and Identify. Task08 reuses its entry with real PDF content; task09 embeds the same panel in full settings. Read the parent `research/display-assignment-plan.md` for the approved acceptance matrix. A native pass must exercise the new controls; historical Swap/config-only evidence is insufficient.

### 2. Signatures

Persist existing `DisplaySettings { monitors: Vec<MonitorBinding>, single_monitor_mode: SingleMonitorMode, .. }`. Binding identity is `MonitorId` with `MonitorRole`; mode is `PresenterOnly` or `WindowedAudience`. No numbered-display settings schema or independent role resolver is introduced. Both fields are committed through the existing Shell transaction and `SettingsStore::save_session`.

The shared UI module is `display_settings`: `DisplayDraft { revision, presenter: Option<MonitorId>, pdf: PdfTarget }`, with `PdfTarget::{None, Windowed, Display(MonitorId)}`. `numbered(&[MonitorInfo])` is the sole list/Identify ordering; `DisplayDraft::settings` validates a current revision/topology before producing the complete `DisplaySettings` edit. `Shell::assign_displays` handles both explicit Apply and the compatible Swap intent.

### 3. Contracts

- Opening/selecting changes only a presenter-owned draft. Cancel leaves runtime windows and persisted settings unchanged. Apply revalidates the current complete topology and selected MonitorIds before preparing replacement windows.
- List and Identify share one transient number mapping, stable for the same complete topology. These labels are not asserted to equal Windows Settings numbers and are never persisted as identity.
- No-play removes Audience and selects `PresenterOnly`. Explicit single-screen windowed playback selects `WindowedAudience` with only a Presenter physical binding. A physical PDF target selects Audience plus `PresenterOnly`; multiple-screen destinations must differ.
- Valid-target migration preserves committed page, stage, blank mode and timer. Explicit no-play during Presenting follows Curtain, Stop, audience close; retain the page and freeze elapsed time in Ready.
- Runtime failure rolls back before saving. Save failure keeps the confirmed runtime value and exposes one unsaved/Retry state. Future-schema settings are read-only.
- Windowed placement confirms the observed client size and containment within the target display's usable bounds, with room for window decoration, in addition to display identity/mode. Mixed-DPI creation can rescale an initial request; correct it through the existing GPUI resize path after the target DPI settles, then observe again. An oversized or off-screen replacement cannot commit merely because its display ID matches; retain the existing bounded timeout/rollback. Fullscreen audience confirmation continues to use full display bounds.
- A resize request is not proof of the resulting client size. The pinned Windows path may apply stale native-frame offsets after a DPI change. After a genuinely changed bounds observation, permit at most one additional correction using the prior request plus the observed client delta, bounded to32 logical pixels per axis. Ignore an unchanged pre-resize observation; never loop, enlarge the2px acceptance tolerance or commit an unconfirmed result. No response or a larger discrepancy follows the existing timeout/rollback.
- Identify contains only number/name, expires after three seconds and is disabled while Presenting. It is not an audience view and cannot receive document/notes/snapshot data. A pending transition must not let Identify survive into Presenting.

### 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Topology changes while draft is open | Refresh list, invalidate old draft; reject stale Apply |
| Enumeration unavailable | Show unavailable, never a fabricated zero-display success |
| Successful zero-display enumeration | Show zero and refuse placement |
| Offline target or duplicate multi-screen destinations | Explain rejection; no runtime/config mutation |
| Replacement placement fails | Close replacements and restore prior complete state; no save |
| Target ID matches but windowed bounds overflow or size is still DPI-scaled | Do not commit; await valid observed placement or roll back on timeout |
| Save fails after placement | Retain effective roles/mode; one warning with Retry |
| Future schema | Display state without modifying runtime settings or disk |

### 5. Good / Base / Bad Cases

Good: B application/C PDF Apply confirms both windows, then saves MonitorIds. Base: open and cancel keeps existing roles and file bytes. Bad: selected row index becomes a saved role, or clearing Audience leaves an inherited `WindowedAudience` mode that silently authorizes playback.

### 6. Tests Required

Exercise real GPUI input/focus and the shared coordinator for draft cancellation, conflicts, unavailable/stale topology, read-only settings, runtime rollback and persistence retry. Assert page/stage/timer invariants for valid migration and explicit no-play. Native evidence separately checks numbered physical overlays, A/C PDF destinations, actual placement, config read-back and restart. Simulations do not satisfy physical single/zero-display or plug/latency gates.

Retain the native V9 F9 regression: on a1066.67-wide logical portrait display, a1280-wide client is invalid even when the display ID matches. Cover mixed-DPI target-size correction and rejection/rollback of unconfirmed geometry; native retest must show the complete titlebar and controls plus event-reported logical client bounds. Screenshot outer dimensions alone do not prove a DESIGN.md size case.

Retain native V10 F10: request1002.67x760 at scale1.5 yielded1000x754 after the first resize. Exercise the actual GPUI coordinator with a bounded Windows resize-response model, delayed/unchanged observations and nonconvergence; an exact TestPlatform resize alone cannot establish this native boundary. Real Windows success still requires a new native receipt.

### 7. Wrong vs Correct

Wrong: each settings entry builds its own numbering, Swap flag and window migration path. Correct:07 owns a shared draft surface, numbering and existing coordinator transaction;08/09 reuse it, and configured Swap remains a compatibility intent to the same owner.

## Forbidden

- A global `lazy_static` deck.
- Cloning speaker notes into the snapshot "just for the audience highlighter".
- Two copies of the current page index, one in the view and one in the core. The view sends a command and renders the snapshot it gets back.
