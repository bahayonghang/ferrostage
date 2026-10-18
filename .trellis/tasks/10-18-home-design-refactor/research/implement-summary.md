# Home implementation report

Implementation frozen for independent check and native QA on 2026-10-18. This report establishes source/build/headless regression results, not native visual acceptance.

## Files and behavior

- `crates/ferrostage-ui/src/presenter/home_view.rs`: dedicated brand, bounded home composition, quiet utilities, one natural-width primary action, two-line recent-file rows, empty history, full-path tooltip/accessibility label and keyboard-focused path detail.
- `crates/ferrostage-ui/src/presenter.rs`: Idle title and chrome/footer omission; common feedback retained after home content; shared activation/focus/disabled/input boundary extracted without changing existing button faces. Home-only retained focus handles and GPUI scroll anchors make offscreen controls reachable by keyboard at the actual minimum window size.
- `crates/ferrostage-ui/src/theme.rs`: 760px home width and dedicated Georgia semibold home brand font. Existing global control size, palettes and button styles remain unchanged.
- `assets/locales/en-US/app.toml`, `assets/locales/zh-CN/app.toml`: concise welcome instruction plus empty-history and accessible recent-path copy, matching interpolation keys.
- `crates/ferrostage-ui/src/shell.rs`: extended Idle tree regression, all requested logical geometry probes across locales/themes and 0/1/3/4 saved entries, plus actual minimum-size modal/focus/Tab/Shift+Tab/Space/chooser-cancel regression.
- Root DESIGN, task status/artifacts and frontend spec updates are owned by the main session.

## Validation

- PASS: `cargo test -p ferrostage-ui --features test-support --lib --locked -- --nocapture`, 125 tests, 0 failures. Log: `research/implement-ui-tests.log`.
- PASS: `cargo build -p ferrostage-app --features gpui,pdfium --locked`, 8.56 seconds. Log: `research/implement-native-build.log`.
- PASS: `cargo fmt --all` followed by formatted source in the successful test/build, plus `git diff --check`.
- Full default CI, strict native-feature lint/check-shell and all-features deny are assigned to the independent check agent. Main owns native visual/interaction evidence.

The initial sandboxed Cargo invocation failed at the external mbx cache/session boundary before compiling application code. Re-running with cache access succeeded. Earlier focused compile/test failures are preserved in `implement-focused.log`, `implement-home-tests.log` and `implement-keyboard-test.log`; the complete final suite is `implement-ui-tests.log`.

### Measured geometry

The GPUI geometry test logs 176 actual view renders: 11 window probes x 2 locales x 2 themes x 4 saved-entry counts. All passed column/face/target/row/gap checks. The fourth saved entry is deliberately excluded by the existing three-entry capture limit.

- At 1100x760: column x=170, width=760; Open target x=170, width=128, height=52; Open face x=174, width=120, height=44. Brand container shares the column's x coordinate and applies 4px internal inset to align its ink with the Open face and file text.
- At 1440x900: column x=340, width=760; Open target width=128 and face width=120.
- At 320px client width: column x=16, width=288. Utilities occupy their own row. Recent targets remain 52px high with 4px gaps.
- At 320x240: actual keyboard navigation reaches a recent-file target and asserts its complete target is inside the 240px viewport after the scheduled GPUI scroll frame. Keyboard focus also exposes the full-path detail; Shift+Tab hides it and reaches Open, Space opens the chooser, and cancellation leaves Idle usable.

### Focus fix and test fidelity

The pinned GPUI applies `.tab_stop()` on an element only when creating an automatic focus handle. An explicitly tracked handle must itself be marked as a tab stop. The first regression exposed that difference; retained home handles now receive their enabled state directly. A second failure exposed offscreen focus, resolved using existing GPUI `ScrollAnchor`, without changing platform code or weakening the target visibility assertion. The test drives a scheduled platform frame because the anchor applies offset on the next frame.

## Shared-consumer inventory

Reviewed four existing source consumer groups, comprising 48 original `Presenter::button` call sites: presenter composition 12, workbench 15, preferences 14, display settings 7. The two original home-only button sites (Open and recent loop) now use home-specific faces; the other 46 keep the original shared face implementation. The presenter command bar's Settings/Displays still use the existing face outside Idle. Idle uses separate quiet utility faces with the same stable IDs and shared activation behavior.

View-family accounting: **checked 5 / changed 1 / left 4**. Families: Idle home changed; document command bar/footer/jump, workbench, preferences, and display settings preserved. The 125-test suite includes the existing notes/workbench geometry, direct navigation, audience lifecycle, activation, busy guards and focus-recovery regressions. These are headless interaction/geometry checks, not screenshots.

## Source/executable binding and follow-up

Native executable: `target/debug/ferrostage.exe`, SHA256 `C1035E7A24FB955CFAA30E5F48ED24778511D95FDA692EF1CAF77052AD42313E`, built successfully from the frozen implementation. Main notified before native capture; later code edits would invalidate this identity.

Independent check should examine focused long-path wrapping and accessibility dispatch in addition to the existing column/row geometry coverage. Native typography, theme/locale appearance, loading/error/Retry states, OS scaling and the full requested screenshot matrix remain the main/check acceptance work. No commit, push or archive performed.
