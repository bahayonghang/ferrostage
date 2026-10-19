# Design

## Direction and repair contract
Visual thesis: warm, quiet utility UI with the existing Georgia FerroStage wordmark, user-selected interface typography, warm neutral surfaces, burnt-orange primary action and flat file rows.

Content plan: brand/utilities, short instruction and natural-width Open PDF, then a compact recent-history table with filename/parent path and last-opened columns; actionable localized import feedback remains inline.

Interaction thesis: no new animation. Preserve existing focus rings, full-row click/Enter/Space activation, hover fill and 1px pressed feedback. CSS strategy: none, this is native GPUI using existing theme tokens and element styles.

Target is the Idle home history and PDF import feedback. Preserve workbench/audience, monitor repair, settings categories, native file dialog/import guards, real path tooltips/focused-path display, source ordering and top-three scope. Existing screenshot and a live 1100x760-class normal window establish the baseline. Use normal-window native screenshot smoke and existing regression checks; no new size/DPI matrix.

## History data
Keep schema_version=1 and `general.recent_files: Vec<String>`. Add optional `recent_opened_at: BTreeMap<String, i64>` containing nonnegative UTC Unix seconds keyed by exact recent path. Missing/invalid/unrepresentable times display localized unknown. Parse recent paths first, prune metadata to effective retained paths, and replace the timestamp map as one known TOML leaf on save so removed keys cannot resurrect. Preserve unrelated unknown TOML fields and future-version read-only policy.

`GeneralSettings::record_recent_file(path, opened_at)` owns successful MRU/time updates and pruning. Shell calls it only after successful session/core document acceptance inside the existing Writable guard; capture UTC time there, not during reading/rendering. A failed save retains runtime success/time and existing Unsaved/Retry. Clear recent clears both containers; existing settings edit serialize/reparse provides limit pruning.

Presenter data captures matching timestamps for its three recents. Format dates using existing chrono in local time, absolute minute-resolution date/time; use localized unknown for missing/unrepresentable values. Rendering performs no file metadata reads. The history row has a flexible filename/path column and a shared aligned time column, quiet header/separators and no new card grid. Reuse existing compact behavior when space is insufficient; do not truncate the timestamp. Keep complete path accessible by tooltip and keyboard focus.

## Error boundary
The screenshot's `PdfStructureError::Unreadable` is flattened into `std::io::Error::other` and rendered through `{error}`. Retain a typed UI read/structure failure across background work. Classify missing/permission/sharing/other IO and the three PdfStructureError variants into catalog text, preserving technical details in tracing. `Unreadable` describes incomplete or invalid content and suggests waiting if compilation is in progress; it never asserts compiler activity. Map later candidate/render failures to localized actionable import copy instead of displaying internal English diagnostics. No automatic retry/compiler detection or PDF writing is introduced.

## Verification
Settings tests cover old/mixed-invalid metadata, whole-map save replacement, unknown preservation, pruning, clear, read-only and retry. UI tests exercise success-only timestamps and failure preservation, row activation/focus, known/unknown dates and table column alignment. Use a deterministic partial-PDF fixture for red/green localization. Preserve all pre-existing monitor changes and independently review both data and UI ownership before release build. Build a new installer for review but do not automatically install without a new user request.
