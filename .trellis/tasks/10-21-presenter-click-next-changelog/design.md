# Design: presenter click-next and About changelog

## Boundary

The presenter workbench dispatches the existing `Intent::Next`. Core `NextStep`, audience click policy, and settings schema stay as they are. Changelog text is a static localized list. Version identity remains `CARGO_PKG_VERSION` via the workspace package version.

## Click path

Add one helper on `Presenter` that dispatches `Intent::Next` for a GPUI `ClickEvent` whose down and up buttons are both left. Attach it with `on_click` to three regions in both layout branches:

- the slide body of `current-pane` (the picture or the rendering placeholder, under the caption)
- the slide body of `next-pane`
- `note_panel`, which already wraps the notes caption, rasters, and empty line, inside the scrolling `notes-panel`

Do not attach the handler to splitters, `workbench-controls`, or the page browser.

`picture(Surface::Current, true)` already receives the drawing and media gesture. For that surface:

- When the tool is not Navigate, stop the click so the pane handler does not advance.
- When Navigate and the point lies in a media slot, keep `Intent::Media` on mouse-down and stop the click.
- Otherwise let the pane `on_click` dispatch Next once.

Next and Notes pictures stay non-interactive. Their pane or `note_panel` handler is the only advance path, including the rendering placeholder and `wb-no-notes`.

`overflow_y_scroll` stays on `notes-panel`. The click handler sits on the inner `note_panel`, so a scrollbar drag is not that target. Wheel events are not clicks.

Shell guards stay in the existing `Intent::Next` arm (`shell.rs:801-816`). The presenter does not reimplement blank, busy, or `audience_click_to_advance` checks. Update `audience_click_does_not_bind_the_presenter_slide_preview` so a current-slide click advances once and an audience click still follows its own policy.

## Changelog

Add `PanelState.changelog_open: bool`, default false. Add `Intent::ToggleChangelog`, handled in `dispatch_preference` by flipping the flag only while Settings is open and the category is About. Clear the flag in `SettingsCategory` and `dismiss_preferences`, next to `confirm_reset`.

Render `about-changelog` through the existing settings button / `activation_target` path at the top of the About column. When the flag is set, render a scrollable block of version headings and paragraphs under the button. Headings use the version strings `1.2.1` … `1.0.0`. Paragraphs use locale keys, one paragraph per version, copy locked by the PRD table. Closing the log leaves the five metadata rows in place.

No settings key, schema migration, or repository request.

## Version

Set `[workspace.package].version` to `1.2.1` and refresh only the FerroStage package version records in `Cargo.lock`. Update the About test expectation from `1.2.0` to `1.2.1`. Do not introduce another version literal for the installer or About row; those already read the package version. The changelog heading `1.2.1` is content, not a second runtime version source.

## Compatibility

Old settings files gain no key and stay valid. Audience click tests that do not click the presenter panes stay valid. Drawing on the current slide stays valid. Pane ids and splitter drag stay valid.

## Rollback

The click helper, About flag, locale keys, and version literal are independent. Reverting the UI commit restores button/audience-only advance and the 1.2.0 About tab. Reverting the version commit restores 1.2.0 without a settings migration.
