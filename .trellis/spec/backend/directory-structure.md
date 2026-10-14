# Directory Structure

## Layout

```text
crates/ferrostage-core/        state machine, no GPUI, no file I/O
crates/ferrostage-document/    notes and media slots
crates/ferrostage-pdf/         lopdf structure; PDFium later
crates/ferrostage-render/      generation checks and cache budgets
crates/ferrostage-media/       shared BGRA frames
crates/ferrostage-platform/    debounce, %%EOF, atomic rename
crates/ferrostage-settings/    TOML
crates/ferrostage-ui/          GPUI (see frontend spec)
crates/ferrostage-app/         binary only
docs/                          plan, architecture, ADRs
assets/                        icons, locales, font README (no font binaries yet)
```

Each library crate has `src/lib.rs`. Split a crate into modules when a file passes roughly 400 lines. Do not add a `src/bin/` next to a library unless the binary is a developer tool for that crate.

## Dependency direction

- `ferrostage-core` does not depend on any other FerroStage crate.
- `ferrostage-ui` may depend on `ferrostage-core` and, later, `ferrostage-document` for the presenter only.
- `ferrostage-core` must not depend on `ferrostage-ui`, GPUI, `lopdf`, or `notify`.

## Example

Page identity is a newtype, not a bare `usize`, so a logical slide cannot be passed where a physical page is required:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PhysicalPage(pub usize);
```

That definition lives in `crates/ferrostage-core/src/ids.rs`.
