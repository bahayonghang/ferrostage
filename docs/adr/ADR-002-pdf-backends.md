# ADR-002: PDF backends

## Status

Accepted. `lopdf` is linked by default; `pdfium` enables the native worker.

## Context

Rendering needs a BSD-licensed engine that can return BGRA8. Text extraction and incremental structure checks need a pure-Rust reader. MuPDF is AGPL and is banned.

## Decision

| Crate | Version | Role in this tree |
| --- | --- | --- |
| `lopdf` | 0.45.0 | Linked. `parse_structure` loads a buffer. |
| `pdfium-render` | =0.9.4 | Optional, only `pdfium_7881`; memory load, BGRA8, one thread. |
| `hayro-syntax` | 0.8.0 | Pinned, not linked. Future syntax reader. |

Versions are also constants in `ferrostage-pdf`.

## Consequences

`pdfium-render` does not ship or download the native library. The exact Windows
151.0.7881.0 runtime and all notice hashes are locked in `docs/pdfium-runtime.json`.
Default tests need no DLL; the separate Windows native job requires one and fails
when it is missing. `RuntimeConfig` supplies an explicit file and optional font
directories. No production environment variable or system search fallback is used.

All native operations, including probe, run in the render worker. Owned bytes
transfer to `load_pdf_from_byte_vec`; native documents never leave its scope.
The wrapper's process-global bindings persist for the process lifetime. Only one
session may run; the first successfully initialized canonical library/font
configuration is immutable. Failed binding does not poison retry. Closing a worker
releases its documents and working bitmap, but does not unload those global bindings.

The safe matrix/clip path uses visible rotated CropBox coordinates and disables
form-data rendering (interactive form overlays are unsupported). It explicitly
disables default byte-order reversal. The checked `PdfBitmap::from_bytes` borrows
a uniquely owned BGRA Vec during rendering; the bitmap dies before moving the Vec
into the immutable core BgraFrame (`Arc<Vec<u8>>`, read-only slice access). Frame
construction and cloning preserve the pixel allocation; no RGBA conversion occurs.
Each real render allocates and clears its own bounded buffer (at most 256 MiB);
there is no scratch reuse dependent on consumers releasing previous frames. This
avoids the wrapper's raw-export copy but is not an end-to-end zero-copy claim.
L2/L3 caches share their byte budgets across all generations; UI owns GPU textures.
