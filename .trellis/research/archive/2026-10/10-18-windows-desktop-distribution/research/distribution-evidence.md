# Distribution evidence, 2026-10-18

## Repository and machine

- `justfile:44`: release workspace prebuild, then featured app and PDFium staging.
- `justfile:58` / `justfile:60`: Cargo install plus DLL-only staging; complete notices are skipped.
- `scripts/stage-pdfium.ps1:49`, `:68`, `:73`: Cargo bin selection, notice gate and per-notice hashes.
- `crates/ferrostage-app/src/main.rs:7`: existing release GUI subsystem. `crates/ferrostage-app/build.rs:17`: embedded icon; reuse `assets/icons/ferrostage.ico`.
- `docs/pdfium-runtime.json`: pinned Windows x64 DLL and sixteen notice files. Preserve hashes and layout.
- `docs/packaging.md:3`, `README.md:41`, Chinese README and contributor guidance describe old commands. `.github/workflows/ci.yml:1` contains CI only.
- Branch `dev`, initially clean. Global WiX 7.0.0 exists; NSIS was not found on PATH. No compiler has been acquired or terms accepted.
- Cargo metadata identified `ferrostage-app 1.0.0` with `gpui,pdfium` from this checkout. No FerroStage process was running. Authorized Cargo uninstall succeeded; separately staged PDFium matched its pinned hash before removal. `../legacy-uninstall.json` is the receipt. Settings were untouched.
- Prior target-cache repair establishes build-before-stage ordering: directory creation alone may appear successful through a dangling mbx-managed target symlink. Current dev recipe already builds first; preserve that property.

## Primary references

- [NSIS manual](https://nsis.sourceforge.io/Docs/) and [scripting reference](https://nsis.sourceforge.io/Docs/Chapter4.html): Setup/uninstall, shell folders, shortcuts, registry and return codes.
- [RequestExecutionLevel](https://nsis.sourceforge.io/Reference/RequestExecutionLevel): `user` runs with normal privileges; combine with current-user folders/HKCU.
- [NSIS license](https://nsis.sourceforge.io/Docs/AppendixI.html): main code/zlib use zlib/libpng; LZMA has separate terms. Use zlib and preserve applicable notices.
- [Latest-version page](https://nsis.sourceforge.io/What_is_the_latest_version_of_NSIS) reports 3.13; [download snapshot](https://nsis.sourceforge.io/Download) reports 3.12. Verify a real stable archive and pin SHA256 during implementation acquisition.
- [WiX EULA/maintenance conditions](https://docs.firegiant.com/wix/osmf/): WiX 7 enforces EULA acceptance. Selecting NSIS avoids introducing this condition for Setup generation.
- [Microsoft software installation guidance](https://learn.microsoft.com/en-us/powershell/scripting/samples/working-with-software-installations): use uninstall registration rather than Win32_Product enumeration, which can trigger repair checks.

## Implementation evidence

- User approved the final plan and local installation on 2026-10-18; task started with status `in_progress`.
- NSIS 3.13 ZIP acquired from `https://github.com/tauri-apps/binary-releases/releases/download/nsis-3.13/nsis-3.13.zip`; 2,363,925 bytes; SHA256 `ba63dffc4410ee89193e1cb5a41989991bd77c61068da17e3156d136b7b0b3d8`. Compared against the official `https://nsis-dev.github.io/release-data/versions.json` 3.13 stable record. SourceForge mirror responses were HTML landing pages and were rejected as non-ZIP, not cached as a usable compiler. The verified mirror archive matches the original release hash.
- Extracted `makensis.exe /VERSION` returned `v3.13`. Original `COPYING` SHA256 is `388357c1215ff403c5ebde3a5ecd273e68f8b79a579996775245d1ee65442aba`. Pin is `packaging/windows/nsis-toolchain.json`; compiler remains under ignored cache.
- Root `LICENSE` referenced absent MIT and Apache texts. Added the complete application texts to `LICENSE-MIT` and `LICENSE-APACHE` for distribution, retaining the existing dual-license declaration.
- Baseline/current-source `just ci` passed (exit 0); `just check-shell` passed build, native strict Clippy and 166 headless UI tests. No Rust product source changes were needed.
- MSVC `dumpbin /dependents` on the release application identified `VCRUNTIME140.dll` plus Windows DLLs. PDFium imports Windows DLLs only. Document Microsoft VC++ x64 Redistributable as a prerequisite and check it in Setup; no unverified DLL copying from the development machine.
- The `computer-use` skill and all required API/guidance/confirmation files were loaded. `@oai/sky` initialized and returned real desktop app/window objects. No direct PowerShell UI Automation is used.

## Decision and current limits

NSIS Setup.exe, current-user install, desktop/Start shortcuts, native uninstall and one shared verified payload. Implementation is underway after explicit approval. Final package/lifecycle/native acceptance receipts will establish completion; tool acquisition or baseline CI alone does not establish installer acceptance.
