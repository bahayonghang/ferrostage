# Implementation results

## Scope completed

Continued the interrupted UI implementation without replacing the existing settings work or the adjacent monitor-discovery changes. The approved home direction remains the existing warm utility surface, Georgia wordmark, user-selected interface font and natural-width Open action. The flat history now uses a flexible filename/parent column, a shared aligned last-opened column, a quiet header and separators. Existing focus, press, whole-row activation, full-path tooltip/focus disclosure, original path ordering and three-entry home scope remain in place.

`ImportReadError` keeps IO and PDF structure failures typed across background work. Missing, locked, permission, other IO, unreadable content, invalid page geometry and unsupported page units receive separate catalog messages. Later candidate/session/core failures use localized recovery copy. Technical details remain in tracing. Incomplete content suggests waiting **if** compilation/export is in progress; it does not claim compiler detection.

`finish_import` records UTC seconds only after descriptor/core acceptance and only for writable settings. Clear removes both paths and timestamp metadata; existing serialize/reparse validation prunes metadata after a limit edit. Save failure retains successful runtime history/time and the existing Unsaved/Retry flow. Presenter projection captures the first three matching dates; formatting uses local minute-resolution dates and localized unknown copy for missing, negative or unrepresentable values, without filesystem metadata reads.

## Files

- UI implementation continued in `crates/ferrostage-ui/src/shell.rs`, `presenter.rs`, `presenter/home_view.rs` and `shell/preferences.rs`.
- Existing English/Chinese catalog changes in `assets/locales/{en-US,zh-CN}/app.toml` were reviewed and retained; monitor-related keys and code were preserved.
- Focused tests added/extended in `shell.rs`, `presenter/home_view.rs`, `settings_panel.rs` and `shell/preferences_tests.rs`.
- Existing settings schema/parser/merge/tests were retained without further changes by this continuation agent. Their previous 48-test pass is reported by the parent context; the independent checker owns fresh final settings/workspace verification.

## Regression coverage

- Deterministic partial PDF reaches the actual asynchronous read/structure boundary, shows conditional Chinese wait/retry guidance, and preserves active generation/path/document plus recent paths/times.
- Both catalogs map every typed read failure category without leaking raw technical messages.
- Successful reopen updates MRU/time, core rejection preserves history/path, read-only successful import preserves history and on-disk bytes.
- Failed save retains successful runtime timestamp; Retry persists it together with the existing runtime display result.
- Clear dispatch and readback empty both containers; checked limit edits prune matching timestamps.
- Missing, negative and `i64::MAX` timestamps show localized unknown; known dates format local absolute minutes.
- Normal 1100x760 home at the user's 18px interface size projects only three source-order paths, displays known/unknown values, aligns header/time boxes, and retains click/Enter/Space activation without refreshing failed history.
- The unchanged pre-existing home, settings, workbench and monitor regression suite was run as part of the full GPUI suite. No new size/DPI matrix was added.

## Checks actually run

| Check | Result |
| --- | --- |
| `cargo test -p ferrostage-ui --features test-support --lib --locked -- --nocapture` | PASS, 179 tests, 0 failed; execution 1.37s. Full output: `research/ui-tests.log`. |
| `cargo build -p ferrostage-app --features gpui,pdfium --locked` | PASS, dev profile, 33.42s. Output: `research/product-build.log`. Debug desktop binary rebuilt for parent native verification. |
| `cargo fmt --all` | Completed after the suite; formatting-only cleanup, no behavior changes. |
| `git diff --check` | PASS after final formatting and implementation report creation. Independent checker repeats final whitespace/fmt gates. |

Tooling note: this shell's `cargo` resolves to the mbx wrapper and did not expose a persistent command result in the initial attempts. Direct Rust 1.99 Cargo initially hit E0514 when the GPUI checkout's rustup resolution selected rustc 1.98.1 against 1.99 artifacts. Successful commands used the absolute 1.99 Cargo executable with process-local `RUSTUP_TOOLCHAIN=1.99.0` and `RUSTC` pointing to that toolchain's rustc. No repository dependency/toolchain/cache cleanup or waiver was introduced.

## Measured headless geometry

At 1100x760, interface size 18 (WenYuan Serif SC requested; headless font substitution is possible):

- Shared header/time x=755..926, width 171px.
- Filename x=174..739, width 565px; 16px gap before time column.
- Row bounds x=170..930; tops 281.5/334.5/387.5, each 52px high.
- Time tops 297.5/350.5/403.5, each 20px high and within its row.

The parent measured the old native screenshot's name/path area at approximately x=174..928 with no time column. These old native and new headless measurements have different evidence strength; they establish the intended column allocation, not a before/after native visual pass. Parent owns the rebuilt normal-window screenshot, real font rendering and final native geometry receipt.

## Remaining parent/checker work

Fresh workspace CI, product lint, native PDF regressions, full-feature dependency policy, independent review and release packaging remain with the parent/checker. Native visual acceptance is not claimed by this report. No commit, push, installation or user-PDF write was performed.
