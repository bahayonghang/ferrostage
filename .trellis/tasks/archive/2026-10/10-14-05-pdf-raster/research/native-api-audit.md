# Exact native API and distribution review

Reviewed `pdfium-render =0.9.4` from the Cargo registry source, with default features disabled and only `pdfium_7881`. The local package is under `C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/pdfium-render-0.9.4`. Registry source hashes are included in `verification-manifest.json`. This record summarizes actual source inspection; it is not a claim that the wrapper is an independent security audit.

| Source | Verified contract and implementation consequence |
| --- | --- |
| `src/pdfium.rs:70,232` | Bindings live in a global OnceCell; `new_with_config` initializes only once. FerroStage serializes sessions and freezes canonical library/font configuration after successful binding. Dropping a worker does not unload the process-global DLL. |
| `src/pdfium.rs:330` | `load_pdf_from_byte_vec` owns document bytes. Native documents remain inside the worker-local Pdfium lifetime; only owned metadata/pixels cross channels. Probe uses the same worker. |
| `src/config.rs:133,184` | Safe user-font-path and AGG renderer configuration is available. Paths must exist and be representable; no raw custom font callback is required. This proves configuration support, not a specific font substitution mapping. |
| `src/pdf/bitmap.rs:164` | Checked safe `from_bytes` computes preferred stride, checked buffer size and validates the borrowed slice. BGRA's stride is width times four. The bitmap scope ends before the owned Vec moves into BgraFrame. No unchecked API or project unsafe code is used. |
| `src/pdf/document/page/render_config.rs:115,510` | Reverse byte order defaults to true; the adapter explicitly disables it. Every pixel of the solid red fixture must be `[0,0,255,255]`. |
| `src/pdf/document/page/render_config.rs:632,655` | Matrix/clip rendering disables form overlays. The adapter uses visible rotated CropBox coordinates and does not combine `set_origin` with the custom transform. Forms are outside this raster contract. |

The Windows x64 archive is publisher release `chromium/7881`, VERSION `151.0.7881.0`. Source: <https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/7881/pdfium-win-x64.tgz>. Independent SHA256 verification matched archive `73cc0de638ac2095e7445bf56a38200a5b7c7ca0e9f4ba144598f2457377ac08` and DLL `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`. The application does not download or install the runtime; the developer/CI acquisition script checks archive, DLL and all notice hashes.

`docs/pdfium-runtime.json` inventories the packaging LICENSE and fifteen component notices. The separately reviewed native distribution includes PDFium BSD/Apache, packaging MIT, Abseil Apache, AGG permissive, fast_float MIT, FreeType FTL, ICU Unicode and third-party terms, JPEG IJG/BSD/Zlib, lcms MIT, OpenJPEG BSD, libpng and legacy terms, libtiff permissive, LLVM Apache/exception/legacy terms, simdutf MIT and Zlib. The combined ICU notice also mentions GPL build scripts with explicit distribution exceptions; those scripts are not shipped or linked as runtime code. Keep complete unmodified notices and the FreeType/IJG acknowledgments described in `docs/packaging.md`. No GPL/AGPL allowlist exception was added.

`cargo deny` covers the Rust dependency graph, including the optional wrapper. It does **not** audit the precompiled DLL's third-party components; that review and its notice inventory are separate evidence. Runtime hashes lock the reviewed artifact, not every possible future distribution.
