# Integrated review: presenter notes and manual navigation

Reviewed 2026-10-17 against task PRD/design, root DESIGN.md and curated frontend/backend contracts. Scope is the complete working-tree diff against HEAD, including core, render, UI, settings, app, locales and documentation. No dependency pin, Cargo.lock, policy or maintained vendor change is present.

## Findings (fixed)

- **Thumbnail selection committed on mouse-down.** The new tile ancestor dispatched JumpTo on mouse-down while its nested shared page button also owned a click handler. A user could not cancel a press by moving outside before release. In `crates/ferrostage-ui/src/presenter/workbench_view.rs`, moved the image action to a stable image element's click handler, separate from the sibling page button. The shell regression now exercises hover, down without release, cancelled release, successful image click, successful button click and following keyboard navigation. `thumbnail-click-check.log`: PASS.
- **Timer regression did not exercise the shell poller.** The original new core test only called immutable elapsed_at. Added `repeated_shell_ticks_never_start_or_advance_a_presentation` in `crates/ferrostage-ui/src/shell.rs`, covering repeated real Shell::tick calls in Ready and Presenting on both sides of the countdown target, then actual PageDown/PageUp dispatch. This guards the route implicated by automatic-advance reports without inventing timer-driven navigation code.
- **Public docs/spec retained removed preview behavior.** Updated `README.md`, `docs/architecture.md`, `docs/beamer-notes.md` and `.trellis/spec/backend/directory-structure.md` to describe direct content-page selection and the remaining render priorities. Architecture explicitly notes that the old project-plan preview priority was superseded. Historical plan text remains historical.
- **Feature-only dead field.** Native strict clippy exposed PresenterData.current after preview removal; the implement agent removed the field and derived its test probe from the existing workbench overlay. No lint waiver was introduced.

## Findings (not fixed)

- **Full-feature dependency gate fails on existing dependencies.** `deny-all-features.log` reports RUSTSEC-2026-0371 (zbus_names), RUSTSEC-2026-0370 (zvariant) and RUSTSEC-2026-0372 (zvariant_derive), with advisories FAILED and bans/licenses/sources OK. Dependency upgrades and policy exceptions are outside this task's approved scope; recommend a separately reviewed maintained-GPUI dependency decision. Do not label the full product gate green or add waivers.
- **Native visual acceptance remains incomplete.** The coordinator obtained successful Windows presenter placement and the user confirmed a visible window. Computer Use could not provide reliable capture/input control. The light/dark, sizes, long-label, focus/hover/press, notes scrolling/freshness and physical audience isolation matrix is NOT_RUN/NOT_VERIFIED on this revision. Headless GPUI bounds/input tests and PDFium tests do not substitute for screenshots and desktop interaction. Keep the task open for that evidence.
- **Automatic timed advance not reproduced natively.** Source tracing shows no timer-to-navigation path; the removed thumbnail hover handler explains pointer-triggered unsolicited preview changes. Repeated shell ticks are now tested, but no claim is made that a native timed-advance defect was reproduced.
- **Performance scope is bounded.** Notification suppression was explicitly deferred in design because snapshot equality could compare pixel buffers at 25 ms cadence. Real PDFium perf-100 retained-frame/request measurements demonstrate thumbnail work removal, not CPU/FPS/latency improvement. The four-thumbnail baseline is a request-model comparison, not an old-executable benchmark.

## Verified paths

- Core JumpTo validates physical bounds/content map before mutation, updates physical/logical positions together and preserves stage/timer. Changed committed page uses existing active-stroke/laser invalidation and media context synchronization.
- Dialog typing stays local, invalid/private page submission does not navigate, Enter/Go submit directly, Escape/Stop/Swap discard input. Removed preview commands/state/render priority have no executable consumers left.
- Notes precede next slide; default 0.65 and legacy effective range 0.55–0.8 agree across view, divider and settings UI. The permanent strip is removed and recent files are Idle-only. Labels and semantic button roles use the existing palette and contrast tests.
- Closed-grid scheduling ignores legacy show_thumbnail_strip=true; opening stays bounded and closing retires workbench thumbnail frames. A repeated unchanged schedule submits nothing. Real perf-100 counts/bytes are in `perf100-native.log`; these bytes are retained-frame ledger values, not whole-process memory.
- Default logging uses warn; isolated subscriber test proves routine silence and explicit diagnostic override while retaining warnings/errors. Startup failures remain observable.

## Verification

- Formatting and whitespace: `cargo fmt --all -- --check`, `git diff --check` PASS after reviewer edits.
- Default CI: `ci.log` PASS (fmt, strict clippy, workspace tests, default dependency policy). Later reviewer changes are GPUI-only code/tests and docs.
- Native type/build/lint and GPUI suite: `check-shell-reviewed.log` PASS after all review edits: GPUI/PDFium app build, strict feature clippy, all 123 GPUI tests including tick and click-cancellation regressions. Final executable SHA256: `50C3EB632630493B1325C644FC9C373BB6CC4B297812DCB23E86AF6728CDD0B0`.
- Native PDFium: `native-tests.log` PASS, 40 tests; `perf100-native.log` PASS, one explicit perf-100 resource regression. These paths were unchanged by the final click fix.
- Focused final click regression: `thumbnail-click-check.log` PASS, 1 test with 122 filtered out.
- Full-feature dependency policy: FAIL, existing advisories above.
- Native screenshot/interaction matrix: NOT_RUN/NOT_VERIFIED as above.

No unresolved task-local code defect was found after the fixes. Product completion remains bounded by the explicit policy and native acceptance gaps.
