# ADR-005: License gate

## Status

Accepted.

## Decision

The project license is MIT OR Apache-2.0 (`LICENSE`, `LICENSE-MIT`, `LICENSE-APACHE`).

`deny.toml` allows a short permissive list and does not allow GPL or AGPL. `mupdf` is denied by name. `just ci` runs `cargo deny check`.

Pympress remains GPL-2.0-or-later and is not a dependency. Do not translate its source into this tree.

## Consequences

A crate that is only "GPL-compatible" is still rejected. Dual-licensed dependencies are fine when one side is MIT or Apache-2.0.
