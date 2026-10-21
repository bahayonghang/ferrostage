# Dismissible document-warning flag

## Goal

On the Ready and Presenting workbench, every document warning that is now a bare line above the toolbar becomes a small closable status flag. The flag floats in that corner. It does not take a row away from the slides and notes.

The user pointed at `已根据页标签将叠加步骤分组。` on `defense.pdf` (Ready, dark theme, 2026-10-20) and asked for a closable popup flag. They then confirmed the recommendation: every document warning uses that flag. Shell warnings stay inline.

## Background

- The circled string is `ln-warning-overlays` (`assets/locales/zh-CN/app.toml:169`, en-US: "Overlay steps were grouped using page labels."). `DocumentWarning::OverlayHeuristic` is pushed when a label group has more than one page (`crates/ferrostage-document/src/builder.rs:191-192`).
- `document_warnings` (`crates/ferrostage-ui/src/presenter/layout_navigation.rs:62-112`) localizes every `DocumentWarning` plus active-content and structure diagnostics, then sorts and dedups the strings.
- The presenter concatenates `shell.warnings` with those strings (`presenter.rs:96-101`) and paints each as a borderless flex line in `p.warning` (`presenter.rs:1383-1389`). That line shortens the workbench. The same loop also runs on Idle.
- `Intent::DismissError` (`shell.rs:664-667`) clears the error banner only. Layout reconfigure keeps `DocumentGeneration` (`shell/layout_navigation.rs:32-36`). Import and hot reload allocate a new generation (`shell.rs:879-882`, `shell/reload.rs:544-545`).
- The header stage badge (`presenter.rs:808-831`) is the status sibling: 4px radius, `p.soft`, no shadow, 8px dot. Warning tokens are light `#825315` and dark `#E8B774` (`theme.rs:180-193`). `Presenter::button` faces are 44px (`presenter.rs:560-565`) inside a 2px focus reservation (`presenter.rs:717-722`), and the face moves down 1px on press (`presenter.rs:619-621`).
- `DESIGN.md` keeps Idle import/topology warnings and the unsaved/Retry row in document flow. Interactive targets are at least 44×44. No nonessential motion. The audience view does not receive presenter chrome.

## Requirements

- R1. On Ready and Presenting, each visible document warning is one flag. The bare flex line for those strings is gone. `presenter-content` keeps the same height it has when no document warning is visible.
- R2. Close hides that exact localized string while `state.generation()` is unchanged. Dismissal is not written to settings. A new `DocumentGeneration` (another PDF, or a hot reload) shows the warning again when it is still emitted. Layout & navigation reconfigure does not bring back a dismissed warning that the reloaded document still emits.
- R3. Warning copy stays the existing localized strings. The close control has its own localized name.
- R4. `shell.warnings` stay bare inline lines on Idle, Ready, and Presenting. The error banner and the unsaved/Retry row stay as they are.
- R5. The audience window does not render the flag or the warning string.

## Flag geometry

Anchor the stack to the bottom-left of `presenter-content`, 12px from the left edge and 12px above the bottom edge. Paint it after the workbench, `absolute`, so the column does not shrink. `workbench-controls` stays below the content box, so the flag does not cover the toolbar.

| Part | Spec |
| --- | --- |
| Surface | `p.card`, 1px `p.border`, 8px radius. No shadow. |
| Row | Text vertically centered with the close face. Padding starts at 12px on the left. Gap 8px. |
| Mark | 8px filled circle in `p.warning`. |
| Text | 14px medium, `p.warning`, wraps inside a 420px max width. No ellipsis. |
| Close | `activation_target` plus a quiet face: 44×44, `×` in `p.body`, transparent at rest, `p.soft` on hover, 1px down on press, no transition. The existing focus ring wraps that face. |
| Stack | Upward, 8px gap. Each flag closes on its own. A stack taller than the content box clips with `presenter-content`. |

The flag is `Role::Status`. Escape does not close it. Tab reaches Close only when no menu or page browser is taking the tab cycle, matching `activation_target`.

## Out of scope

- Rewording warning strings, or changing when `DocumentWarning` and structure diagnostics are produced.
- Turning `shell.warnings` into flags, or persisting dismissal across launches.
- Restyling the error banner, the header stage badge, or the tools strip.
- Auto-dismiss timers, enter/exit motion, and a modal dialog.

## Acceptance criteria

- [ ] A Ready document that emits `OverlayHeuristic` shows that sentence in `document-warning-0`: card, 1px border, 8px warning dot, 14px warning text, and `dismiss-document-warning-0`. `presenter-content` height matches the same window with the flag dismissed.
- [ ] Activating Close removes that flag. It stays gone across a layout reconfigure that still emits the warning and keeps the generation. A later `finish_import` or hot reload with a new generation shows it again.
- [ ] Two document warnings stack upward with an 8px gap. Closing one leaves the other.
- [ ] The flag's bottom is at or above the top of `workbench-controls`.
- [ ] A string pushed only into `shell.warnings` renders as `shell-warning-*` and never as `document-warning-*`, on Idle and on Ready.
- [ ] Tab reaches the close control, Enter/Space activates it, and Escape leaves the flag up.
- [ ] Light and dark both use `palette()` for card, border, and warning. The audience window has no `document-warning-0`.
