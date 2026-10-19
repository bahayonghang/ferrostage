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

Blank mode is `Option<BlankMode>` on the snapshot so the audience window does not invent a second blank flag. [PDF playback](./pdf-playback.md) defines transition metadata, exact-image identities, delivered paint acknowledgements and the separate authored-dwell scheduler.

## Manual navigation and original Notes workbench

### 1. Scope / Trigger

Direct page selection, presentation timers, pane composition and thumbnail scheduling share one committed core page. The notes-first task removes staged selection preview, including hover-triggered preview changes.

### 2. Signatures

Use `PresentationCommand::JumpTo(PhysicalPage)` through `PresentationState::apply_at`. `PresenterSettings::effective_notes_ratio()` normalizes legacy small notes shares. `show_thumbnail_strip` remains a read-compatible schema field with no rendering effect.

### 3. Contracts

Import enters Ready with no audience or running presentation timer. Start is explicit. Talk-timer reads and expiry never navigate; authored PDF dwell follows the separate PDF playback contract. Grid clicks commit a direct jump; hover is inert. Page-number editing is local to its modal and submits only on Enter/Go. Committed-page changes use the existing media/annotation invalidation and exact-key raster path. No second authoritative preview page exists.

Current slide stays left; Next occupies the upper-right pane and original Notes the lower-right pane. Fresh `next_slide_ratio` is 0.40 (right-column width); stored ratios remain authoritative. Default `notes_ratio` is 0.65, with effective range 0.55 to 0.8 for old settings and divider interactions. Vertical drag measures the complement of the upper pane after subtracting the 6px divider and adjacent 4px gaps: dragging down reduces Notes height. Never invert stored ratios on load. Each ordered Notes raster fills the inner pane width at its own aspect ratio, does not flex-shrink, and shares the existing vertical scroll. No text extraction/reflow or Notes-mode/font selector is introduced. Recent files appear only in Idle. Only an explicitly open grid schedules thumbnails; closing it retires obsolete requested/retained thumbnail frames. Keep the existing configured grid page bound.

### 4. Validation & Error Matrix

| Input | Result |
| --- | --- |
| Valid content page | Set physical/logical page together, preserve session stage and timer |
| Out-of-range or notes-only physical page | Typed error, no partial navigation |
| Hover, talk-timer elapsed time, partial input, invalid dialog submit, Escape | No page change |
| Legacy thumbnail strip enabled, grid closed | Zero thumbnail requests |
| Legacy small notes ratio | Readable effective share without rewriting the settings file on load |
| Divider dragged downward | Next grows, Notes shrinks; persisted ratio restores those sizes |
| Multiple Notes rasters | Ordered natural-height pages in one scroll; no equal-height compression |

### 5. Good / Base / Bad Cases

Good: clicking a grid page immediately navigates and its current notes follow. Base: opening a PDF shows Ready while the clock updates. Bad: moving the pointer over a thumbnail changes content, or talk-timer expiry dispatches Next.

### 6. Tests Required

Exercise actual GPUI mouse/key dispatch, partial versus submitted page input, modal cancellation and focus restoration. Assert Ready/Presenting page stability across ticks and overtime. Check real notes crop identity/privacy and pane ordering, closed/open/closed grid work, stale-frame retirement and duplicate-submission prevention. Native screenshot evidence remains separate from source or headless checks.

Retain actual divider-input/settings-readback and multiple-Notes aspect/height regressions in `shell/presenter_layout_tests.rs`. PDF font measurements and raster geometry cannot establish standing readability; test the final original PDF in the integrated native presenter separately.

### 7. Wrong vs Correct

Wrong: hide a thumbnail strip while its persisted flag still schedules raster work, or preserve invisible preview state after removing controls. Correct: gate raster requests on actual grid visibility and route explicit selection through the one committed-page command.

## Start fullscreen and presenter restoration

### 1. Scope / Trigger

Start enters fullscreen on the existing presenter window while the audience is placed independently. Stop, failed Start and cancellation restore the private window without changing startup preferences or monitor bindings.

### 2. Signatures

`Shell::enter_presenter_fullscreen(cx) -> Result<(), String>`, `start_presenter_confirmed(cx) -> bool`, `restore_presenter(cx)`, `poll_presenter_restore(cx)` and `restoring_presenter()` own this lifecycle in `shell/presenter_fullscreen.rs`. `Restore` captures WindowId, MonitorId, display identity, bounds, fullscreen/maximized state, optional deadline and `exit_requested`.

### 3. Contracts

Capture native state before toggling in place; the existing audience transaction commits Start only when both its audience placement and presenter fullscreen readback pass. A manually moved private window whose observed display does not match its mapped role cannot authorize entry. Requested flags alone do not prove placement. Preserve the existing view/page/focus and do not save `display.presenter_start_fullscreen` or display roles as a Start side effect.

Cancellation/Stop retires audience output first and requests restoration once. Restoration remains part of the busy guard. Windows queues fullscreen toggles: when Stop precedes entry, observe fullscreen entry before issuing exactly one exit, then verify the saved fullscreen/maximized state, display identity and normal bounds (2px tolerance). Do not accept the still-windowed pre-entry flag as restored. Starting already fullscreen retains fullscreen on Stop.

Restore applies only to the captured WindowId. Confirmed role migration retires that record instead of applying old bounds to a replacement window. Missing original mapping or restore timeout uses existing safe windowed recovery; timeout reports `PlacementTimeout`. Preserve current-versus-retired window checks and deferred native callbacks.

### 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Requested fullscreen without confirmed native placement | Start stays pending; no running session/timer |
| Audience failure, cancellation or Stop before entry finishes | Retire audience; wait for entry, exit once, verify restore |
| Repeated Stop/restore or already-fullscreen Start | Idempotent restoration, no extra toggle |
| Missing original monitor or timed-out restore | Existing safe recovery and truthful placement error |
| Presenter replaced by confirmed role migration | Discard obsolete restore record |

### 5. Good / Base / Bad Cases

Good: Start and Stop reuse the presenter and restore its maximized state. Base: an initially fullscreen presenter stays fullscreen after Stop. Bad: a queued entry arrives after Stop because the pre-entry windowed flag was mistaken for completed restoration.

### 6. Tests Required

Retain flag-only refusal/timeout, delayed-entry restoration, repeated/already-fullscreen cases in `shell/presenter_layout_tests.rs`. Native dual-display success, normal/maximized restoration and monitor-loss recovery require separate runtime evidence; headless GPUI placement flags do not establish them.

### 7. Wrong vs Correct

Wrong: toggle blindly on every Start/Stop or rewrite persisted startup fullscreen. Correct: capture native identity/state once, commit after readback, and restore through the identity-fenced asynchronous state machine.

## Audience click to advance

### 1. Scope / Trigger

Audience playback accepts completed left clicks, including the letterbox client area. Presenter previews and their pen/laser/media handlers retain their own behavior.

### 2. Signatures

`Audience` emits payload-free `AdvanceRequested`. `Shell` subscribes with the source window handle and routes accepted requests through `dispatch(Intent::Next)`, which applies `PresentationCommand::NextStep`. `display.audience_click_to_advance: bool` defaults to true.

### 3. Contracts

The assembled snapshot remains Audience's only incoming product data. Do not pass Shell, settings, documents or notes into Audience. Handle input after the source window update lease ends; recheck current state before dispatch. Accept only the committed audience handle while Presenting, enabled, without any modal/busy operation, with Live projection and no effective or published black/white blanking. Pending/retired windows cannot navigate. One completed click advances one existing Beamer content step; the last step does not wrap. Mouse-down alone, non-left input and release outside the surface are inert. Do not add a second navigation path, debounce timer or synthetic keyboard click binding.

The effective committed settings are the only enable authority. Changes apply to the existing window, including runtime edits whose disk save failed. Exclude this non-placement leaf from the `continue_preferences` display-layout comparison, as for `presenter_start_fullscreen`; actual placement/fullscreen/monitor changes must still use the display transaction.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Enabled, normal playback, committed audience click | Dispatch Next once through existing page/media/render handling |
| Disabled, blank, Curtain, modal, loading/reload/preferences/window transition | No navigation |
| Pending/replaced/stopped audience input | No navigation |
| Toggle alone | Save/apply immediately without replacing windows or changing page/timer/roles |
| Save failure or future-schema edit | Existing Unsaved/Retry or read-only behavior; no alternate persistence path |

### 5. Good / Base / Bad Cases

Good: turn the setting off and the same audience window stops advancing. Base: old settings missing the optional leaf enable clicks. Bad: dispatch directly from Audience, or rebuild projection windows for an input preference.

### 6. Tests Required

Drive real GPUI audience mouse down/up, including non-left/outside cancellation, guard states, retired-window rejection, Beamer/notes mapping and final-page behavior. Drive the real settings control and store readback with unchanged window handles, plus future/read-only and save failure. Native first-click activation/fullscreen delivery remain separate from headless assertions.

### 7. Wrong vs Correct

Wrong: add settings to the public projection or increment a page index in the view. Correct: outgoing event, coordinator policy, existing Next intent and snapshot publication.

## Shared display assignment contract

### 1. Scope / Trigger

Task07 G-13/I-8 owns the presenter-side panel and Identify. Task08 reuses its entry with real PDF content; task09 embeds the same panel in full settings. Read the parent `research/display-assignment-plan.md` for the approved acceptance matrix. A native pass must exercise the new controls; historical Swap/config-only evidence is insufficient.

### 2. Signatures

Persist existing `DisplaySettings { monitors: Vec<MonitorBinding>, single_monitor_mode: SingleMonitorMode, .. }`. Binding identity is `MonitorId` with `MonitorRole`; mode is `PresenterOnly` or `WindowedAudience`. No numbered-display settings schema or independent role resolver is introduced. Both fields are committed through the existing Shell transaction and `SettingsStore::save_session`.

The shared UI module is `display_settings`: `DisplayDraft { revision, presenter: Option<MonitorId>, pdf: PdfTarget }`, with `PdfTarget::{None, Windowed, Display(MonitorId)}`. `numbered(&[MonitorInfo])` is the sole list/Identify ordering; `DisplayDraft::settings` validates a current revision/topology before producing the complete `DisplaySettings` edit. `Shell::assign_displays` handles both explicit Apply and the compatible Swap intent.

### 3. Contracts

- Opening/selecting changes only a presenter-owned draft. Cancel leaves runtime windows and persisted settings unchanged. Apply revalidates the current complete topology and selected MonitorIds before preparing replacement windows.
- List and Identify share one transient number mapping, stable for the same complete topology. These labels are not asserted to equal Windows Settings numbers and are never persisted as identity. `screen_copy` builds the shared card title, meta line, and Identify caption from the model token, built-in marker, resolution, portrait, primary flag, and relative position. `MonitorInfo.name` is that model when the Windows friendly name has one, otherwise the driver description. Saved `friendly_name` copies `MonitorInfo.name`, not the composed caption. A failed friendly-name lookup leaves the driver description and does not fail enumeration.
- No-play removes Audience and selects `PresenterOnly`. Explicit single-screen windowed playback selects `WindowedAudience` with only a Presenter physical binding. A physical PDF target selects Audience plus `PresenterOnly`; multiple-screen destinations must differ.
- Valid-target migration preserves committed page, stage, blank mode and timer. Explicit no-play during Presenting follows Curtain, Stop, audience close; retain the page and freeze elapsed time in Ready.
- Runtime failure rolls back before saving. Save failure keeps the confirmed runtime value and exposes one unsaved/Retry state. Future-schema settings are read-only.
- Windowed placement confirms the observed client size and containment within the target display's usable bounds, with room for window decoration, in addition to display identity/mode. Mixed-DPI creation can rescale an initial request; correct it through the existing GPUI resize path after the target DPI settles, then observe again. An oversized or off-screen replacement cannot commit merely because its display ID matches; retain the existing bounded timeout/rollback. Fullscreen audience confirmation continues to use full display bounds.
- A resize request is not proof of the resulting client size. The pinned Windows path may apply stale native-frame offsets after a DPI change. After a genuinely changed bounds observation, permit at most one additional correction using the prior request plus the observed client delta, bounded to32 logical pixels per axis. Ignore an unchanged pre-resize observation; never loop, enlarge the2px acceptance tolerance or commit an unconfirmed result. No response or a larger discrepancy follows the existing timeout/rollback.
- Identify shows that shared caption under the number, expires after three seconds, and is disabled while Presenting. It is not an audience view and cannot receive document/notes/snapshot data. A pending transition must not let Identify survive into Presenting.

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

## Display location and allocation feedback

### 1. Scope / Trigger

Use this contract when drawing monitor topology, role summaries or assignment rows, or when observing native window movement. Configuration, a draft selection and actual window placement are separate facts.

### 2. Signatures

Presenter-only `display_settings::WindowLocation` is `NotRunning | Unknown | Monitor(MonitorId)`. `observed_location(running, display, valid, monitors, mapping)` resolves a platform display ID only through the current unique mapping and unique monitor identity. `RoleProjection::new(app, pdf, configured, draft)` owns current observations/configuration/pending state; `markers(id, draft)` supplies both diagram and row labels.

### 3. Contracts

`WindowRecord::monitor/display` remain requested placement, not actual location. Per-window `observed_display` cells are seeded at creation, updated by native bounds events and refreshed by existing Shell window reads. Only currently owned records contribute to the Presenter projection; transaction candidates and retired windows cannot publish actual roles. Shell consumes this data outside render and preserves the deferred input/window-lease boundary. A manual OS move never saves new role bindings.

Ready/Idle has no audience window, so its PDF location is NotRunning and any destination is labeled configured/not presenting. Presenting may truthfully show both windows on one monitor in windowed-audience mode. Unknown mapping/readback remains Unknown, with no guess from requested settings. Draft selection changes pending markers only; Apply uses the existing confirmation transaction and Cancel discards allocation draft. Separately autosaved ordinary preferences are not undone by allocation Cancel.

The settings host owns one content scroll; the standalone Displays dialog owns its own single body scroll/action frame. The embedded body has no fixed 650/450px wrapper or second scroll. Topology uses contain fit, preserving relative positions/aspects, and shares numbering/role markers with rows. Number-linked callouts appear only for boxes whose role labels cannot fit; unconditional duplicate legends consume the height needed for actions. At the ordinary launch view, collapsed diagnostics, all three rows/summary and Apply/Cancel fit inside the actual content viewport, including pending-allocation feedback. Diagnostic disclosure must not hide actionable missing/degraded/stale warnings or Retry. Preferences Saved does not mean a dirty allocation draft has been applied.

### 4. Validation / Error Matrix

| Condition | Result |
| --- | --- |
| Running window, unique current native mapping | Actual MonitorId marker |
| Running window, absent/ambiguous/stale mapping or readback failure | Unknown, no requested-position fallback |
| No audience outside Presenting | NotRunning, configured PDF destination labeled separately |
| Draft role selection / Cancel | Actual markers/windows/config unchanged; Cancel restores draft |
| Confirmed Apply | Committed current window records supply new actual markers |
| Retired window or Stop/Home | Old observation no longer contributes; audience location cleared with lifecycle |
| Physical screen not assigned/occupied | Explicit unused marker; no automatic assignment invented by the view |

### 5. Good / Base / Bad Cases

Good: dragging a window changes its actual marker while its configured role remains visible and unsaved allocation stays independent. Base: Ready shows application here and PDF target not running. Bad: mark a pending choice as actual, infer actual from WindowRecord.monitor, or expose Presenter state/controllers to the audience.

### 6. Tests Required

Retain pure projection separation/ambiguity tests and actual GPUI draft/Cancel/Apply/lifecycle/two-host tests, including existing failure rollback and stale-topology regressions. Native manual move is a separate empirical check; source API availability or headless injected observation is not proof that Windows delivered movement. Core normal-window visual/interaction checks suffice for this polish task; do not add dimensions/DPI or physical hotplug matrices without user request.

### 7. Wrong vs Correct

Wrong: create a second role authority from screen numbers or auto-save observed OS moves. Correct: MonitorId settings remain explicit allocation authority, current native observations describe placement, and DisplayDraft describes pending intent.

## Transactional slide textures (task08)

### 1. Scope / Trigger

Use this contract when updating Presenter/Audience rasters or applying a GPU cache budget. CPU completion does not establish successful GPU atlas insertion.

### 2. Signatures

`TextureCache<K>::sync(frames, active, budget, window, cx)` stages desired pixels while retaining active surfaces. `paint(key, overlay, window, cx, draw) -> bool` commits only when the actual `Window::paint_image` callback returns `Ok(())`. `accepts_budget(usize) -> bool` and `effective_budget() -> usize` support runtime settings preflight. `AudienceVisual.texture_budget_bytes` is pure visual metadata; `audience::RenderStatus(Option<String>)` reports a render error without granting control access.

### 3. Contracts

Retain each committed frame with its complete overlay identity while replacement raster/upload is pending. Failed candidates are explicitly dropped; repaint the retained image and its matching overlay, with current Curtain/blank state overriding all visual layers. Presenter input must not target a newer page while showing fallback pixels. Each window gets half of total L1; half of that allocation is committed capacity and half is synchronous replacement headroom. Count old plus candidate bytes. Before changing settings, preflight both windows; rejected shrink keeps the previous effective budget.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Actual upload fails | Drop candidate; retain complete previous image; Presenter-visible error |
| Same failed pixel identity and same accepted budget | Suppress repeated failed allocation |
| New pixels or successfully changed budget | Permit retry |
| Capacity refusal, then other surfaces release capacity | Retry; capacity refusal is not an irreversible upload failure |
| Active surface awaits CPU raster | Preserve fallback, including notes |
| Successful retained-image repaint | Clear its stale paint error |
| Shrink cannot retain committed bytes plus headroom | Reject; do not save an unapplied budget |

### 5. Good / Base / Bad Cases

Good: candidate atlas insertion succeeds, then retire the old image. Base: identical pixels reuse the committed image. Bad: install a new overlay/page before its pixels, or prune active notes during a raster gap.

### 6. Tests Required

Keep task08 texture regressions for failed upload rollback, covers, identity-aligned overlays, capacity recovery, rejected-budget retry suppression, notes retention and resource accounting. Exercise actual GPUI paint-result handling and Audience error propagation. Native snapshots establish displayed PDF/notes isolation separately; paint timing is not actual Present latency.

### 7. Wrong vs Correct

Wrong: `image = candidate` before calling `paint_image`. Correct: stage candidate, attempt actual paint, commit only after success, otherwise drop candidate and repaint the retained complete projection.

## Runtime settings transaction (task09)

### 1. Scope / Trigger

Apply this contract to full Settings, shared display assignment and any workbench gesture that writes the same SettingsSession. A hidden or closed Settings panel does not end a pending runtime transaction.

### 2. Signatures

`Shell::apply_preferences(Settings, cx)` validates and starts `PendingPreferences`; `poll_preferences` waits for `RenderHandle::set_cache_budgets(CacheBudgets) -> Reply<CacheBudgetAck>`. `commit_preferences` rechecks fallible consumers before editing/saving the session; `rollback_preferences` restores acknowledged worker budgets before releasing the pending transaction. `Shell::busy()` guards competing writes, including `Intent::Split`.

### 3. Contracts

Validate the whole candidate with existing schema rules; reject fallback/default substitution for an invalid interactive edit. Preflight both GPU window budgets, validate checked core pointer style, then apply worker/display changes. Recheck GPU capacity after asynchronous work. The Audience takes its budget only from the pure snapshot scalar. Settings persistence follows successful runtime application; a save failure keeps the effective session value and dirty Retry. Future-schema sessions remain read-only. Every settings writer, including layout dragging and save/retry, observes the same pending/rollback boundary.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Invalid coupled timer limits or numeric/color text | Keep prior value and show validation feedback |
| GPU budget preflight or checked core style fails | No Settings mutation or save |
| Worker budget acknowledged, later display application fails | Restore old worker budgets before permitting another edit |
| Panel closes while acknowledgement/rollback is pending | Transaction stays active; layout drag cannot write around it |
| Persistence fails after runtime success | Effective value remains; show unsaved/Retry |
| Future schema | No edit, reset, directory mutation or save |

### 5. Good / Base / Bad Cases

Good: CPU budget acknowledgement completes, both GPU windows pass their final check, then save once. Base: an unchanged budget does not restart the PDF worker. Bad: drag a divider and save during a pending settings snapshot, then overwrite that layout when the snapshot commits.

### 6. Tests Required

Exercise actual GPUI editor/slider/key recording events and runtime consumers, normalized key conflicts/removal, both-window GPU refusal, worker acknowledgement, display failure/rollback, dirty Retry, future read-only and competing layout writes after closing the modal. Native settings effects and restart read-back are separate evidence; headless tests do not establish a theme/DPI/focus matrix.

### 7. Wrong vs Correct

Wrong: treat Settings as independent widget-local values or save before runtime acknowledgement. Correct: route edits through the shared session transaction and update full application keybindings exactly once after a successful edit; register action handlers only once.

## External PDF replacement transaction (task10)

### 1. Scope / Trigger

Apply this contract to external replacement of the currently open PDF, including cancellation by another file event, page/target changes, Close and a new import. The committed projection remains complete until the replacement can be published.

### 2. Signatures

`watch_pdf(path, debounce, sink) -> Result<PdfWatcher>` emits invalidation, stable read-only bytes and timeout events. The shell owns `poll_reload`, staging identities and per-window upload acknowledgements. `RenderHandle::commit_reversible(CandidateReady) -> Reply<RollbackToken>` returns a non-clone worker-bound token; `rollback_committed(RollbackToken)` and `retire_generation` require acknowledgements. `PresentationState::commit_reload` checks the current stage, strictly newer generation and explicit annotation-page mapping.

### 3. Contracts

Watch the parent directory so same-path rename/recreate remains observable. Use the configured quiet debounce (default300ms), stable stat interval and trailing EOF check before reading/closing/restating the file. Keep shadow bytes in memory; never write the PDF. Prepare structure and exact-target P0/P1 rasters, then stage real GPU uploads in each live window. Nonempty image bounds with an empty content mask perform atlas insertion without publishing visible candidate primitives; zero image bounds are not upload evidence. Count retained plus staged allocations and promote the same image owners after success.

Only pure staging pixels/crop/identity/budget may enter the Audience snapshot. Wait for all required window acknowledgements, then worker commit acknowledgement, then revalidate current identities and remap the latest core state before publishing. Preserve intervening timer/pause, blank and annotations. Retire the old generation only after publication. Page/target changes rebuild from retained stable bytes; pointer motion alone must not starve replacement.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Incomplete/missing file, missing EOF or invalid candidate PDF | Keep old session; debug/log only, with no error banner or modal during an external write |
| GPU upload/capacity or worker transaction failure | Preserve old complete projection and report a recoverable runtime failure |
| Commit sent, then attempt becomes obsolete | Retain its reply; successful commit requires rollback acknowledgement before candidate retirement |
| Rollback fails | Keep old UI, suppress mixed-generation rendering and expose reimport recovery |
| Retire send/ack fails after publication | Keep the published generation active; retain only the old generation as pending retirement, never fall through to Abort |
| Unknown/ambiguous label group | Use defined physical-page fallback; do not confuse notes pages with content |
| Watcher settings cannot be applied | Do not persist the unapplied candidate settings |
| Core is Idle | Reject reload without state mutation |

### 5. Good / Base / Bad Cases

Good: both windows stage successfully, the worker acknowledges commit, the current paused state is remapped, then both promote the staged images. Base: an unrelated directory event has no effect. Bad: discard an outstanding Commit reply and issue Abort as though the candidate could not already be active.

### 6. Tests Required

Cover real notify chunked writes and rename, stable-byte parsing, real single-owner PDFium, actual GPUI staging insertion/failure, obsolete Commit rollback/retirement, latest-state preservation, page/target rebuild, queued import and watcher settings failure. Isolate real PDFium tests when process-global library bindings prohibit repeated initialization; do not weaken production ownership. Headless scene/atlas results do not establish native GPU Present or latency.

### 7. Wrong vs Correct

Wrong: publish a cloned metadata-time state before worker acknowledgement, or use two public generation integers as rollback authority. Correct: retain opaque transaction ownership, revalidate after acknowledgement and publish the latest checked state atomically.

## Media control and public composition (task11)

### 1. Scope / Trigger

Use this contract for media selection, playback controls, document transitions and both-window composition. GIF/WebP run through the pure decoder; native audio/video run through the isolated helper.

### 2. Signatures

Presenter-owned `media_playback::Playback::sync`, `select`, `frames` and `slots` bridge current state/document to `ferrostage_media::Controller`. `Controller::start(Start)` returns an `Identity` containing generation/page/private slot/epoch; `latest()` yields a checked `Update`. Only `AudienceMediaFrame`'s opaque public playback-instance slot ID, public rectangle and BgraFrame reach the Audience.

### 3. Contracts

One supervisor owns one active decoder/audio source. Choose the first public autostart slot in document order on entering a Presenting page. Ready and notes-only slots never play. Stop the previous owner before starting another; synchronously invalidate accepted identities on page, blank, Stop, Close, committed-generation or successful document geometry reconfiguration. Leaving blank does not revive the prior decoder. Failed document reconfiguration preserves the prior valid state. Direct page selection follows the same media invalidation contract as Next/Previous; page-number editing alone does not change playback.

Use a checked process-wide playback-instance counter so controller/helper recreation cannot reuse a cached surface ID. Filter generation/page/epoch/sequence before publication. Keep source paths, embedded bytes, progress, errors and controls private. Pipe read/write, stderr drainage, bounded process shutdown/reap and decode/crop/scale run off UI. Keep commands and latest frames bounded; no UI blocking join. Missing native runtime must leave PDF and independent GIF/WebP usable.

Fit media into a bounded transparent annotation-aspect canvas, then crop the corresponding source pixels to the public intersection. Bound canvas and output before allocation; animation peak accounting includes retained frames, decode scratch, the contain canvas and crop output before decoding the next frame. Do not stretch or squeeze hidden regions into the visible rectangle. Both windows share the resulting BgraFrame and use their existing budgeted texture cache. Paint PDF, then media, then ink/laser; Curtain and blank cover all layers. Media rectangle clicks apply to Navigate mode, with explicit controls available for other tools.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Ready or entirely private annotation | No playback/audio owner |
| Missing helper/runtime/source or unsupported codec | Typed localized Presenter status; retain PDF |
| Partial native runtime | Detect unavailable inputs before unsafe startup assumptions; GUI remains usable |
| Late frame after invalidation | Reject; cannot revive playback or old texture fallback |
| Duplicate or regressing native sequence | Reject within the current instance; epoch still isolates different instances |
| New-instance upload/budget failure | Never display prior-instance pixels |
| Malformed IPC geometry/length or source allocation limit | Reject before allocating the claimed buffer |
| Helper EOF/crash/stop timeout | Bounded cleanup and recoverable status; never start a second owner before settlement |

### 5. Good / Base / Bad Cases

Good: a crossing annotation projects only its authorized cropped pixels, with both windows sharing one immutable allocation. Base: a paused frame remains visible while position is held. Bad: reuse annotation index0 as the public cache identity across pages or expose private-source controls through the snapshot.

### 6. Tests Required

Exercise real GIF/WebP delays, safe native MP4/WebM/audio-only control, start/duration/repeat, pause/resume/EOS, missing and partial runtime, cancellation/reaping, malformed IPC, memory bounds and same-allocation publication. Keep real document mapping/notes-only/crossing-crop and actual texture-failure/instance-identity regressions. Native tests use a silent sink until the separately observed desktop pass. Verify the scanner path against the selected runtime manifest and run native packaging regressions with a fresh isolated plugin registry. GUI PE imports, native package provenance and explicit native compile/lint are separate from default CI. Long performance/VRAM and stop-latency matrices remain separately classified.

### 7. Wrong vs Correct

Wrong: link native video DLLs into the GUI, rely on gst initialization to survive loader failure, or block the UI waiting for a decoder. Correct: keep the GUI graph pure, supervise the optional helper in the background and publish only currently authorized public pixels.

## Forbidden patterns

- A global `lazy_static` deck.
- Cloning speaker notes into the snapshot "just for the audience highlighter".
- Two copies of the current page index, one in the view and one in the core. The view sends a command and renders the snapshot it gets back.

## Monitor error recovery

### 1. Scope / Trigger
Applies to initial discovery, watcher failures/recovery and Detect again in both display-settings hosts.

### 2. Signatures
`Shell::monitor_error: Option<PlatformError>` owns discovery errors separately from `Shell::error` operation failures. `Shell::monitor_error_text()` projects the current typed failure (or watcher-start failure) through `Locale::monitor_error`. `DisplayPanel::monitor_error` carries the localized result into the shared panel.

### 3. Contracts
Startup failure and Unavailable set only the discovery-error slot. Changed sets valid topology and clears only that slot, including recovery to the same monitors. Never clear unrelated import/render/save/operation errors. Start/Swap shortcuts must not duplicate unavailable topology into the operation-error slot. Apply must leave derived panel validation in its live panel owner, including failure during its fresh probe and a stale draft after a successful topology change. A missing watcher remains a failure after successful synchronous detection; it cannot be repaired by clearing the discovery slot. Log structured typed diagnostics at startup/distinct failure delivery, not every healthy poll. Presenter and shared panel must use catalog copy instead of `PlatformError::to_string()`.

Both Settings > Displays and standalone Display settings use the same recovery instructions and existing Detect again action: allow connections to settle, use Windows System > Display > Multiple displays > Detect when missing, select Extend when independent presenter/PDF screens are desired, detect again in FerroStage and explicitly choose destinations/Apply. Do not change Windows settings automatically, reset all preferences or silently replace an absent audience. Failure-specific guidance disappears after successful recovery; a permanent short help hint may remain.

### 4. Validation / Error Matrix

| Transition | Expected state |
| --- | --- |
| Startup or watcher discovery failure | Localized monitor reason, invalid topology, projection disabled |
| Failed discovery then unchanged-topology Changed | Monitor message clears; unrelated operation error survives |
| Detect again succeeds after unavailable | Fresh topology is usable, prior monitor error clears |
| Apply probe fails or makes its draft stale | Live panel reason remains visible; no duplicate global error |
| Unavailable topology plus Start/Swap shortcut | No transaction/config mutation or duplicate global error |
| Watcher start failed, synchronous detection succeeds | Watcher failure remains visible; presentation stays blocked |

### 5. Good / Base / Bad Cases
Good: a transient failure recovers without leaving a red English banner. Base: healthy topology stays quiet. Bad: a shared unowned error string survives recovery, or clearing all errors hides an unrelated failure.

### 6. Tests Required
Run a real red/green same-topology recovery regression using the existing Shell interface. Cover startup locale projection, watcher lifecycle, preserved operation errors, Apply failure/stale-draft recovery and actual Start/Swap shortcuts, plus both shared panel hosts/retry controls using GPUI dispatch. Catalog tests must cover every PlatformError variant and matching interpolation fields. Keep native startup/three-monitor/GDI evidence separate from headless UI and physical hotplug acceptance.

### 7. Wrong vs Correct
Wrong: translate the global string and set it to None on every topology event. Correct: retain typed owner-specific discovery state, localize during presenter capture and clear it on successful recovery only.
