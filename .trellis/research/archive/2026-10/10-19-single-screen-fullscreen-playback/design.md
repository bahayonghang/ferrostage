# Design: single-screen fullscreen playback

## Boundary

- `ferrostage-core`: no change. `StartPresentation { audience_available: true }` stays the only start command. The audience window still exists only in `Presenting`.
- `ferrostage-document` / `AudienceProjectionSnapshot`: no change. Notes isolation is already a type property; slide crop and the slide map already remove Beamer notes halves and independent notes pages. This task adds tests that prove the property for the new path, not new filtering.
- `ferrostage-settings`: one enum value, one legacy alias, new default.
- `ferrostage-ui`: placement policy, Start transaction, focus routing, Displays dialog, locale text.
- No GPUI patch, no new dependency, no child HWND.

## Settings contract

```rust
pub enum SingleMonitorMode {
    /// Fullscreen audience window on the only monitor. Default.
    FullscreenAudience,
    /// Windowed audience view on the same monitor (current behavior).
    WindowedAudience,
}
```

- TOML values: `fullscreen_audience` (default), `windowed_audience`.
- Parse: `presenter_only` maps to `FullscreenAudience` without a warning (decision D1). Every save writes the new value, so the alias disappears from the file after the first save.
- `display.audience_fullscreen` keeps its meaning for a separate audience monitor only. Single-screen fullscreen does not read it.
- Rollback: a 1.1.x build reading `fullscreen_audience` warns `display.single_monitor_mode` and falls back to its default `presenter_only` (existing `parse_choice` behavior). No data loss.

## Placement policy

Replace the tuple result of `policy::audience_target` with a named result:

```rust
pub struct AudienceTarget {
    pub monitor: MonitorId,
    pub fullscreen: bool,
    /// The audience shares the presenter monitor (one monitor, or `StartMode::Single`).
    pub shared: bool,
}
```

| Monitors      | Mode                 | Result                                                                                                            |
| ------------- | -------------------- | ----------------------------------------------------------------------------------------------------------------- |
| 1             | `FullscreenAudience` | presenter monitor, `fullscreen = true`, `shared = true`                                                           |
| 1             | `WindowedAudience`   | presenter monitor, `fullscreen = false`, `shared = true` (unchanged)                                              |
| 2+            | any                  | resolved Audience, `fullscreen = audience_fullscreen`, `shared = false`; `NoAudience` when unresolved (unchanged) |
| 0 / ambiguous | any                  | error (unchanged)                                                                                                 |

`start_reason`, `DisplayDraft::new`, `swap` callers and the no-play checks use the new struct. Only the field access changes for them.

## Start transaction (`Shell::start`)

For `shared && fullscreen` ("single-screen playback"):

1. Do not call `enter_presenter_fullscreen`. The presenter window keeps its bounds, so `presenter_restore` stays `None` and `restore_presenter` is a no-op on stop or failure.
2. `publish(true)` (curtain), then `open(target.monitor, true, audience = true)`. `open` computes `takes_focus = !audience || fullscreen && presenter.monitor == monitor` and passes it as `WindowOptions.focus`; the signature does not change. Bounds come from the same display's `visible_bounds()` as today, wrapped in `WindowBounds::Fullscreen`.
3. Confirmation uses the existing `Transaction { kind: Start }` path and timeout. `start_presenter_confirmed` returns true when `presenter_restore` is `None`, because this Start did not toggle the presenter. Without that rule a maximized or user-resized presenter fails the windowed client-size and margin check, and Start times out with `placement-timeout` behind a black curtain (native report of 2026-10-19; regression test `single_screen_start_accepts_an_untoggled_maximized_presenter`).
4. On commit with a new audience window, when `single_screen_playback()` is true, call `activate_window()` on the audience handle so it is in front of the presenter and owns keyboard input. The same check covers an Assign commit that replaces the audience.

All other targets keep the current code path, including the presenter fullscreen toggle for `WindowedAudience` and multi-monitor playback.

## Input and focus

- Navigation, blank and stop keys are App-level actions (`actions.rs`). They work from the active audience window without change. Default `Escape` runs `stop`, which retires the audience window; the presenter window becomes visible in `Ready`.
- `request_focus` activates the presenter window for Open, Settings, Jump and Displays. During single-screen playback it returns before activation. Reason: in Duplicate mode, activation shows the presenter interface with notes on the projector.
- Left click to advance uses the existing `audience_click_to_advance` path.
- Alt+Tab to the presenter window is a user action outside the application; it is documented, not blocked.

## Single-screen state

No stored flag. `Shell::single_screen_playback()` is derived: the audience record is fullscreen and its `monitor` equals the presenter record's `monitor`. Two or more monitors never share a monitor, because Displays validation rejects a conflicting target. When `self.audience` is retired (`stop`, topology recovery, user close, Home, Assign), the state becomes false without extra code.

## Displays dialog

`PdfTarget` gains `SameScreenFullscreen`. On one monitor the monitor card shows two PDF choices:

| Choice id                | Label key                                                   | Target                 | Saved mode           |
| ------------------------ | ----------------------------------------------------------- | ---------------------- | -------------------- |
| `display-pdf-fullscreen` | `displays-role-fullscreen` (new, "全屏播放" / "Fullscreen") | `SameScreenFullscreen` | `FullscreenAudience` |
| `display-pdf-windowed`   | `displays-role-windowed`                                    | `Windowed`             | `WindowedAudience`   |

`DisplayDraft::new` maps a `shared && fullscreen` target to `SameScreenFullscreen`. `validation_error` rejects both same-screen targets when the monitor count is not 1 (`displays-windowed-single`, wording generalized). No-play on one monitor has no saved representation after D1, so `validation_error` rejects it (`displays-no-play-single`) and the No-play button is disabled with one monitor. On 2+ monitors `settings()` writes `FullscreenAudience`, so a later unplug to one monitor starts fullscreen.

## Locale

- `no-audience`: remove the instruction to enable windowed mode; it now applies only to multi-monitor setups without an Audience (zh-CN: `未找到观众显示器，请在显示器设置中分配`).
- Add `displays-role-fullscreen` and `displays-no-play-single` in both locales.

## Documents to update

ADR-006 and ADR-007 (single-monitor exception now has two modes, fullscreen default), `docs/architecture.md:48`, `docs/PROJECT_PLAN.md:105,143`, `.trellis/spec/frontend/state-management.md:149`.

## Start mode (R9)

- Settings: `pub enum StartMode { Dual (default), Single }` in `DisplaySettings.start_mode`, TOML `display.start_mode = "dual" | "single"`. Unknown values warn and use `dual`.
- Policy: `policy::start_target(monitors, settings)`. `Single` returns the presenter monitor with `fullscreen = true, shared = true` at any monitor count. `Dual` delegates to `audience_target`. `Shell::start` and `start_reason` call `start_target`; Assign and Displays keep `audience_target`.
- UI: the trigger `start-mode` sits right of Start in `session-navigation` and shows the current mode with a chevron. `Intent::StartModeMenu` toggles `ContextualSurface::StartMode`; the menu (`start-mode-menu`, radio rows `start-mode-row-{i}`) is anchored bottom-left to the trigger. `Intent::SetStartMode` closes the menu and saves through the normal settings save. Both intents do nothing while busy or presenting; the trigger is not rendered while presenting.

## About tab (R10)

- `crates/ferrostage-ui/src/about.rs`: `VERSION`, `AUTHORS`, `LICENSE`, `REPOSITORY` from Cargo env (workspace `repository`); `BuildInfo { built_at }` set once by the app at start-up.
- `crates/ferrostage-app/build.rs` emits `FERROSTAGE_BUILD_EPOCH` (`SOURCE_DATE_EPOCH` or the current time). A missing value shows `about-unknown`.
- Build time renders in local time with the locale key `about-build-time-format` (zh-CN `%Y年%-m月%-d日 %H:%M`, en-US `%Y-%m-%d %H:%M`) and a separate `UTC%:z` tag.
- The repository row is an activation target `about-repository-link` (`Role::Link`); `Intent::OpenRepository` calls `cx.open_url`.
- `settings_panel::Category::About` renders read-only rows; no settings field.

## Error flag

`presenter-error` renders as a flag: error-colored border with a left accent, an "!" badge, the message, and a close button `dismiss-error` (`Intent::DismissError`). The intent clears the event errors `Shell.error` and `Shell.graphics_error`. Persistent conditions (monitor or watcher errors, texture upload errors) show the flag without a close button; they clear when the condition clears.

## Test limits

GPUI `TestWindow` keeps windowed bounds for a fullscreen request, so headless tests cannot confirm a fullscreen placement or exercise `Shell::start` (the headless shell has no topology watcher). The headless test injects the committed state and checks input, focus guard, crop and Stop. Placement, the skipped presenter toggle and commit activation are native checks (AC9).

## Risks

- GPUI Windows fullscreen on the same display as an existing window: placement confirmation reads back the actual display. A mismatch times out through the existing rollback path; the presenter is untouched.
- Focus stealing by Windows foreground rules: if `activate_window` does not take effect, keys still reach App-level actions only when the audience is active. The native check (AC9) must observe the active window.
- Version: 1.2.0 ships with the feature; no installer or packaging change.
