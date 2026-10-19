# Settings implementation results

## Scope

Changed only `crates/ferrostage-settings` plus this requested evidence file. No monitor changes, UI, locales, specs, dependencies, schema bump, Git commit or installation were performed by this worker.

## Implemented contract

- `GeneralSettings::recent_opened_at: BTreeMap<String, i64>` defaults empty and stores exact path keys with nonnegative Unix seconds.
- `GeneralSettings::record_recent_file(path: String, opened_at: i64)` updates the one path's time and MRU position, deduplicates all paths, applies the configured history limit and prunes orphan times. A negative timestamp leaves this path's time unknown instead of inventing a date.
- The parser resolves the existing paths/limit before validating timestamp leaves. Missing map stays empty without forcing optional-leaf writeback. Wrong table type falls back independently; malformed entries warn individually and valid siblings remain. Large nonnegative i64 values remain accepted, with date representability left to the UI.
- Save replaces the whole timestamp map as one known leaf. Cleared/pruned entries therefore cannot reappear through unknown-field merge. Unrelated general/top-level/monitor fields retain their existing preservation behavior.
- Existing session save failure/retry and future-version read-only policies are unchanged.

## Files

- `src/schema.rs`: typed optional metadata, default, record/prune API.
- `src/parse/sections.rs`: leaf validation and retained-path pruning.
- `src/merge.rs`: whole-map replacement.
- `src/tests/parse.rs`: compatibility, invalid leaves, range, dedup/MRU/limit/prune API tests.
- `src/tests/store.rs`: Windows-path roundtrip with unknown fields, raw-tree whole-map replacement after trim/clear, failure/retry and future-version byte preservation.

## Validation

- Before implementation, new regression tests failed to compile because the timestamp field/API were absent.
- `cargo test -p ferrostage-settings --locked`: PASS, 48 unit tests and doc tests (0 cases). The full result was independently read using `rtk bypass` after one combined command produced no usable output.
- `cargo clippy -p ferrostage-settings --all-targets --locked -- -D warnings`: PASS.
- `cargo fmt -p ferrostage-settings -- --check`: PASS.
- `git diff --check -- crates/ferrostage-settings`: PASS.
- Tightened the record API dedup regression to retain an unrelated duplicated path before enforcing the bound; its targeted rerun passed.

## Integration boundary

The parent/UI worker must call `record_recent_file` only after document acceptance and within the existing Writable guard, clear both path list and timestamp map, preserve failed-import history, format local times with an unknown fallback, and test UI success/save-failure/read-only/limit integration. This worker's tests prove storage behavior, not actual UI timestamps or native rendering.
