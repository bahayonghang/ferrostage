# Implementation notes

Measured 2026-10-15 after the staging script and GUI-subsystem change. The cache DLL already matched `docs/pdfium-runtime.json`, so staging did not download or re-extract. `cache/pdfium-7881-win-x64/pdfium-win-x64.tgz` stayed at 2026-10-14T15:01:11.7711916Z and `bin/pdfium.dll` stayed at 2026-06-08T17:05:46Z.

`target` is a directory symlink to `D:\mbx-cache\targets\v1\b228c6f37bcb27af7d1a2d6821dcfcb308164a0bd33ec7c00848c486151f07e0`. Windows reports the resolved path for a loaded module. File IDs below show that path is the staged file.

## Commands

| Command | Result |
| --- | --- |
| `powershell -NoProfile -File scripts/stage-pdfium.ps1 -Destination target/debug` | Pass. Cache timestamps unchanged. Destination DLL SHA256 `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`. All 16 notice hashes matched. `include/`, `lib/`, the archive, and SDK files were not copied. |
| `cargo test -p ferrostage-app --locked` | Pass. 5 tests in `tests/check_config.rs`, including the new startup-log case. |
| `cargo fmt --all -- --check` | Pass. |
| `cargo clippy -p ferrostage-app -p ferrostage-ui --all-targets --features gpui,pdfium --locked --offline -- -D warnings` | Pass. |
| `just ci` | Pass. Default features. `cargo deny` ended with advisories, bans, licenses, and sources ok. |
| `just build` | Pass on the second run. The first run linked the release app, then staging failed because Just's Windows PowerShell could not see `Get-FileHash` (PowerShell 7's Utility module shadows it). Both `stage-pdfium.ps1` and `fetch-pdfium.ps1` now hash with `System.Security.Cryptography.SHA256`. The rerun staged successfully. A later `fetch-pdfium.ps1` against the existing cache exited 0 without downloading. |

`-DllOnly -CargoBin` was checked with a temporary `CARGO_HOME`, not the real Cargo bin. It wrote only `pdfium.dll`, ignored `-Destination`, and the DLL hash matched. `just install` was not run.

## PE and hashes

| Binary | Subsystem | Size | `presenter UI is not linked yet` |
| --- | --- | --- | --- |
| `target\debug\ferrostage.exe` after `cargo build -p ferrostage-app --features gpui,pdfium` | 3 (`WINDOWS_CUI`), magic `0x020B` | 42,928,640 | absent |
| `target\debug\ferrostage.exe` after `just ci` | 3, magic `0x020B` | 5,149,696 | present (default-feature rebuild) |
| `target\release\ferrostage.exe` after `just build` | 2 (`WINDOWS_GUI`), magic `0x020B` | 17,606,656 | absent |

`target\debug\pdfium.dll` and `target\release\pdfium.dll` both hash to `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`. Release `LICENSE` and `licenses/` match all 16 manifest notice hashes. The release tree does not contain `include/`, `lib/`, the archive, or the SDK files.

The release executable does contain `PDFium library could not be loaded from the configured path`, which is the linked error string.

## check-config

Isolated fixture `schema_version = 1` plus `timer.target_minutes = 0`, stdout and stderr redirected:

- Debug exit 0, release exit 0.
- Both stdout texts contain `target_minutes: 45`.
- The fixture directory has no `startup-error.log`. Neither `target\debug` nor `target\release` gained one.

Release `--config` pointed at a directory (so `open_for_startup` cannot read it): exit 1. `startup-error.log` in that parent contains one line, `ferrostage: cannot read settings: <temp>\settings-dir: Access is denied. (os error 5)`. The process did not stay open.

## Desktop smoke

Release `ferrostage.exe` with an isolated `--config` under `%TEMP%` and `fixtures/pdf/raster.pdf`. Stdout and stderr redirected. After 6 seconds the process was still alive. The log did not contain `PDFium library could not be loaded from the configured path`. The loaded `pdfium.dll` file ID matched `target\release\pdfium.dll` (`0x080000002cc3fb`); Windows showed the resolved cache path. The process was then stopped.

A second launch used `Start-Process` with no redirected stdio and no new console, the same shape as Explorer. After 6 seconds it was still alive without a console held open for it. Its loaded `pdfium.dll` was the same file. It was then stopped.

`Get-Process ferrostage` afterwards was empty. `fixtures/pdf/raster.pdf` and `%APPDATA%\ferrostage\settings.toml` kept their previous timestamps.

The in-window Open PDF control was not clicked. The PDF was passed on the command line, and the worker had the staged DLL loaded.
