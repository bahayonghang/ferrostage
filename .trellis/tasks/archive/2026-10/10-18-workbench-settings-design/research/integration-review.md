# Independent integration review

Review date: 2026-10-18. Reviewer: dispatched Trellis check agent. Scope: the complete tracked working-tree diff from `7c25e46d1868f5520b8c2225c2ab89acfe9cfbed`, plus untracked product files (none; untracked paths are the four task/artifact directories). Branch: `codex/workbench-settings-design`. The settings, workbench and displays children were reviewed together after their sequential implementation. No app was launched or automated by this reviewer.

Native feedback reopened five scoped repairs after the initial successful checkpoint below, including a later font18 recent-row containment issue. See `integration-native-repairs.md` for reproduction, fixes and the latest build identity. The initial hashes/gate counts in this report remain historical evidence for the coordinator's first native round.

## Findings (fixed)

### F1: font selection was skipped by Tab

- File: `crates/ferrostage-ui/src/presenter/settings_view.rs:514`.
- Issue: replacing the shared button focus handle with `font_picker_focus` removed Tab eligibility. Mouse selection and direct focus worked, which let the original mouse-first input test pass while keyboard users could not reach the selector.
- Fix: retain the explicit handle with `.clone().tab_stop(enabled)`, preserving both eligibility and disabled guards. Added a test-only focus probe at `presenter.rs:133` and changed the existing real-input regression at `shell/preferences_tests.rs:613` to reach the selector by actual Tab traversal and open it with Enter before filtering/selecting, persisting and resetting fonts.
- Reproduction: `cargo test -p ferrostage-ui --features test-support --lib settings_fonts_filter --locked` failed before the repair with `font family must be reachable by Tab` (0 passed, 1 failed). The same command passed after the repair (1 passed), and the final full GPUI suite also passed it. The initial test-probe draft tried to read a private Presenter field from the sibling test module; a narrow test-only method corrected that test compilation issue before reproduction.

### F2: two existing Close tests targeted the former footer position

- Files: `crates/ferrostage-ui/src/shell.rs:3470` and `:3927`.
- Issue: the new Settings header intentionally makes Close the first control. Two older regressions used Shift+Tab to reach the last control and therefore no longer activated Close. First integrated GPUI run: 133 passed, 2 failed. This was an obsolete test assumption, not a reason to move the accepted Close control back to the footer.
- Fix: change only the traversal direction to Tab. Preserve the actual Enter/Space priority, unchanged page/stage and post-close keyboard traversal assertions. Both focused failures passed after the correction, followed by all 135 GPUI tests.
- Original failure receipt: `integration-shell-first.log`. Final successful receipt: `integration-shell.log`.

## Findings (not fixed)

No remaining concrete code or spec-contract violation was identified in the reviewed diff. Native visual/OS behavior remains an acceptance gap assigned to the coordinator, not a passing result or a source defect. The exact remaining checks are listed below.

## Cross-layer and ownership review

- Font leaves update v1 schema/defaults, leaf parser, authoritative size range, known-key merge and the documented example together. Wrong leaf values retain loader fallback/diagnostics; interactive edits reject invalid values. Font installation stays out of settings IO. Unknown TOML keys, future read-only policy and failed-save effective-value retention are covered by real parser/store tests.
- Installed-name enumeration is cached by Shell, loaded on picker entry or before first draw for a saved override. `Typography::resolve` retains a missing requested family and resolves the UI fallback. Captured Presenter data and tooltips consume role-scaled typography; the home Georgia wordmark and mono timer retain their roles. No audience, core, raster, PDF, media, platform, dependency or vendor production file changed.
- Existing preference application, GPU/cache preflight, runtime rollback and save/Retry remain the mutation path. Font filtering does not save. The picker participates in modal key capture and returns focus to its invoker when closed. The review fixed its previously missing Tab eligibility.
- Recent settings rows use actual basename/parent/full path and preserve the complete configured list, order, limit and shared history clear. They do not gain the home top-three cap or import action. All seven categories continue through the common row/nav/header and existing editing/recording/reset paths.
- `tools_panel` owns one Start/Stop action and the navigation/progress group, including no-step and busy cases. The old independent full-width session footer is removed. Compact progress retains full detail and original Jump intent; focus detail is an overlay so it cannot move the activation target. Page editing, notes privacy, timers, blanking/tool semantics and audience lifecycle retain their authorities.
- Window observations belong to exact native handles (`WindowRecord.observed_display`), not requested monitor fields. Creation/bounds/placement reads and existing tick reads update samples. Failed current-handle reads clear the sample. Only current committed presenter/audience records enter `RoleProjection`; retired/replacement candidates cannot publish actual roles. Mapping must be valid and uniquely reversible to a unique current MonitorId. Missing/ambiguous observations stay Unknown. Ready/Idle PDF is NotRunning. No observation writes settings.
- Diagram, callouts, summary and rows use the same role projection and numbering. Configuration and pending allocation remain separate from observation. Selection does not move windows; Apply uses existing confirmed transaction/rollback; Settings Cancel discards allocation only and preserves ordinary saved preferences. Missing/degraded/stale warnings are outside diagnostic disclosure. Settings and standalone Displays share a body with one scroll owned by each host; no fixed 650/450px embed remains.
- Existing deferred input dispatch, first-window drawing under a Shell lease, removal/close observation and current-versus-retired window checks remain. Full GPUI regressions exercise those lease/lifecycle boundaries; this review did not add platform patches or weaken guards.
- Root DESIGN and the updated frontend/backend specs match these boundaries. No new spec abstraction or unrequested test matrix was introduced.

## Acceptance mapping

“Source/input verified” below includes current code review and passing focused/full tests. It does not mean native screenshots or OS interaction passed.

| Parent / child criteria | Current evidence | Remaining coordinator native proof |
| --- | --- | --- |
| AC1, W1 | `presenter.rs::command_bar`, `tools_panel`; real Jump/edit/confirm/private-note regression and stable focus-detail geometry | Ordinary Ready/Presenting hierarchy, concise progress, pointer/focus full context, Jump |
| AC2, W2 | Single session action, no independent footer; `workbench_session_strip_survives_empty_steps_and_busy_start`; strict build | Recognizable play/stop icon, same-row navigation, preparing/disabled feedback |
| AC3, W3-W4 | Actual-state check indicators/swatches, original Intents, unchanged split/image paths; workbench input, pointer, notes, timer and lifecycle regressions | Tool/blank selection, Previous/Next, Start/Stop, Home and workspace visual balance |
| AC4, S1/D1 | All-category regression; shared display body/two-host ordinary three-screen fit test; removed nested scroll/embed | Seven categories remain usable; both hosts' single scrolling and compact visual composition |
| AC5, S2-S3 | Live captured typography after real Tab/Enter/filter/size/default input; real saved readback; resolver missing-family test | Installed native selection, size/default/fallback appearance, relaunch persistence, mixed CJK/Latin |
| AC6, S3-S4 | 42 settings tests, checked edits, future policy, unknown-key/store/save-failure tests; audience/PDF/raster boundary source review | Font changes preserve native document/audience appearance and distinct home/timer roles |
| AC7, S5 | Real basename/parent and full-path rows; count/limit/empty paths; history model and real clear/save test | Ordinary General history/empty/Clear appearance and home synchronization |
| AC8, D1-D2 | Contain-fit helpers and source; both-host 1400x900 three-screen test sees summary, three rows and Apply/Cancel | Actual current three-screen overview/rows visible together, readable role/number agreement |
| AC9, D3 | Pure unique/unknown mapping tests; real GPUI move-callback/current-owner test; exact-handle sample review | Windows manual window move changes actual markers without saving allocation |
| AC10, D4-D5 | Draft/actual/configured model separation; Apply/failure/Cancel/stale/read-only/no-play/single-screen/retired-owner regressions | Draft selection/Cancel and valid Apply confirmation; Ready versus Presenting actual role labels |
| AC11, S4/D4-D5 | Scoped saved/pending text; shared save/error/Retry and transaction regressions; warnings outside disclosure | Saved preference versus pending allocation is unambiguous; visible warning/recovery copy |
| AC12, S6/W5/D6 | Final source hashes, required gates and logs below; no added size/DPI/performance/hotplug matrix | Final executable-bound ordinary-window core/visual receipt, relevant light/dark and Chinese/English, clean close logs |

The coordinator must bind new native results to this executable and compare the existing task-owned `research/native/before-*.jpg` baseline. User attachments and headless GPUI captures are not native after evidence. Use isolated task-owned config and fixtures; do not modify user PDF/personal settings. No minimum-size, DPI, performance or physical-hotplug matrix is required by this task.

## Verification

| Check | Final result / receipt |
| --- | --- |
| Format | PASS: `just ci` runs `cargo fmt --all -- --check` |
| Default lint | PASS: `cargo clippy --workspace --all-targets -- -D warnings` |
| Default tests | PASS: 194 tests across workspace suites; includes 42 settings and 38 UI tests. Native-feature-specific test binaries with zero enabled tests are not counted as native verification. |
| Dependency policy | PASS: `cargo deny check`, `advisories ok, bans ok, licenses ok, sources ok`; existing unmatched optional license exceptions and duplicate-version warnings remain warnings, with no waiver/dependency edits |
| Native application build / typecheck | PASS: `cargo build -p ferrostage-app --features gpui,pdfium --locked` |
| Native strict lint | PASS: `cargo clippy -p ferrostage-app -p ferrostage-ui --features gpui,pdfium --all-targets --locked -- -D warnings` |
| Full GPUI tests | PASS: `cargo test -p ferrostage-ui --features test-support --lib --locked`, 135 passed, 0 failed |
| Whitespace | PASS: `git diff --check` |
| Native desktop capture/input | NOT_RUN by this reviewer, explicitly coordinator-owned after this checkpoint |

`integration-ci.log` is the complete successful default CI receipt. `integration-shell-first.log` retains the initial two obsolete Close-order failures. `integration-shell.log` is the successful final complete native gate. Only two GPUI test traversal strings changed after successful default CI, so the unchanged default gate was not repeated; final check-shell rebuilt/re-linted the changed UI source and ran the complete feature-enabled suite. `integration-diff-check.log` records the whitespace check (empty is success). Existing LF/CRLF conversion notices are Git warnings, not whitespace failures.

The first sandboxed Cargo invocation was unable to access the existing mbx cache at `D:/mbx-cache`; authorized elevated tool calls ran the checks successfully. This environment failure was not counted as a product test failure. No dependency migration, warning suppression or cache restructuring was used.

## Initial source and executable handoff

- Base: `7c25e46d1868f5520b8c2225c2ab89acfe9cfbed`, branch `codex/workbench-settings-design`, uncommitted working tree.
- Per-file hashes of all changed product sources/locales and root DESIGN: `integration-source-manifest.json`, recorded UTC `2026-10-18T09:47:01.0698644Z`.
- Manifest SHA-256: `0c86fa3fe7f7a4bf50c76c939e502f92e67430d0cd0725bd7ea70282c580f677`.
- Executable: `target/debug/ferrostage.exe`, 43,292,672 bytes, SHA-256 `5ce61a3feb7a87ae3931e4a1f8cc093a65ee4eb16cdf6fca8d17ac5137ec8e4f`.
- Existing staged `target/debug/pdfium.dll`: SHA-256 `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`, matching the pinned runtime.
- Source/build handed to coordinator and frozen for its first native round. Later native feedback reopened the scoped repairs documented separately. No commit, push, archive or reviewer desktop launch. Acceptance remains open until final native results are attached.
