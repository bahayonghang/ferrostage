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

Core `AudienceProjectionSnapshot` includes `stage` and `visual`; its metadata projection always sets `visual=None`. Tasks 08/11 supply matching pixels. Task 02 source now contains a sectioned settings schema and `SettingsStore`; runtime UI application and save-status display remain task 09 work. The audience window is absent outside `Presenting`. A saved Audience that is missing does not fall back to another monitor (see the [frontend quality guidelines](./quality-guidelines.md)).

The audience view receives only the assembled snapshot, never the document, notes, media sources, a full-cache handle, or an arbitrary-page request API. The assembler publishes matching generation/page/crop/target pixels together; it keeps the previous complete projection until the next one is ready. `Curtain` and black/white blank cover every visual layer immediately.

Blank mode is `Option<BlankMode>` on the snapshot so the audience window does not invent a second blank flag.

## Forbidden

- A global `lazy_static` deck.
- Cloning speaker notes into the snapshot "just for the audience highlighter".
- Two copies of the current page index, one in the view and one in the core. The view sends a command and renders the snapshot it gets back.
