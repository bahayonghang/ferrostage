# ADR-002: PDF backends

## Status

Accepted. Only `lopdf` is linked.

## Context

Rendering needs a BSD-licensed engine that can return BGRA8. Text extraction and incremental structure checks need a pure-Rust reader. MuPDF is AGPL and is banned.

## Decision

| Crate | Version | Role in this tree |
| --- | --- | --- |
| `lopdf` | 0.45.0 | Linked. `parse_structure` loads a buffer. |
| `pdfium-render` | 0.9.4 | Pinned, not linked. Future raster path: memory load, BGRA8, one thread. |
| `hayro-syntax` | 0.8.0 | Pinned, not linked. Future syntax reader. |

Versions are also constants in `ferrostage-pdf`.

## Consequences

`cargo test` does not download PDFium. The raster task must disable any default feature that fetches binaries until the worker is ready, and it must not convert BGRA to RGBA.
