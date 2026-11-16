# Audience click to advance: approved design

## Decision and scope

Ship one cohesive change: one typed setting, one outgoing audience event, and coordinator routing through existing Next. Keep the core and vendor source unchanged. The user approved the PRD behavior package and implementation on 2026-10-18.

Minimal alternative: hard-wire audience mouse-down directly to Next. Reject it because it omits the requested setting, fires before a completed click, and bypasses source-window validation. Passing a Shell/settings reference into Audience is also rejected because it breaks the existing snapshot-only input contract.

## Current evidence

See `research/source-analysis.md` for source anchors and framework behavior. Local vendored GPUI is the authoritative implementation for the pinned framework; no external API or dependency is needed.

## Contracts

### Optional v1 settings leaf

- Add `DisplaySettings::audience_click_to_advance: bool` and TOML `display.audience_click_to_advance`.
- Default: true. Missing and invalid values use that default through existing `parse_bool`, with invalid types producing the established leaf warning.
- Extend `parse_display` and `merge_display` together. Keep `SCHEMA_VERSION = 1`; do not add a legacy alias, section, env var, command, or schema migration.
- Add a `Kind::Toggle` field under `Category::Displays`. Its label keys follow `settings-field-display-audience_click_to_advance`; add localized helper text stating that the change saves automatically, applies immediately, and acts only during normal playback.
- In `continue_preferences`, normalize this non-placement field in the comparison copy exactly as `presenter_start_fullscreen` is normalized. Changing only the click toggle goes to `commit_preferences`, without `assign_displays`. A combined real display-placement edit must still take the existing display transaction.
- Do not introduce an independent cached enable flag. Every accepted input reads the committed effective settings in Shell, so an already-open audience sees edits immediately, including effective edits whose disk save failed.

### Outgoing audience event

- Add a private payload-free event, e.g. `AdvanceRequested`, and `EventEmitter<AdvanceRequested>` beside existing audience status events.
- Attach stable ID `audience-playback-surface` and GPUI `on_click` to the root full-size audience div. Accept `ClickEvent::Mouse` with left-button down/up; do not turn keyboard synthetic click activation into another Next binding.
- Use framework press/release hit testing. Do not add a second mouse-down navigation path, debounce timer, double-click suppression, pointer coordinate mapping, or custom drag recognizer. The contract covers completed clicks and release outside cancellation, not a new drag gesture policy.
- Audience emits the request only; it does not receive enablement, Shell, settings, notes, document, or render worker. Keep existing render/staging event behavior intact.

### Coordinator acceptance and routing

Subscribe when `Shell::open` creates Audience, mirroring existing status subscriptions. Capture the source `AnyWindowHandle`. Before dispatch, check all of:

1. Handle equals the current committed `self.audience` handle; a pending or old window never qualifies.
2. `state.stage() == Presenting` and effective `display.audience_click_to_advance` is true.
3. `modal.is_none()` and `!busy()` using the existing busy definition, including reload and preference changes.
4. Published projection is `AudienceStage::Live`, and both effective core blanking and published blanking are absent.

Then call existing `dispatch(Intent::Next, cx)` exactly once. Do not call `state.apply_at` directly from Audience or add a new core command. Do not activate the presenter as a side effect. This preserves existing normal page cleanup and matching-frame publication.

Presenter tools have no audience mouse handlers: selecting Pen/Laser on the presenter does not disable this independent audience navigation route. Existing presenter media/tool behavior is unchanged.

## Data flow

```text
Settings toggle -> checked edit -> preference commit -> SettingsStore
                                  |
                                  v
Audience click -> AdvanceRequested -> Shell guards -> Intent::Next -> NextStep
      ^                                                               |
      |                                                               v
      +---------- assembled AudienceProjectionSnapshot <-------- publication
```

The outgoing event grants no read access to Shell. The only incoming Audience product data remains the snapshot; no settings are added to the projection type.

## Compatibility and failure behavior

- Default-enabled is an intentional new behavior for old settings that omit the leaf; approved explicitly by the user. Explicit false always disables it. An old binary should preserve the new unknown leaf through the existing merge path.
- Existing black/white behavior for keyboard/buttons remains unchanged; only the new mouse route ignores blanked playback. No click-to-unblank behavior is added.
- Save failure keeps current runtime behavior plus Unsaved/Retry. A future settings version stays read-only in the settings UI; the parsed effective value still governs input.
- Window replacement/unplug/Stop cannot accept stale requests because every event validates its source against the committed window.
- No PDF data, account, credential, network, child HWND, dependency, or worker-thread contract changes.

## Rollback

Revert the cohesive implementation change. Unknown-key-preserving settings make the extra boolean harmless to the previous binary; deleting user configuration is unnecessary. Product toggle offers an immediate behavioral opt-out before rollback.

## Change surface

Expected 13 product/test/locale files plus 2 focused spec updates. The count exceeds 8 because this project deliberately separates typed schema, parser, merge, store tests, view, coordinator, and locales; it does not imply new subsystems. Exact file ownership is listed in `implement.md`. No new service or production source file is needed.
