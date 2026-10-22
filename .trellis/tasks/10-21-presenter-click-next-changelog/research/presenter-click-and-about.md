# Presenter click-next and About changelog

Evidence for task `10-21-presenter-click-next-changelog`. Inspected 2026-10-21.

## Presenter surfaces

`Presenter::workbench_panel` (`crates/ferrostage-ui/src/presenter/workbench_view.rs:1130`) builds two layouts:

- `NotesFirst` (`StartMode::DualNotes`, width >= 800): `notes-panel` (scroll), `column-split`, then `previews-column` with captioned `current-pane` and `next-pane`.
- Standard / stacked: `current-pane`, then `next-notes-pane` with captioned `next-pane`, `vertical-split`, and scrolling `notes-panel`.

Both layouts keep ids `current-pane`, `next-pane`, and `notes-panel` (`component-guidelines.md` presenter composition contract).

`picture(Surface::Current, true)` is the only interactive slide (`workbench_view.rs:1342`). Mouse-down on that surface handles media slots, pen, highlighter, and eraser (`workbench_view.rs:322-399`). Next and Notes call `picture(..., false)` (`workbench_view.rs:1200`, `:1368`) and have no click handler. Notes rasters sit in `overflow_y_scroll` (`workbench_view.rs:1293`, `:1472`). There is no notes zoom control in `ferrostage-ui`.

Splitters own left mouse-down and must stay drag handles (`workbench_view.rs:1120`, `:1391`, `:1452`). The page grid commits `Jump`, not Next (`state-management.md` manual navigation).

## Existing Next and audience click

`Intent::Next` becomes `Command::NextStep` only when no settings modal, no contextual surface, no pending transaction, and stage is not `Idle` (`shell.rs:801-816`). `Ready` and `Presenting` both qualify. Blank mode is not a Next guard. Last-step and notes-page exclusion stay in core `SlideMap::next_step`.

Audience advance is a separate event. The shell accepts it only for the committed audience window, while Presenting, `display.audience_click_to_advance`, no modal, no contextual surface, not busy, Live, and not blank (`shell.rs:1243-1264`). The setting label is the playback window (`settings-click-to-advance`). `audience_click_does_not_bind_the_presenter_slide_preview` (`shell.rs:4902-4944`) currently asserts a click on `surface-Current` does not change the page.

## About and version

Workspace version is `1.2.0` (`Cargo.toml:36`). About reads `env!("CARGO_PKG_VERSION")` (`about.rs:4`). The About tab renders five rows and no changelog control (`settings_view.rs:378-457`). `about_tab_shows_version_author_build_time_license_and_repository` asserts `VERSION == "1.2.0"` (`preferences_tests.rs:942-964`).

Version history is commit boundaries, not git tags:

| Version | Commit | User-facing delta after the previous bump |
| --- | --- | --- |
| 1.0.0 | `746b18b` | First release: dual-window PDF/Beamer presenter |
| 1.1.0 | `f6f5c48` | Fullscreen notes, outline, transitions, timed playback, recent-file times, Windows installer |
| 1.1.1 | `e262705` | Installer accepts Windows 10 1903 / build 18362 x64. The interim notes-first layout was reverted before this bump |
| 1.2.0 | `487653e` | Single-screen fullscreen playback, start-mode menu, About page |
| since 1.2.0 | `aa6584b`, `4b3a572`, `51f0570`, `a9e5e65` | Notes-first dual layout, resumable talk timer, dismissible document-warning flag |

`6ef6a1a` is a packaging-script encoding fix and is not end-user changelog copy. Packaging reads the workspace version; do not add a second hardcoded product version. `just release` is outside this task.

## UI state

`PanelState` (`settings_panel.rs:270`) is cloned into the presenter each frame (`settings_view.rs:18`). `confirm_reset` is cleared on category change (`preferences.rs:49`). A changelog flag belongs on that panel state and an `Intent` handled by `dispatch_preference`. It is not a settings schema leaf.
