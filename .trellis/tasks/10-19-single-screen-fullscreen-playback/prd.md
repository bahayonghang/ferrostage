# Single-screen fullscreen playback and release 1.2.0

## Goal

When the operating system reports exactly one monitor (one physical screen, or Windows "Duplicate these displays"), Start must show the PDF slides fullscreen on that screen, in the style of a PDF reader full-screen mode. The fullscreen view shows slide content only. It never shows speaker notes, the notes half of a Beamer split page, or the presenter interface. The workspace version becomes 1.2.0.

## Confirmed facts

- The supplied 1.1.1 screenshot shows one monitor and the start error `未找到观众显示器；单屏须显式启用窗口化观众模式` (`assets/locales/zh-CN/app.toml:57`, key `no-audience`).
- `SingleMonitorMode` has two values: `PresenterOnly` (default) and `WindowedAudience` (`crates/ferrostage-settings/src/schema.rs:55-69`, default at `:221`).
- `policy::audience_target` returns `NoAudience` for one monitor with `PresenterOnly`. With `WindowedAudience` it returns the presenter monitor with `fullscreen = false` (`crates/ferrostage-ui/src/policy.rs:93-106`).
- `Shell::start` always calls `enter_presenter_fullscreen` before it opens the audience window (`crates/ferrostage-ui/src/shell.rs:1336-1369`). With `WindowedAudience`, the result is a fullscreen presenter window (with notes) plus an 800 x 760 audience window on the same screen. In Duplicate mode the projector receives the presenter interface.
- Settings save always writes `display.single_monitor_mode` (`crates/ferrostage-settings/src/merge.rs:83-88`). Existing settings files therefore contain `presenter_only`.
- The audience window opens with `focus: false` (`shell.rs:1066`). Global key actions are App-level (`crates/ferrostage-ui/src/actions.rs:89`), so Next, Previous, Black, White and Stop (default `Escape`) already work when the audience window is the active window (test `input_route_audience_open_activates_presenter_and_escape_cancels`).
- `AudienceProjectionSnapshot` has no notes field. `slide_crop` removes the notes half of Beamer split pages (`crates/ferrostage-document/src/layout.rs:32-40`). Independent notes pages are excluded by the slide map.
- ADR-006, ADR-007, `docs/architecture.md:48` and `docs/PROJECT_PLAN.md:105,143` define the current single-monitor exception.
- The workspace version is set once in `Cargo.toml:36` (`1.1.1`); `Cargo.lock` repeats it for each workspace crate.

## Requirements

- R1: With exactly one monitor and the new default mode, Start opens the audience window fullscreen on that monitor. The presenter window stays on the same monitor, does not enter fullscreen, and stays behind the audience window.
- R2: The fullscreen audience window receives keyboard focus. Next, Previous, Black, White and Stop use the existing key bindings. `Escape` (default `stop_presentation`) ends playback and returns to the presenter window in `Ready`. Shortcuts that activate the presenter window (Open, Settings, Jump, Displays) do nothing during single-screen playback, because activation would show the presenter interface on the projected screen.
- R3: The fullscreen view shows only `AudienceProjectionSnapshot` content: slide crop, overlays, transitions, media and blank modes. Notes text, the Beamer notes half and independent notes pages never appear.
- R4: Add the mode value `fullscreen_audience`. It is the default. A saved `presenter_only` value loads as `fullscreen_audience` (see design decision D1). `windowed_audience` keeps its current behavior.
- R5: The Displays dialog on one monitor offers "fullscreen on this screen" and "window on this screen". The selection persists through the existing display transaction.
- R6: Topology change during single-screen playback (for example, the user switches from Duplicate to Extend) uses the existing recovery path. No notes become visible on a projected surface as a result of the recovery itself.
- R7: Bump the workspace version to 1.2.0 in `Cargo.toml` and `Cargo.lock`.
- R9: The Start button in the bottom-left tools panel has a dropdown trigger on its right side. The dropdown offers "Dual screen" (default) and "Single screen". Dual follows the display roles (and the single-monitor mode on one monitor). Single always plays slides fullscreen on the presenter monitor, at any monitor count. The choice persists as `display.start_mode` (`dual` | `single`). The trigger is hidden while presenting.
- R10: Settings has an About tab that shows the version, author, build time, license and the GitHub repository link. No git commit row (user review of 2026-10-19).
- R11: Event error messages in the presenter window show as a flag with a close button. Closing clears the message. Persistent condition messages have no close button.
- R8: Update ADR-006, ADR-007, `docs/architecture.md`, `docs/PROJECT_PLAN.md`, the frontend state-management spec, and both locale files to describe the new single-monitor behavior.

## Acceptance criteria

- [x] AC1 (R1, R4): Policy test: one monitor with default settings resolves to the presenter monitor with fullscreen playback; one monitor with `windowed_audience` keeps windowed playback; two monitors ignore the single-monitor mode.
- [~] AC2 (R1, R2): GPUI test: Start on one monitor in `fullscreen_audience` opens the audience window with fullscreen bounds and focus, does not toggle presenter fullscreen, commits `Presenting`, and makes the audience window active.
- [x] AC3 (R2): GPUI test: keystrokes on the active fullscreen audience window advance and go back exactly one step; `escape` stops, closes the audience window and leaves the presenter window in `Ready`; `ctrl-o` and the settings shortcut do not activate the presenter window.
- [x] AC4 (R3): Test with a right-split Beamer notes fixture: the published audience snapshot has `slide_crop == LEFT_HALF`, and navigation in single-screen playback skips independent notes pages.
- [x] AC5 (R4): Settings tests: `presenter_only` parses to `fullscreen_audience` without a warning; save writes `fullscreen_audience`; `windowed_audience` round-trips; an unknown value warns and uses the default.
- [x] AC6 (R5): Display dialog test: on one monitor the two choices map to `FullscreenAudience` and `WindowedAudience`; the windowed-only validation stays.
- [x] AC7 (R7): `cargo metadata --locked --offline` succeeds and every workspace crate reports 1.2.0.
- [x] AC10 (R9): Policy test: `single` resolves to the presenter monitor, fullscreen and shared, with one or two monitors. GPUI test: the trigger opens the menu with Dual checked; selecting Single closes the menu, saves `start_mode = "single"`; while presenting the trigger is absent and the intents do nothing. Settings test: `start_mode` defaults to `dual` and round-trips.
- [x] AC11 (R10): GPUI test: the About tab renders version, author, build time, license and repository rows and no commit row; the version is 1.2.0; the build time format gives `2026年10月19日 23:35` / `2026-10-19 23:35` plus `UTC-04:00`; clicking the link opens the repository URL.
- [x] AC12 (R1): Single-screen Start does not gate on the untoggled presenter's bounds (`single_screen_start_accepts_an_untoggled_maximized_presenter`).
- [x] AC13 (R11): `event_error_flag_closes_and_persistent_monitor_error_has_no_close_button`.
- [x] AC8: `just ci` passes.
- [ ] AC9 (native): On the Windows host with one monitor (or Duplicate mode), `just dev` with `defense.pdf`: Start shows slides only, fullscreen, no taskbar; arrow keys and click advance; `Escape` returns to the presenter. Record NOT_RUN with a reason if the native check cannot run.

## Out of scope

- Laser, pen, highlighter and eraser on the fullscreen audience window.
- Right-click to go back, auto-hide cursor, and a separate "full screen" shortcut beside Start.
- Multi-monitor behavior changes, installer changes, release publication.

## Decisions (confirmed by the user on 2026-10-19)

- D3 (request of 2026-10-19, turn 3): the start mode is a persisted setting with default `dual`. Author comes from the workspace `authors` field (git user name). Build time is the time the app build script last ran (`SOURCE_DATE_EPOCH` when set).

- D1: A saved `presenter_only` value loads as `fullscreen_audience`.
- D2: The presenter window stays windowed behind the fullscreen audience window.

## Delivery status

See `research/validation.md`. AC2 is partial: headless tests cover the focus option, derived single-screen state and input; fullscreen placement confirmation, the skipped presenter toggle and commit activation need the native check. AC9 is NOT_RUN.
