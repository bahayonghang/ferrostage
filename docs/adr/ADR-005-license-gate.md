# ADR-005: License gate

## Status

Accepted.

## Decision

The project license is MIT OR Apache-2.0 (`LICENSE`, `LICENSE-MIT`, `LICENSE-APACHE`).

`deny.toml` allows a short permissive list and does not allow GPL or AGPL. `mupdf` is denied by name. `just ci` runs `cargo deny check`.

Pympress remains GPL-2.0-or-later and is not a dependency. Do not translate its source into this tree.

## Consequences

A crate that is only "GPL-compatible" is still rejected. Dual-licensed dependencies are fine when one side is MIT or Apache-2.0.

## Exact task07 exceptions approved

After the concrete independent maintenance review in [ADR-008](ADR-008-gpui-dependency-gate.md), the user approved only these package/version pairs: `arrayref@0.3.9` under BSD-2-Clause, `libbz2-rs-sys@0.2.5` under bzip2-1.0.6, and `cbindgen@0.28.0` under MPL-2.0 (Apple build tool). Apply them as exact per-crate exceptions, retain notices/source obligations and rerun the actual full product graph. MPL is file-level copyleft; this is a specific approved exception, not a general permissive-license reclassification or build-dependency exemption. The global allowlist, full target/build scope, private checks, GPL/AGPL exclusion and advisory policy are unchanged. No additional version or package is authorized by this decision.
