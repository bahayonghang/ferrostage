# Packaging

Not implemented yet. The first package is a Windows 11 executable.

- `just install` runs `cargo install --path crates/ferrostage-app --locked`.
- The optional `pdfium` feature links `pdfium-render =0.9.4` with only `pdfium_7881`. The runtime is acquired separately as described below; no installer is produced by this task.
- URW Base35 fonts, if required for a fallback, belong under `assets/fonts/urw-base35/` and must carry a license that `cargo deny` accepts. The directory currently holds only a README.
- The app does not embed a telemetry endpoint. Settings stay in a local TOML file.

Settings default to `%APPDATA%\ferrostage\settings.toml` on Windows,
`$XDG_CONFIG_HOME/ferrostage/settings.toml` (or `~/.config/ferrostage/settings.toml`)
on Linux, and `~/Library/Application Support/ferrostage/settings.toml` on macOS.
`ferrostage --config <path>` overrides the startup path. Startup fills missing
sections in writable configurations; `ferrostage check-config <path>` only reads
that path and never creates backups or writes defaults. Invalid syntax or an I/O
failure exits with code 1; invalid individual values produce warnings and exit 0.

Saving preserves unknown TOML values and replaces only the configured settings
path after syncing and closing a unique sibling `<config-file-name>.tmp-<pid>-<seq>`.
Malformed files are backed up to `<config-file-name>.broken-<timestamp>-<sequence>`
before writes are allowed; backup failure or an unsupported future schema makes
the session read-only. These operations never write the opened PDF.

## PDFium runtime

`docs/pdfium-runtime.json` locks the Windows x64 151.0.7881.0 archive URL, archive
SHA256, DLL SHA256, and all 16 notice files (packaging LICENSE plus 15 component
notices). `scripts/fetch-pdfium.ps1 -Destination <cache-directory>` downloads only
that artifact and verifies every hash. The application never downloads it.
Distribute `bin/pdfium.dll`, the packaging `LICENSE`, and the complete unmodified
`licenses/` directory beside the application; do not ship the SDK headers/static
libraries or build tools. Retain the following distribution acknowledgments:

- This software is based in part on the work of the FreeType Team.
- This software is based in part on the work of the Independent JPEG Group.

PDFium is BSD-3-Clause with included Apache notices. The native notice inventory
also covers Apache-2.0 (Abseil), MIT (fast_float, lcms, simdutf), FTL (FreeType),
Unicode and third-party notices (ICU), BSD/IJG/Zlib (JPEG), BSD-2-Clause (OpenJPEG),
Libpng and legacy notices, libtiff's permissive notice, AGG's permissive notice,
Apache with LLVM exception plus legacy notices, and Zlib. ICU's combined notice
mentions GPL `aclocal.m4`/`config.guess` build scripts with explicit distribution
exceptions; those scripts are not shipped or linked as runtime code. The entire
notice remains intact. This native audit is separate from `cargo deny`, which
checks Rust dependencies; its allowlist has not been broadened.

Pass the explicit DLL file to `RenderHandle::spawn(RuntimeConfig { ... })`.
Relative paths are canonicalized before loading. `PDFIUM_DYNAMIC_LIB_PATH` is
an upstream build/link variable, not FerroStage runtime configuration. The
first successful library/font configuration persists for the process lifetime;
concurrent sessions or later configuration changes return typed errors. Missing
DLLs fail cleanly, and a failed initialization permits a corrected retry.

No fallback fonts are bundled. `font_directories` may provide existing directories
at initialization; the pinned native tests instead use original embedded Type3
glyphs, independent of host fonts. Fixture generation is a standalone Python
developer script and has no product or user-PDF write path.

## Native shell development build

Build with `cargo build -p ferrostage-app --features gpui,pdfium --locked`.
Launch `ferrostage --config <isolated-settings.toml> --pdfium-library <pdfium.dll> <pdf>`.
The PDF argument is optional; the library defaults to `pdfium.dll` beside the
executable. A missing runtime leaves the shell available with an import error.
The shell reads metadata on the single PDFium worker; task07 displays page
placeholders and does not yet render the presenter workbench.

`vendor/README.md` and each maintained package's original notices document the
approved GPUI0.2.2 source from Zed v1.23.3 and the bounded pulp, metal, cosmic-text
and HarfRust repairs. Original OFL fonts under `vendor/fixtures` are regression
inputs, with their complete licenses; they are not runtime fallback assets.
Only the three exact package/version/license exceptions in ADR-005 are applied.
Full dependency gates and native notice requirements remain in force. This task
does not build an installer or grant distribution approval.

For local native tests set `FERROSTAGE_TEST_PDFIUM` to the verified DLL file and
run `just test-native` and `just clippy-native`. This variable belongs only to the
test harness. CI separately checks default pure-Rust tests and real Windows native
fixtures, including a separate missing-library process and a deliberately failing
normal fixture run. Performance runs use `cargo run --release -p ferrostage-render
--example poc2 --features pdfium -- <dll-file> <report.json>`; they report 20 warmups
and every one of 200 uncached worker renders per case. Hardware thresholds are
reported separately from hosted-CI correctness and do not imply GPU acceptance.
