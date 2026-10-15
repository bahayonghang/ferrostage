# Independent v6 check — F7/F8

**Disposition: ready for Root's targeted native retest.** Source and retained automated evidence support the scoped F7/F8 repairs; no additional product defect requiring a source change was found. This is not task07/Gate B acceptance. Native v6 assertions remain pending, physical topology cases remain NOT_RUN, and tasks08–11 remain blocked. The prior v5 independent review/manifest are unchanged.

The reviewer read current task context, PRD/design/implementation updates, the retained native-v5 checkpoint, v5/v6 frozen sources, focused diffs, tests and logs, pinned GPUI focus/defer implementation, and SettingsStore save semantics. Only new independent review/audit/index artifacts were written. No Cargo command, product edit, executable rebuild, native launch, desktop action, nested agent, commit/push/archive/journal operation was performed.

## Findings (fixed by implementation; independently reviewed)

### F7 — Disappearing focused controls

File: `crates/ferrostage-ui/src/presenter.rs`, modal return/focus recovery in `Render`.

The v5 native Settings Close and Stop paths could remove the selected button while leaving no traversable focus. Retaining a FocusHandle does not establish that its node is still present. The v6 repair retains the modal return handle long enough to capture the current focus, then checks it after rendering with `Window::defer`. A newer focus selection or explicit blur wins because the callback compares the current handle against the captured handle. If unchanged, `contains_focused` checks membership in the current persistent Presenter/modal root; a missing old node falls back to that same root. A valid control remains selected.

This follows actual pinned APIs: GPUI `FocusId::contains_focused` and `contains` consult the **most recently rendered dispatch tree** (`vendor/gpui/src/window.rs:504,517`); `Window::defer` runs at the end of the effect cycle after leased entities/windows can return (`2503`). `Window::focus` changes internal focus/keystroke state and refreshes the window (`2302`); this repair does not request platform activation or update Shell. Its weak Presenter reference and window-scoped deferred callback cannot redirect recovery to Audience or a replacement Presenter.

Reviewed real TestPlatform mouse/key/draw paths cover Settings Close, Jump Commit, Stop, successful Retry disappearance, valid focused controls, a newer choice after draw, explicit blur and unchanged Audience activation. These are not tests that call the new focus helper directly. The final Close assertion requires actual Tab movement and an available button action; it does not require Open specifically when another valid return control exists. The corrected assertion was rerun against retained v5 Presenter and still failed (`None → None`). Thus the assertion correction preserved the real regression.

**Coverage limit:** there is no direct automatic focused-Start→Presenting→Tab regression: TestPlatform lacks the real topology watcher required to enable Start. Commit is covered but already passed in the earlier red run; do not call every covered route a reproduced failure. Root explicitly selected targeted native **Start activation → Tab to a valid control → Shift+Tab to Stop → Enter**, without modifying this freeze or claiming Start automated red→green. The source uses the same node-removal recovery, but that inference is not native proof.

### F8 — Duplicate/stale persistence error

File: `crates/ferrostage-ui/src/shell.rs`, `Shell::save`.

V5 copied a failed save into both `SettingsSession.save_error` and generic `Shell.error`. A successful Retry cleared only the settings copy, leaving stale red text. V6 retains the diagnostic warning but removes the duplicate assignment. SettingsStore remains the single save-feedback owner: `save_session` retains runtime values and sets `dirty/save_error` on failure; successful save clears both (`crates/ferrostage-settings/src/store.rs:234–255`). Presenter already renders that owner once alongside Retry. Unrelated generic errors are neither overwritten by failure nor erased by successful Retry.

The tests use the real SettingsStore and unique temporary directories, not synthetic SaveError values. A parent path occupied by a regular file causes an actual filesystem failure; removing that file permits actual save and `store.load()` read-back. Coverage verifies repeated failure, success, unrelated-error preservation, `poll_import` commit/recent-file save failure, confirmed replacement transaction role commit/save failure, retained runtime document/roles, and durable role/recent-file read-back after Retry. The Retry disappearance test uses actual Tab/Enter and checks subsequent traversal.

Boundary: import receives constructed owned worker metadata and does not run PDFium; the replacement test supplies a pending transaction and exercises the actual confirmation/commit/save branch, not physical mapping. Deterministic parent-path failure is distinct from Root's native Windows FileShareRead lock test. That native v5 test proved runtime migration and durable Retry, while its feedback failed; v6 must repeat the feedback and focus assertions.

## Findings (not fixed / not claimed)

- Start focus continuity requires the planned native retest; absence of its dedicated automated case is a coverage limit, not a newly reproduced product defect.
- Generic retired-window `window not found` / invalid-handle diagnostics remain unlocalized. No vendor change, suppression or waiver was introduced. Source-level focus recovery does not establish these logs harmless or fixed.
- Native v5 evidence contains excluded harness runs: invalid CRCRLF TOML preparation and an overlapping system-profile launch. English provisional resize filenames did not establish requested dimensions. The checkpoint explicitly excludes these from acceptance; file integrity does not make them passes.
- Successful native chooser import, live OS theme switching, full size matrix and physical unplug/port/single/zero-display requirements retain their recorded limitations. No test fixture or no-Audience configuration substitutes for physical evidence.

## Verification and source binding

The independent command `python .trellis/tasks/10-14-07-gpui-shell/research/product-independent-v6-audit.py` returned **0**. Its JSON records exact hashes and comparisons; all reads completed:

- V6 freeze SHA256 `b7df3a62ff2b1b7212924e47f79d72468ff86356265a1aaf650c0694adb6d7ea`.
- Source dictionary SHA256 `b71eb54d7da71f9b30ac5ab3bdb58fc1ada637bf2991adde10041f0a172170be`; **602/602 match** current files. Only Presenter and Shell differ from v5; all other600 match. The retained before-files match their exact v5 hashes. All94 current crate files are covered.
- Executable SHA256 `d4f2e653ee793176b384034797ad99e1ad665aa0da6535a454ba88a36b055074`, independently matched. Freeze-bound build/clippy logs, Cargo.lock, PDFium DLL and fixture also match.
- Native v5 manifest SHA256 `ddfec0154c3dfe6731eb08b2c5dcf4b9b27c2a49ccfd54589a05f2ed295d73d0`: **108/108 records match bytes and hashes**.
- Implementation manifest SHA256 `dac5512c8887bbb3278e0b9c68fc8a622731fbc40eea187aa7dfef090a40912c`: **26/26 records match bytes and hashes**.
- Adopted vendor comparison: **498/498 match**. Dependency manifests/features/lock/deny/CI are unchanged, so earlier full all-feature deny and maintenance-source evidence remain applicable. Only the approved three exact license exceptions remain; direct HarfRust upstream lint limitations are unchanged.

Reviewed commands were executed by the implementation owner, not rerun by the reviewer:

| Verification | Result and receipt |
|---|---|
| `cargo test -p ferrostage-ui --features test-support --lib v6_ --locked --offline` | Actual behavioral red: `product-v6-red-3.log`, exit101, four failures/two passes; isolated focus red `product-v6-focus-red-final.log`, exit101, three failures/four passes. |
| `cargo test -p ferrostage-ui --features test-support --lib --locked --offline` | `product-v6-headless-final.log`, exit0, **49 passed, zero failed/ignored**. Includes the prior41 tests plus eight v6 tests; do not add previous overlapping runs. |
| `just ci` | `product-v6-just-ci.log`, exit0: fmt, default strict lint, **145 tests** (independently summed), default deny PASS. |
| `cargo clippy -p ferrostage-app -p ferrostage-ui --all-features --all-targets --locked --offline -- -D warnings` | `product-v6-clippy.log`, exit0. Consumer lint PASS; disclosed dependency warning remains, not direct upstream strict-lint PASS. |
| `cargo build -p ferrostage-app --features gpui,pdfium --locked --offline` | `product-v6-build.log`, exit0. Native-feature compilation/typecheck PASS; native behavior pending. |
| `git diff --check` | `product-v6-diff-check.log` and command ledger, exit0. |

Failure history is preserved and classified: `red.log` was sandbox target-lock denial; `red-retry.log` and `red-2.log` were new test-fixture compilation errors, so no behavior ran. `green.log` and `green-final.log` actually returned101 with a surviving modal focus failure; their names do not make them passes. `green-2.log` returned0 with48 tests; the final49-test receipt supersedes it. The intermediate `focus-candidate.rs` is not the frozen final implementation.

The initial review read attempted `product-v6-repair.md` before the implementer finished that artifact; the later successful read and verified implementation manifest are authoritative. No missing-artifact read was interpreted as a product failure or evidence pass.

## Native handoff

Root may now run the existing targeted native-v6 protocol against this exact freeze: Settings Close/Commit/Cancel focus traversal; focused Start→Tab and Stop→Tab without page movement; locked-config Swap→unlock→focused Retry with exactly one save message, matching durable roles and restored traversal; retain valid control and Audience ownership. V5 native results can describe unchanged paths only with their original identity. V6 fixes need current native observations.

The reviewer has no active command, build or owned native process. The product stays frozen, Root remains sole desktop controller, and no overall task07/Gate B completion is asserted.
