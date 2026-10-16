# Evidence for the two launch failures

Captured 2026-10-15 local time, before any product edit. Repository `HEAD` was `da04e25`. Working tree was clean.

## just dev

Screenshot text is `PDF 导入不可用：PDFium library could not be loaded from the configured path`.

- Locale prefix: `assets/locales/zh-CN/app.toml:33` `import-unavailable = "PDF 导入不可用：{error}"`.
- Error variant: `crates/ferrostage-pdf/src/raster.rs:27-28`.
- Display site when the worker failed to start: `crates/ferrostage-ui/src/shell.rs:645-647`.
- Default path: `crates/ferrostage-app/src/main.rs:116-125` uses `current_exe().with_file_name("pdfium.dll")`.
- Failure inside the worker: `crates/ferrostage-pdf/src/native.rs:27-30` (`canonicalize`) and `:56-57` (`Pdfium::bind_to_library`). Both map to `LibraryUnavailable`.
- Recipe: `justfile:47-48` is `cargo run -p ferrostage-app --features gpui,pdfium` with no library argument and no copy step.

`target\debug\ferrostage.exe`:

- 42,918,912 bytes, last write 2026-10-15 21:38:25.
- PE32+ magic `0x020B`, subsystem 3 `WINDOWS_CUI`.
- UTF-8 contains `PDFium library could not be loaded from the configured path` and `starting FerroStage`.
- UTF-8 does not contain `presenter UI is not linked yet`.

`target\debug\pdfium.dll` was absent. `cache\pdfium-7881-win-x64\bin\pdfium.dll` was present, 7,211,520 bytes, SHA256 `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`, matching `docs/pdfium-runtime.json` `dll_sha256`.

The debug GUI in the screenshot therefore had the PDFium bindings. The native DLL was not placed beside that executable.

## just build

Recipe: `justfile:42-44` is `cargo build --workspace --release` with default features. `crates/ferrostage-app/Cargo.toml:15-17` leaves `gpui` and `pdfium` off unless requested. `crates/ferrostage-app/src/main.rs:92-99` then prints the skeleton line and returns success.

`target\release\ferrostage.exe`:

- 2,009,088 bytes, last write 2026-10-15 22:16:59.
- PE32+ subsystem 3 `WINDOWS_CUI`.
- UTF-8 contains `presenter UI is not linked yet`.
- UTF-8 does not contain `PDFium library could not be loaded from the configured path`.

Launched with an isolated `--config` under `%TEMP%` so the user settings file was not touched:

- exit code 0
- stdout included `starting FerroStage` and `ferrostage 0.1.0 skeleton — presenter UI is not linked yet (GPUI pin v1.23.3)`
- the temporary settings file was created by startup fill and then deleted

`target\release\pdfium.dll` was absent. Explorer allocates a console for subsystem 3. This process exits immediately, so the console only flashes.

## Same-shaped gap

`justfile:51-52` `just install` is `cargo install --path crates/ferrostage-app --locked` and also omits `gpui,pdfium`. It was not launched in this diagnosis.
