# Implementation plan

## Order

1. Add locale keys for the changelog button label and the five paragraphs in `assets/locales/zh-CN/app.toml` and `assets/locales/en-US/app.toml`. Copy is the PRD table, one paragraph per key.
2. Add `PanelState.changelog_open` and `Intent::ToggleChangelog`. Handle it in `dispatch_preference`. Clear it on category change and `dismiss_preferences`.
3. Render the About button and, when open, the newest-first version list in `settings_view.rs`. Keep the five existing rows and `about-repository-link`.
4. Add the completed-left-click helper and attach it to the current-slide body, next-slide body, and `note_panel` in both `workbench_panel` branches. On the interactive current picture, stop the click for non-Navigate tools and for media-slot hits.
5. Replace `audience_click_does_not_bind_the_presenter_slide_preview` with coverage for AC1–AC3: current, next, and notes clicks; mouse-down and outside release; pen and media; Settings modal; last step; audience toggle left untouched.
6. Extend the About test for AC4: button, both locales' 1.2.1 and 1.0.0 paragraphs, toggle, category change, no `opened_url` from the changelog button. Repository link still opens the repo.
7. Set workspace version to 1.2.1, update FerroStage `Cargo.lock` records only, and change the `about::VERSION` assertion to `1.2.1`.

## Validation

```text
cargo clippy -p ferrostage-ui --features test-support --all-targets --locked -- -D warnings
cargo test -p ferrostage-ui --features test-support --lib --locked
cargo metadata --format-version 1 --locked --no-deps
```

`cargo metadata` must show FerroStage packages at 1.2.1. `just ci` does not compile the `gpui` presenter; the test-support commands above are the required check. Do not run `just release`.

## Rollback points

- After step 4, presenter clicks can be removed without touching About.
- After step 6, the About flag can be removed without a settings migration.
- The version edit is the last content change so a failed UI test does not leave a bumped lockfile as the only delta.

## Review before start

`prd.md`, `design.md`, and this file are the implementation gate. Do not start until the user approves this planning summary.
