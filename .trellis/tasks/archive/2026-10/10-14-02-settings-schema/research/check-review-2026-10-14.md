# Task 02 independent check — 2026-10-14

Result: PASS after one local fix. Task 02 implementation and independent verification are complete; changes remain uncommitted. No remaining task 02 blocker. Parent Gate A still needs 04/03 and is not claimed here.

## Scope and evidence applicability

- Reviewed the full native injected context from its saved hook output, task PRD/design/implementation plan, backend specs, parent shared contracts and requirement traceability.
- Reviewed all settings schema, parse, merge, store/path/file modules and tests; core MonitorRole and dependency direction; app startup/check-config paths and integration tests; packaging/spec alignment.
- Branch `main`, HEAD `ab78a84d96c7cf4a0bbc8dfd4f8e45da180bafb3`. Before review edits all 47 hashes in `verified-source-sha256.json` matched current files. The implementation's 57-test CI, Windows protocol, CLI probe and dependency trees were applicable to that state.
- Review changes to production/test code are only `parse/controls.rs` and `tests/parse.rs`. New post-fix evidence is `check-ci.log` and `check-verified-source-sha256.json`; the original manifest/logs remain historical evidence. Backend database guidance and task records were synchronized. Parent records and other children were not edited.

## Findings (fixed)

- File: `crates/ferrostage-settings/src/parse/controls.rs`, `parse_binding_list`.
- Issue: non-string array elements were discarded. For `keybindings.next = [123]`, the effective value became an empty list; mixed arrays silently lost elements. This violates S-5/R-32 fallback of an invalid typed leaf and can unintentionally unbind an action.
- Reproduction: added `invalid_keybinding_elements_fall_back_only_that_action`; the initial run failed with `[]` versus `[PageDown, Space, Right]`. See `check-keybinding-regression.log` (exit 101).
- Fix: an invalid element reports its index and falls back to that action's default list. Other actions retain their values. Explicit `[]` remains a valid unbinding. Tests cover numeric-only, mixed-type and explicit empty arrays alongside existing duplicate/empty-string/conflict diagnostics. The backend settings spec records this boundary.

## Findings (not fixed)

None within task 02. Cargo deny's existing duplicate-version warnings for `syn` and `windows-sys` are non-failing and do not warrant dependency changes here.

## Requirement review

| Requirements | Reviewed behavior and evidence |
| --- | --- |
| S-1–3 | Eight sections/defaults match PROJECT_PLAN and parent additions; core owns MonitorRole, settings owns lowercase serialization; invalid roles become None. Cache defaults reuse render constants; no platform/GPUI dependency was introduced. |
| S-4–5 | Platform paths and CLI override are separated from pure parsing. Startup explicitly fills writable missing sections; pure parse/check-config do not write. Syntax recovery verifies a backup before allowing explicit save; failed backup is read-only. Section and leaf fallback retain unrelated values. |
| S-6 | File replacement writes a unique sibling, syncs, closes and safely renames without deleting the old destination. Actual Windows denied-share/readonly/interruption/retry evidence remains applicable; this CI also reran the Windows test successfully. No universal power-loss guarantee is inferred. |
| S-7 | Merge updates known leaves in cloned tables and retains unknown nested values. Monitor provenance matches id plus duplicate occurrence; deleted bindings are removed, new bindings start fresh, and successful session saves rebase provenance. Repeated-save and failure/retry tests pass. |
| S-8 | Implicit v1 migrates the real flat cache/debounce keys; legacy display indexes remain unknown with warnings. Future/unrecognized versions deny saves before temp creation and do not trigger startup writeback. |
| S-9 | Vec<String> invalid-type fallback is now fixed. Explicit empty lists remain valid; duplicate/empty-string/conflict diagnostics retain raw string semantics. Runtime syntax and dispatch remain task 07 responsibilities. |
| S-10–11 | CLI syntax/read failures exit 1, leaf fallback exits 0, and check-config creates no backups or writes. SettingsSession retains runtime values, dirty/error and original provenance on failure; successful retry clears them. Diagnostics use tracing and do not dump source file text. |

All production write targets are the configured settings file and its sibling temporary/backup artifacts. No user PDF write path was added. Public interfaces and crate boundaries remain unchanged by this review. Module splitting retains private parsing/IO boundaries; no template or platform hook touch points apply to this product settings task.

## Verification

- Expected regression FAIL: `cargo test -p ferrostage-settings invalid_keybinding_elements_fall_back_only_that_action`, exit 101 before the fix. The initial sandbox attempt could not access Cargo's existing build lock; the authorized unsandboxed retry ran and reproduced the assertion failure.
- Targeted PASS: `cargo test -p ferrostage-settings keybinding`, 3 tests, exit 0; `check-keybinding-tests.log`.
- Full PASS: `just ci`, exit 0; `check-ci.log`. Format check and Clippy `--workspace --all-targets -- -D warnings` passed (including Rust type checking); 59 tests passed: settings 38, core 10, CLI 4, other crates 7. Cargo deny advisories/bans/licenses/sources passed.
- PASS: `git diff --check` and `python .trellis/scripts/task.py validate .trellis/tasks/10-14-02-settings-schema` (16 implement and 13 check context entries). Context validation only checks references.
- Environment: Rust/Cargo 1.99.0, x86_64-pc-windows-msvc, Windows 10.0.26300.0; persisted native test protocol used NTFS (`windows-persistence.log`). Linux/macOS paths were tested as pure inputs, not native filesystem behavior.
- No running checks remain. No required task 02 manual item remains; UI/runtime binding, three-screen behavior and other native product gates belong to their owning children. No commit, push, archive or journal operation was performed.
