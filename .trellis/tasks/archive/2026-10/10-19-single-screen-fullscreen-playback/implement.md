# Implement: single-screen fullscreen playback

## Checklist

1. [x] Version: `Cargo.toml` workspace version `1.2.0`; regenerate workspace entries in `Cargo.lock` with `cargo update --workspace --offline`. Verify with `cargo metadata --locked --offline --format-version 1`.
2. [x] Settings (`crates/ferrostage-settings`): replace `PresenterOnly` with `FullscreenAudience`, add the `presenter_only` parse alias, change the default, update parse/merge tests (AC5).
3. [x] Policy (`crates/ferrostage-ui/src/policy.rs`): `AudienceTarget` struct, single-monitor table, update callers in `shell.rs` and `display_settings.rs`; policy tests (AC1).
4. [x] Start transaction (`shell.rs`): `open(.., focus)`, skip presenter fullscreen for shared fullscreen, derived `single_screen_playback()`, activate audience window; GPUI tests (AC2, AC3).
5. [x] Focus guard: `request_focus` returns early during single-screen playback; GPUI test (AC3).
6. [x] Notes isolation test with a right-split notes fixture (AC4).
7. [x] Displays dialog: `PdfTarget::SameScreenFullscreen`, two choices on one monitor, validation, settings mapping; tests (AC6).
8. [x] Locale: `no-audience` text, `displays-role-fullscreen`, generalized `displays-windowed-single` in en-US and zh-CN.
9. [x] Docs and spec: ADR-006, ADR-007, `docs/architecture.md`, `docs/PROJECT_PLAN.md`, `.trellis/spec/frontend/state-management.md`.
10. [ ] Native check (AC9) with `just dev` on one monitor / Duplicate mode, or record NOT_RUN.
11. [x] Start mode: `StartMode { Dual, Single }` in `DisplaySettings` (`display.start_mode`, default `dual`), `policy::start_target`, `Shell::start` / `start_reason` use it; `Intent::StartModeMenu` / `SetStartMode`, `ContextualSurface::StartMode`, `presenter/start_mode.rs` menu, trigger button right of Start; tests (AC10).
12. [x] About tab: `about.rs` (`VERSION`, `AUTHORS`, `LICENSE`, `REPOSITORY`, `BuildInfo`), `ferrostage-app/build.rs` emits `FERROSTAGE_BUILD_EPOCH`, `Category::About` rows with build-time format and repository link; locale keys; test (AC11).

## Validation commands

```bash
cargo test -p ferrostage-settings
cargo test -p ferrostage-ui --features test-support
just ci
```

`just ci` runs default features without the PDFium DLL. GPUI shell tests need the feature set the existing shell tests use; check `crates/ferrostage-ui/Cargo.toml` before running.

## Review gates

- After step 4: review the flag lifetime against every `self.audience` assignment (`grep -n "self.audience" crates/ferrostage-ui/src/shell.rs`).
- Before commit: `just ci` output saved under `research/`.

## Rollback points

- Steps 2-8 are one feature commit; revert restores 1.1.1 behavior. Settings written by 1.2.0 fall back to the 1.1.x default with a warning (see design).
- Step 1 is independent and can stay if the feature is reverted.
