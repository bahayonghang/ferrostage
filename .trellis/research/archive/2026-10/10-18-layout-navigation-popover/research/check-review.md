# Combined integration check, 2026-10-18

Scope: approved parent task `10-18-presenter-navigation-options`, both children, complete tracked product diff and all six untracked product modules. This checker is the dispatched Trellis reviewer and reviewed/fixed directly. No recursive agents, GUI launch, commit, push, archive or branch changes were performed. Coordinator owns native acceptance, DESIGN/spec/task lifecycle updates.

## Findings (fixed)

- File: `crates/ferrostage-ui/src/presenter.rs`, regression in `crates/ferrostage-ui/src/shell.rs`.
- Issue: focus bookkeeping distinguished layout-menu entry from other surfaces, but treated Settings/Displays and PageBrowser as the same still-open state. Switching Settings to Jump rendered the numeric field without focusing it. Reopening Jump while a density/tile control owned focus similarly reset the edit state without returning focus to the field. A real GPUI-key regression reproduced the first case: typing `3` left the browser value at `1`, despite the browser being visible.
- Fix: retain the browser's previous per-open id in Presenter and handle changes alongside the existing layout-menu transition. New/reopened browser focuses its numeric editor and returns to progress on dismissal; reverse transition focuses the modal host. Ordinary repaint does not steal focus, because the id does not change. Added `page_browser_jump_refocuses_editor_after_modal_and_reopen`, exercising a real Settings-to-Jump transition, typing without a pointer click, density-control focus followed by repeated Jump, and final explicit Enter navigation. Editing does not change the committed page. Before/after receipts: `check-focus-before.log` (exit 101, failed assertion) and `check-focus-after.log` (exit 0, 1 passed).

The initial test-harness attempt polled activation before TestPlatform processed activation and was corrected to wait for the actual owner before polling. The retained failing receipt is the subsequent product-focus assertion failure, not the test setup error.

## Findings (not fixed)

No remaining concrete product-code defect was identified in the approved scope after repair. The normal-window native smoke is coordinator-owned and NOT_RUN by this checker; it must use the final executable below. This is a verification boundary, not a passing native result.

The earlier browser implementation's documented rich AccessKit text-run/caret/selection-action gap remains outside this menu integration fix. Ordinary TextInput role/label/value and keyboard/pointer editing are implemented. MenuItemRadio/MenuItemCheckBox roles with real toggled state and shared deferred activation are present; this source/build check does not claim screen-reader execution. No extra AX delivery qualification was invented.

Root DESIGN and frontend specs reflect requested/detected layout, host alignment, transactions, pure-core annotation query, private notes and preference compatibility. Coordinator should sync the new repeated-Jump/modal-to-browser editor-focus regression name and behavior. Checker did not edit coordinator-owned documents.

## Source audit

- `ContextualSurface` has one Shell-owned slot for browser/menu. Menu open carries an incrementing id and captured document generation. Deferred selection requires current id, generation and non-busy state; closing/reopening or replacing documents rejects retired selections.
- `shell/layout_navigation.rs`: identical complete overrides close before candidate construction or core mutation. Actual change builds a candidate, produces a descriptor, obtains core acceptance, assigns the document, retires frames, publishes and closes. Failure retains document, core/check state and frames; only localized local error changes. No override reaches SettingsStore.
- `presenter/layout_navigation.rs`: six radio rows map to None/None-layout/Left/Right/Top/Bottom, two independent checkbox rows flip only their own field, Auto separately displays effective detected layout. Rows reserve a check column and expose real menu semantics/toggled state. Localized document/structure warnings replace the old Debug dump; no raw true/false UI remains.
- Shared host uses BottomRight for the menu, BottomLeft for browser, distinct measured triggers, 8px upward gap and 12px clamp. Menu body scrolls internally. Existing geometry regression checks an ordinary 1100x760 window, all eight >=44px row targets, unchanged workbench bounds, width and actual upward gap. No size/DPI matrix was added.
- Menu Up/Down/Home/End/Tab routes focus before configured navigation; Enter/Space activate exactly once, held repeats are refused, Escape closes before Stop. Outside left click is consumed by the shared occluding backdrop. Browser/menu exclusivity, Stop/Home/Swap/import/replacement cleanup and valid focus recovery remain shared.
- Captured PresenterData drives both contextual surfaces and warning data. Render paths do not acquire Shell during synchronous first draw. Existing first-frame/window-lifetime/input regressions remain in the full GPUI suite.
- `PresentationState::has_annotations()` is a read-only pure-core query: live laser or any point-containing stroke on any page, false for empty strokes/retained empty vectors. Regression covers off-page ink, read-only behavior, erasure and laser. Actual reconfiguration still uses the original clear policy; identical menu selection preserves exact state/frames.
- Independent notes exclusion precedes physical overlay grouping. Reconfiguration updates audience only through the existing snapshot/crop boundary. Menu data/overrides/notes are not added to audience snapshots. Recognition remains explicit and independent.
- Browser editor, tile commit, preference persistence and scheduler internals rely on the earlier independent child check and were inspected at shared input/focus/host/lifetime boundaries. Closed browser drops UI thumbnail frames/targets; visible requests remain <=16. Legacy density/read-only/save failure/Retry and private-page regressions remain green.
- Cargo manifests, Cargo.lock, deny policy and vendor sources have no diff against HEAD. Therefore the coordinator's `integration-deny-all-features.log` successful full-feature policy receipt remains applicable. No dependency/source/license waiver was added.

## Verification

All final commands ran after the focus fix. Receipts below are in the parent `research/` directory unless noted otherwise.

- Lint: **PASS**. `just ci` includes `cargo fmt --all -- --check` and workspace all-target Clippy `-D warnings`. Final exit 0, `integration-ci-final.log`.
- Default TypeCheck/tests/policy: **PASS**. Final `just ci`, **198 tests passed**, 0 failed; advisories/bans/licenses/sources all OK. Earlier pre-fix baseline also passed, `integration-ci.log`.
- Native TypeCheck/build: **PASS**. `just check-shell`, exit 0, app gpui/pdfium build plus native all-target app/UI Clippy and **159 GPUI tests passed**, 0 failed, 0 ignored. `integration-shell-final.log`; test runtime 1.59s. Earlier pre-fix baseline had 158 passing tests, `integration-shell.log`.
- Combined UI strict lint: **PASS**. `cargo clippy -p ferrostage-ui --features gpui,pdfium,test-support --all-targets --locked -- -D warnings`, exit 0. `integration-ui-clippy-final.log`.
- Focus regression: **PASS**, 1 test after an observed failing product assertion. Child receipts `check-focus-before.log`, `check-focus-after.log`. Full 159-test suite rerun passed after this targeted repair.
- Real PDFium retention: **PASS**. Verified target/debug/pdfium.dll against docs/pdfium-runtime.json before setting FERROSTAGE_TEST_PDFIUM. `cargo test -p ferrostage-ui --features pdfium --lib --locked native_perf100_grid_retention -- --nocapture`, exit 0, **1 passed**, `integration-pdfium-final.log`.
- DLL SHA256: `79D4676B656CFB1ABCEA88F9ADE3B4B0826C5200382DB5F4EC72A636C598C118`.
- Real fixture ledger: old strip 4 thumbnails / 17,510,400 retained bytes; closed 0 / 16,588,800; open 16 / 20,275,200; closed again 0 / 16,588,800. These are retained-frame bytes, not CPU/FPS/RSS results.
- `git diff --check`: **PASS**, exit 0. Existing LF-to-CRLF warnings are informational. Existing unused atspi-connection patch metadata warning remains unchanged; no warning suppression was introduced.
- Native normal-window visual/live-effect acceptance: **NOT_RUN by checker**, coordinator owns the frozen build handoff.

## Frozen executable and source identity

- Base HEAD: `5ec8437e4ff82c3cb7a915250b428d79bd148e00`; approved changes remain uncommitted.
- Executable: `D:/Documents/Code/Rust/Exp/ferrostage/target/debug/ferrostage.exe`.
- Size: **43,604,992 bytes**.
- UTC modified: `2026-10-18T14:21:09.8241416Z`.
- SHA256: `35D8C54E16509067FBDA1AA2E3FA75D670583C85276D9B4280304C47A4E738E0`.
- Product/locale source hashes at this freeze: child `research/check-source-sha256.txt`.

All source repairs and required build/test commands are complete. No additional Cargo process is planned by this checker, so coordinator native launch will not collide with checker relinking. Final native dark/zh and light/en/private-note/page-browser observations should be recorded independently against this exact executable.

## Post-native test-isolation follow-up

The coordinator subsequently completed native acceptance on the frozen production code above and identified one test artifact: `crates/ferrostage-ui/unused-headless-settings.toml`. The new focus regression clicked the density control, which legitimately autosaves, while still using headless_shell's dummy working-directory store. That was a test isolation defect, not a product settings or navigation defect.

Surgical repair changes only three lines inside the cfg(test) GPUI regression `page_browser_jump_refocuses_editor_after_modal_and_reopen`: create the existing SaveFixture, unblock its normal save path, and assign `shell.store = fixture.store()` before the density edit. The fixture remains alive through the test and removes its unique temporary directory afterward. No unrelated fixture or production path was changed. The known generated artifact was resolved to the exact expected path inside the workspace before deleting only that file.

Verification after this test-only delta:

- Targeted focus regression: **PASS**, exit 0, 1 passed; `check-focus-isolated.log`.
- `just check-shell`: **PASS**, exit 0, native app build, strict all-target app/UI Clippy and **159 GPUI tests passed**, 0 failed; `check-shell-isolated.log`, suite runtime 1.51s.
- `cargo fmt --all -- --check` and `git diff --check`: **PASS**.
- Exact generated-artifact path absent after both targeted and full suite runs; it was not recreated.
- No additional native resmoke was performed or required for this cfg(test)-only change. The original native evidence and source hashes remain historical snapshots, not rewritten receipts.

The required build relinked the executable, so its file hash changed despite no production-code change. Current `target/debug/ferrostage.exe`: **43,604,992 bytes**, UTC modified `2026-10-18T14:31:13.4404444Z`, SHA256 `FBBE35BA06CE8E47BF16003CB6418330236197E2BA99AFF639C0868DDFCA8B53`. This identity supersedes the earlier identity only for the file currently on disk; coordinator owns parent report/identity ledgers and native provenance. No parent report, ledger, task lifecycle, commit or push was changed by this follow-up.
