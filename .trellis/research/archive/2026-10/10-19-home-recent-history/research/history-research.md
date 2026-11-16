# Research: Successful-open timestamps and v1 history compatibility

- Query: Add last successful PDF-open time without changing recent-path compatibility, read-only policy or save/retry semantics.
- Scope: internal
- Date: 2026-10-18

## Findings

### Recommended data and contracts

Keep `general.recent_files: Vec<String>` and `schema_version = 1`. Add optional `general.recent_opened_at: BTreeMap<String, i64>`, keyed by the exact path string already used by recents, values nonnegative UTC Unix seconds. Missing map means empty; missing per-path entry means unknown, rendered as localized `暂无记录`, never file mtime, migration time or startup time. No additional dependency is needed for storage. Chrono already exists behind UI `gpui` for local display.

Expose `GeneralSettings::record_recent_file(path: String, opened_at: i64)` to deduplicate, move path first, update its timestamp, truncate to `recent_files_limit`, then retain timestamps only for retained paths. Optional pruning helper is justified if used both by this method and parser. Parse the recent paths before the map so pruning uses the effective deduped/truncated paths. Reject invalid table/individual noninteger or negative values with individual warnings and preserve valid sibling entries. An out-of-date map entry removed by path-limit pruning is not a malformed setting and should not warn.

Merge the map as one known leaf, replacing its table rather than recursively preserving removed path keys. Otherwise clear/limit trimming would resurrect history. Unknown general sibling keys, top-level keys and monitor metadata retain the existing preservation contract. Older current-version binaries already preserve unknown general leaves, so they retain the new map while continuing to read the unchanged path list. They cannot update new timestamps; stale/orphan map entries are pruned by the new loader on return.

Capture the clock only after both session descriptor creation and core `LoadMappedDocument` succeed. Existing `Shell::finish_import` lines 765-773 define this commit point; update history where the existing Writable-gated mutation occurs, lines 792-800. No updates on failed read, renderer/session build, rejected descriptor/core transition, canceled import, or reload. Keep future-version read-only behavior: current Shell intentionally does not update recents at all under that policy. A settings-save failure must retain the successful open and in-memory timestamp, mark dirty and allow existing Retry, rather than reporting document-open failure.

`chrono::Local.timestamp_opt`/`DateTime::from_timestamp` must be handled as optional at display time; huge but nonnegative TOML integer values cannot be assumed representable in local time. Fall back to unknown safely. Use absolute local date/time format with clear minute resolution; do not poll filesystem during rendering or generate fake relative times.

### Files and code patterns

| File | Responsibility / evidence |
| --- | --- |
| `crates/ferrostage-settings/src/schema.rs:13` | Schema stays version 1. General defaults and recent paths at lines 140-174. |
| `crates/ferrostage-settings/src/parse/sections.rs:16` | Leaf-by-leaf general parser; recent list loader at line 335 preserves source order, deduplicates and truncates. Add map parsing here. |
| `crates/ferrostage-settings/src/merge.rs:42` | `merge_general` writes known leaves into the cloned raw tree; unknown siblings remain. Add complete map replacement. |
| `crates/ferrostage-settings/src/store.rs:203` | `save_with_fault` refuses read-only before temp writes. `save_session` at line 234 retains edited values with dirty/error on save failure. No new store pipeline needed. |
| `crates/ferrostage-ui/src/shell.rs:758` | `finish_import` owns success, MRU reordering and save; change this sole import mutation. |
| `crates/ferrostage-ui/src/settings_panel.rs:327` | Runtime edit serializes the complete settings then re-parses, lines 363-382. Parser-side map pruning automatically handles limit reduction. |
| `crates/ferrostage-ui/src/shell/preferences.rs:134` | `ClearRecent` currently clears only paths; also clear the timestamp map. ResetPreferences constructs defaults, so empty map follows automatically. |
| `crates/ferrostage-ui/src/presenter.rs:89` | Presenter projection currently exposes first three paths in source order; project matching optional timestamps without mutating history. |
| `crates/ferrostage-ui/src/presenter/home_view.rs:92` | Existing row loop sends `Intent::Import`, derives basename/parent from exact path, and keeps full-path tooltip/focus support. Retain these behaviors. |
| `crates/ferrostage-ui/src/presenter/settings_view.rs:116` | Settings also displays full history source. Keep it coherent after clear/limit edits. |
| `crates/ferrostage-ui/Cargo.toml:13` | `gpui` already enables optional chrono; workspace chrono is pinned at Cargo.toml:57, no dependency change. |

### Targeted verification

- `crates/ferrostage-settings/src/tests/parse.rs:258`: extend recents fixture with known and unknown timestamps, duplicate and truncated paths. Add omitted-map compatibility, invalid whole map and mixed-valid leaves; preserve future policy.
- `crates/ferrostage-settings/src/tests/store.rs:59`: extend unknown-data preservation fixture with map roundtrip while retaining unknown general/display/monitor metadata. Add complete map replacement for clear/prune and old-consumer behavior by removing the new leaf from typed knowledge conceptually: current old merge already clones raw then overwrites only known leaves.
- `crates/ferrostage-settings/src/tests/store.rs:188`: failed save/retry retains runtime timestamp and later persists it. Future refusal bytes test at line 257 remains relevant.
- `crates/ferrostage-ui/src/settings_panel.rs:468`: limit edit test should assert map pruned to kept path, unrelated entries retained correctly, coupled validation still atomic.
- `crates/ferrostage-ui/src/shell/preferences_tests.rs:875`: actual ClearRecent dispatch test currently has duplicate paths-empty assertion; replace second assertion with map-empty and assert readback clears both.
- `crates/ferrostage-ui/src/shell.rs:4772`: existing `v6_import_and_swap_keep_runtime_results_when_saving_fails` is the best success/save-failure regression to extend with map assertions and readback at line 4848. Add separate no-time-update assertions on failing import path and successful import under read-only.
- Existing home tests around `shell.rs:6768` and `6842` cover keyboard row activation, return-to-home and history retention. Add displayed known/unknown timestamp state without replacing user fixtures with invented metadata.

### Related specs

- `.trellis/spec/backend/database-guidelines.md`: optional v1 leaves require typed schema + parser + known-leaf merge together; missing optional leaves should not force schema bump or startup rewrite. Unknown data, atomic settings save, future read-only and effective-value retention on failure are contracts.
- `.trellis/spec/frontend/state-management.md`: one settings owner/save-retry flow, recents only in Idle, import enters Ready with explicit Start.
- `.trellis/spec/backend/error-handling.md`: keep typed errors at library boundaries; UI owns localized product copy.
- `.trellis/spec/guides/cross-layer-thinking-guide.md`: verify storage-to-projection-to-view pipeline and all settings/history consumers.

### External references / versions

No external research needed for this internal data contract. Verified current workspace dependency is `chrono = =0.4.45`, features `clock`, UI optional under `gpui`.

## Caveats / Not Found

- Existing history contains paths only. No reliable past open time is stored, so migration cannot recover it.
- Old binaries preserving an unknown map do not know to prune it when clearing paths. New loader must prune against effective current paths; do not claim the old binary updates times.
- No import timestamp acceptance test existed at research time; add focused assertions to existing success/failure and preference tests.
- Screenshot parse error does not establish a compiler race; error localization should describe interrupted/incomplete/unsupported content as possibilities, not assert compilation as the cause. Error pipeline investigation is separate from this narrow history research.
- Research is read-only outside this file; no code, specs, Git operations, task activation or validation runs were performed.
