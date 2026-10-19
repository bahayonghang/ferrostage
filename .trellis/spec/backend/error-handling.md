# Error Handling

Libraries return `Result`. They do not panic for bad input, and they do not `unwrap` except in tests.

## Types

`ferrostage-core` uses `thiserror`:

```rust
#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("document has no pages")]
    EmptyDocument,
    #[error("page {page} is outside document length {len}")]
    PageOutOfRange { page: usize, len: usize },
    #[error("shadow generation {next} is not newer than {current}")]
    StaleGeneration { current: u64, next: u64 },
    #[error("shadow document is empty")]
    EmptyShadow,
}
```

`ferrostage-pdf::PdfStructureError` and `ferrostage-settings::SettingsError` follow the same pattern. Add a variant; do not stringly-type a new failure mode.

## Binary

`ferrostage-app` prints a one-line message to stderr and returns `ExitCode::from(1)`. It does not panic on a missing settings file.

## Forbidden

- `unwrap` / `expect` on library paths that handle user files.
- Swallowing a `Result` with `let _ =`.
- Using `Box<dyn Error>` across crate boundaries. Keep the concrete error type public.
- `unsafe` to "simplify" an error path. Workspace lint `unsafe_code` is `forbid`.

## PDF import feedback

### 1. Scope / Trigger
Apply when a user opens a PDF from the chooser, recent history or startup argument. Technical parser/renderer diagnostics belong in tracing, while user recovery copy belongs in the locale catalogs.

### 2. Signatures
The UI's `ImportReadError` preserves `Io(std::io::Error)` and `Structure(PdfStructureError)` through background reading and structure extraction. `message(Locale)` selects a catalog entry without interpolating internal Display text.

### 3. Contracts
Distinguish missing files, sharing/lock violations, permissions, other IO, unreadable structure, invalid geometry and unsupported page units. On Windows, raw sharing/lock codes 32/33 take precedence over PermissionDenied. Unreadable bytes can be incomplete or invalid; they do not prove compilation is active. Suggest waiting conditionally, then reopening or re-exporting. Later renderer/candidate/core failures also use localized import copy with structured diagnostics retained in logs. Failed opens preserve the active document and successful-open history/timestamps.

### 4. Validation / Error Matrix

| Failure | User feedback |
| --- | --- |
| Missing path / access denied / sharing violation | Distinct actionable catalog entry |
| Truncated or malformed PDF | Incomplete-or-invalid guidance, conditional compile/export wait |
| Invalid geometry / unsupported page unit | Specific re-export guidance |
| Renderer unavailable / candidate rejected | Localized restart or re-export guidance |

### 5. Good / Base / Bad Cases
Good: preserve the typed parser error across the background task and log its details. Base: a legacy recent path still opens through the same import boundary. Bad: wrap a parser error in `io::Error::other`, expose its English Display text or assert that a compiler is running.

### 6. Tests Required
Retain deterministic truncated-PDF localization and failed-import state/history preservation, distinguish IO/structure categories in both catalogs, and keep successful-open timestamp coverage separate from failed saves and read-only settings. Native screenshots establish rendered copy independently of headless checks.

### 7. Wrong vs Correct
Wrong: translate a raw parser string or refresh history when reading begins. Correct: typed error classification, localized recovery at the UI boundary and success-only history updates after document acceptance.

## Monitor snapshot failures

### 1. Scope / Trigger
Apply when combining Windows device/geometry samples or exposing a monitor error to the presenter. The two device queries and geometry query are not one atomic Windows snapshot.

### 2. Signatures
`PlatformError::TopologyChanging { before: Vec<String>, after: Vec<String> }` reports inconsistent device snapshots. `IncompleteTopology { devices, geometry }` reports an actual device/geometry connection-set mismatch. `windows::combine_snapshots(devices, geometry, after)` validates snapshots before calling `combine`.

### 3. Contracts
Compare the entire owned Device records, not only their connection keys. Identity path, name or primary metadata may change while keys stay identical. Either failure remains unavailable topology, never permission to reuse stale geometry or assign another audience. Retain typed errors through the UI boundary and structured diagnostics on failure transitions; the library's English Display text is not localized product copy.

### 4. Validation / Error Matrix

| Input | Result |
| --- | --- |
| Same keys, different path/name/primary | TopologyChanging |
| Stable device snapshots, missing/extra geometry | IncompleteTopology |
| Stable snapshots and complete valid geometry | Owned MonitorInfo list |
| Either failure followed by stable original topology | Watcher publishes recovery even if monitors match its last good list |

### 5. Good / Base / Bad Cases
Good: distinguish a device change from missing geometry. Base: unchanged samples retain all identity/geometry fields. Bad: present identical key lists as proof of a geometry mismatch or accept their intersection.

### 6. Tests Required
Cover metadata-only changes with identical keys, actual geometry mismatch and unchanged success in `windows/tests.rs`. Keep the native repeated-enumeration GDI resource regression. The watcher recovery test must retain same-topology recovery after failure.

### 7. Wrong vs Correct
Wrong: return IncompleteTopology for every device snapshot change. Correct: classify unstable snapshots separately and keep completeness rejection intact. A screenshot without structured fields does not establish the original OS trigger; do not replace native acquisition based on an unconfirmed clipping/DPI hypothesis.
