# V6 focused-node removal and persistence feedback

Root reproduced F7 and F8 in the frozen V5 native application. Settings click → Tab to Close → Enter removed the focused node, leaving region0 and no Tab movement; the same hazard appeared after Stop/Retry. A real FileShareRead lock let Swap finish at runtime while settings replacement failed; the error appeared twice, and the red copy survived a successful Retry. Root's native-v5-checkpoint.md and native-v5-evidence-manifest.json retain those observations. The source freeze was explicitly released for these two repairs.

Only crates/ferrostage-ui/src/presenter.rs and shell.rs change from V5. All other600 frozen inputs, including core, vendor, manifests, lock, policy, locale and DESIGN.md, are identical. product-v6-source-delta.json and the focused patch establish that boundary. No desktop automation or native launch was performed by this implementer.

## F7 repair

GPUI's automatically created button focus handle can remain selected after its element disappears from the dispatch tree. Modal return_focus adds a second lifetime issue: dropping its last owner immediately after window.focus leaves no live handle to validate.

Presenter now keeps the modal return handle alive until a deferred check owns it. After drawing and releasing the window/view lease, the check compares the selected handle with the captured one. A newer focus choice or explicit blur wins. If unchanged, it tests membership in the actual current Presenter/modal dispatch subtree and restores that subtree's persistent root only when the old node is absent. Valid buttons retain their focus. It does not activate a platform window or mutate Shell. The existing deferred action boundary and audience ownership remain unchanged.

The tests dispatch actual mouse/Tab/Enter/Space input through production elements and handlers, redraw, then verify working Tab traversal/actions. They cover Settings Close, Jump Commit, Stop and successful Retry, plus valid button retention, Audience activation, and focus/blur chosen after draw before the deferred callback. Start is unavailable in TestPlatform without a real topology watcher; no fake hardware readiness was introduced. The same render/removal mechanism applies, but actual Start-to-Stop focus continuity remains Root's native check.

## F8 repair

SettingsSession.save_error is now the single persistence feedback source. Shell::save retains its diagnostic warning but no longer copies that same error into the unrelated general Shell.error slot. Existing save_session owns dirty/save_error and clears them only after a successful write. The view's existing unsaved message and Retry therefore disappear together on success, while an unrelated current error remains.

Tests use the real SettingsStore in unique temporary directories. A blocking parent file produces a real create-directory failure; removing that fixture permits save and read-back. This is a deterministic persistence regression, not a replacement for Root's Windows FileShareRead-lock evidence. Coverage includes failed/repeated Retry, successful Retry, unrelated error retention, poll_import success with owned worker metadata followed by recent-file save failure, and the actual confirmed replacement transaction's role-commit/save-failure branch. Runtime document/roles remain committed; subsequent Retry is read back from disk. The worker metadata fixture does not claim PDFium execution, and TestPlatform replacement does not claim physical monitor placement.

## Commands and retained results

Commands were written RUNNING to product-status.txt before execution; each exit was recorded after return. The version command-status file is an immutable extraction of that ledger.

| Receipt | Actual result |
| --- | --- |
| product-v6-red.log | exit101, sandbox denied shared target lock; no test ran |
| product-v6-red-retry.log | exit101, new test fixture omitted MonitorBinding.friendly_name; no test ran |
| product-v6-red-2.log | exit101, fixture used Option instead of String; no test ran |
| product-v6-red-3.log | exit101,4 behavioral failures: modal/Stop focus, duplicate error, stale error after Retry;2 pass |
| product-v6-focus-red.log | exit101, save-feedback fix passes and3 focus-removal failures remain, including Retry |
| product-v6-green.log | exit101,46 pass; modal return still fails |
| product-v6-focus-red-final.log | exit101,3 focus failures against exact V5 Presenter with final traversal assertion;4 pass |
| product-v6-green-final.log | exit101,47 pass; modal return handle lifetime still fails |
| product-v6-green-2.log | exit0,48 pass after retaining return handle through validation |
| product-v6-headless-final.log | exit0,49 pass including final import/replacement save coverage |
| product-v6-just-ci.log | exit0,145 default tests and fmt/strict lint/default deny gates |
| product-v6-clippy.log | exit0, strict app/UI all-features/all-targets |
| product-v6-build.log | exit0, native gpui,pdfium app build |
| product-v6-diff-check.log | exit0 |

The initial modal test demanded Tab return specifically to Open. That was narrower than the contract to retain a valid return focus. It was corrected to require actual Tab movement and a dispatchable button; rerunning against the retained V5 Presenter still failed (None→None), so the correction did not remove the regression. The retained product-v6-focus-candidate.rs is the intermediate, incomplete first repair, not the final source. Use the frozen V6 source or focused final patch.

Final test command: `cargo test -p ferrostage-ui --features test-support --lib --locked --offline`. Strict consumer lint: `cargo clippy -p ferrostage-app -p ferrostage-ui --all-features --all-targets --locked --offline -- -D warnings`. Native build: `cargo build -p ferrostage-app --features gpui,pdfium --locked --offline`. The unchanged full dependency graph retains the previous all-features deny/core-tree/vendor evidence; no dependency or policy suppression was added. The documented upstream HarfRust direct-lint limitation remains unchanged.

## Frozen handoff

- product-native-freeze-v6.json SHA256: `b7df3a62ff2b1b7212924e47f79d72468ff86356265a1aaf650c0694adb6d7ea`.
- 602 source aggregate: `b71eb54d7da71f9b30ac5ab3bdb58fc1ada637bf2991adde10041f0a172170be`.
- Executable SHA256: `d4f2e653ee793176b384034797ad99e1ad665aa0da6535a454ba88a36b055074`,36,987,904 bytes.
- Cargo.lock SHA256: `2557e10b37fa5cfe63e78ce19965b8bbbd83438bdd540eed50405fbbc3dbc23e` (unchanged).
- Focused repair patch SHA256: `69bed99db29112e2a948fc76896fbc227b5d8776e034f1de17a41baca63d5300`.

Executable: `D:/mbx-cache/targets/v1/b228c6f37bcb27af7d1a2d6821dcfcb308164a0bd33ec7c00848c486151f07e0/debug/ferrostage.exe`. Use Root's existing isolated --config, verified --pdfium-library cache/pdfium-7881-win-x64/bin/pdfium.dll and optional PDF positional argument. Root was notified as soon as the final build/freeze completed. No command remains active in this implementation session; product code and binary remain frozen pending independent review/native feedback.

Root should repeat Settings→Tab Close→Enter→Tab, Jump Commit/Cancel→Tab, focused Start/Stop→Tab, and the actual locked-config Swap→unlock→Retry flow. Verify a single unsaved message before Retry, correct durable roles, no stale save message afterward, and usable keyboard traversal after Retry disappears. Existing valid focus and audience ownership must remain intact. These native assertions are pending for V6; prior V5 screenshots do not establish them. Physical unplug/port changes and unavailable modes remain NOT_RUN; Gate B remains incomplete. No commit/push/archive/journal/task switch/install/publish occurred.
