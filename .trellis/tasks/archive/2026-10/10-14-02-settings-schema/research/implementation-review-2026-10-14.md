# Task 02 implementation review — 2026-10-14

Scope: existing settings schema/parse/merge/store, app configuration CLI, core MonitorRole. All work remains uncommitted; no other child is implemented here.

## Resume audit and boundary

- Branch main, HEAD ab78a84; task02 is in_progress. Existing uncommitted implementation already provides the eight sections, leaf fallback, legacy migration, unknown-key merge, path resolution, backups, atomic save, CLI and tests. It had no applicable recorded completed checks.
- Read task PRD/design/implement, injected backend specs/shared contracts and task01 review. Preserve existing implementation. MonitorRole is already core-owned and serde stays in settings.
- Concrete gap: save_session updates raw_table but leaves monitor source_occurrence relative to the old tree. Deleting/reordering duplicate IDs can lose unknown fields on a subsequent save. Reproduce before fixing; rebase provenance only after successful persistence.
- Windows test currently accepts old OR new bytes regardless of success/failure and does not prove cleanup or retry. Strengthen it to verify result/bytes consistency, cleanup and retry after closing the lock. Report actual filesystem/OS, not a hardcoded C drive.
- No GPUI, topology resolution, lifecycle, native raster, or UI code is added. No PDF path is written.

## Checks

- FAIL (expected regression): `cargo test -p ferrostage-settings -p ferrostage-core -p ferrostage-app`; app 4 pass, core 10 pass, settings 34 pass / 1 fail. New duplicate_monitor_metadata_survives_repeated_session_saves lost marker `second` on second save. Output: `baseline-tests.log`. Initial sandbox build-lock access denied; authorized elevated retry executed successfully and exposed the regression.
- Fix: successful save_session rebases source_occurrence to the merged tree's duplicate order. Failed saves do not rebase or discard in-memory changes.
- PASS: targeted settings/core/app tests after fix: 35 settings + 10 core + 4 CLI tests. Output: `targeted-tests.log`.
- PASS: Windows protocol, `cargo test -p ferrostage-settings tests::windows -- --nocapture`. Native OS 10.0.26300.0; actual temp volume C: NTFS. Missing/existing destinations produced complete new content. Read-only attribute and handle shared READ|WRITE without DELETE both returned Replace/OS error 5 and retained complete old content with zero temp leftovers. Releasing the handle allowed retry. Interrupt-after-sync fault left a complete temp and unchanged original; injected replace failure retained original and cleaned temp. Output: `windows-persistence.log`. This is controlled interruption simulation, not power-loss durability testing.
- Added session failure/retry state coverage. Existing parse.rs (~1,300 lines), store.rs (~630), schema.rs (~530) exceed backend module guidance; split along parsing/validation, file IO, and keybinding boundaries without behavior changes, then re-run tests.
- PASS: full `just ci` after module split and formatting, exit 0: fmt, Clippy `-D warnings`, 57 workspace tests (36 settings, 10 core, 4 CLI, 7 other), cargo deny advisories/bans/licenses/sources. Duplicate syn/windows-sys versions remain non-fatal warnings. Output: `ci.log`.
- PASS: `cargo tree -p ferrostage-core --edges normal` and settings equivalent; no prohibited dependency edges. Outputs: `core-tree.log`, `settings-tree.log`.
- PASS: required `cargo run -p ferrostage-app -- check-config <disposable fixture>` exit 0, target 0 becomes 45, SHA256 C007F8055D104F188F99506CBF97431A4B1A617C670DEFC1547EBB574A7AB97D unchanged. Output: `cli-probe.log`. CLI syntax/read errors exit 1 without backup via integration tests.
- PASS: `git diff --check`. Rust 1.99.0 / x86_64-pc-windows-msvc / Cargo 1.99.0 / just 1.58.0; `toolchain.log`.
- PASS: `python .trellis/scripts/task.py validate .trellis/tasks/10-14-02-settings-schema`; implement.jsonl 16 entries and check.jsonl 13 entries valid. This validates context references only.

## Schema/default/range/consumer audit (I-1)

Known leaves are owned by schema.rs and parse/value.rs; monitor role string adaptation stays in settings, with the enum in core. Cache defaults import render constants. No new dependencies were added while resuming.

| Section / leaves | Defaults | Accepted input / consumer |
| --- | --- | --- |
| schema_version | 1 | Missing is implicit v1; unsupported versions/types read-only / Store |
| general.language, theme | zh-CN, dark | zh-CN/en-US; dark/light/system / 07+09 |
| general.reopen_last_document, debug_overlay | false, false | bool / 07, 08+09 |
| general.recent_files_limit, recent_files | 10, [] | 1..100; string array deduplicated/truncated / 07+09 |
| display.single_monitor_mode | presenter_only | presenter_only/windowed_audience / 03+07 |
| display.audience_fullscreen, presenter_start_fullscreen | true, false | bool / 07+09 |
| display.monitors | [] | id/friendly_name strings; role presenter/audience/none; invalid role safely None / 03+07+09 |
| presenter.next_slide_ratio, notes_ratio | 0.35, 0.40 | finite (0,1] / 08+09 |
| presenter.notes_font_scale | 1.0 | finite (0,8] / 08+09 |
| presenter.show_clock, show_thumbnail_strip | true, true | bool / 08+09 |
| presenter.grid_columns, grid_visible_pages | 4, 16 | 1..4; 1..16 / 08+09 |
| timer.mode, target_minutes | count_down, 45 | count_up/count_down; positive u32 / 04+08+09 |
| timer.warn_yellow_remaining_min, warn_red_remaining_min | 10, 3 | u32; yellow > red >= 0 / 08+09 |
| pointer.laser_color_bgra, pen_color_bgra | [0,0,255,255] both | Exactly four u8 channels / 08+09 |
| pointer.laser_radius_norm, pen_width_norm | 0.008, 0.004 | finite (0,0.25] / 08+09 |
| pointer.highlighter_alpha | 0.5 | finite [0,1] / 08+09 |
| keybindings.next / previous | PageDown,Space,Right / PageUp,Backspace,Left | Vec<String>; duplicates/empty/cross-action diagnostics, runtime grammar and conflict resolution owned by 07 / 07+09 |
| keybindings.start_presentation / stop_presentation | F5 / Escape | Same diagnostic policy / 07+09 |
| keybindings.blank_black / blank_white / swap_screens | B / W / S | Same diagnostic policy / 07+09 |
| keybindings.toggle_laser / toggle_pen / jump | L / P / G | Same diagnostic policy / 07+09 |
| cache.l1_gpu_mib, l2_cpu_mib, l3_thumb_mib | 256, 512, 64 | positive u32 / 05+08+09 |
| hot_reload.enabled, debounce_ms | true, 300 | bool; positive TOML integer / 10+09 |

## Requirement and verification coverage

- S-1/2/3: defaults, complete roundtrip, every missing section, safe invalid role, color/ratio/grid/timer checks. Existing MonitorRole tests and dependency-tree review.
- S-4/5/6: pure platform-path fixtures; missing/startup/backup failure; actual native Windows replace protocol; fault injections retain original bytes. Native Linux/macOS filesystem behavior is not claimed.
- S-7/8: known edits preserve unknown root/section/monitor values and binding deletion, original flat keys migrate, future save is denied without changes. Repeated session saves now retain duplicate-monitor metadata.
- S-9: raw keybinding strings diagnosed once by settings; runtime dispatch belongs to task07.
- S-10/11: CLI tests/probe, SettingsSession failure/retry test, tracing warnings and no raw-document diagnostic dump. All file-write targets reviewed: configured settings, sibling temporary files/backups, and isolated test fixtures only.
- File organization split only moved function bodies and adjusted imports/visibility; the post-split CI above is applicable. Production modules are each below 400 lines.

## Next

Implementation and implementer verification complete; independent trellis-check remains next. No checks still running; no external task02 blocker. Source SHA256 manifest covers Cargo manifests/lock/toolchain/justfile, workspace Rust files, and docs/packaging.md at the verified state. Parent Gate A still needs 04 and 03; no UI, GPU, PDFium or full-parent acceptance is implied. Keep task.json in_progress and all modifications uncommitted for review.
