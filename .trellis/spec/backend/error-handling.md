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
