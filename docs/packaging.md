# Packaging

Windows 11 x64 desktop distribution uses one verified payload for the NSIS Setup executable and portable ZIP. Build tools acquire runtimes; the installed application remains offline.

- `just ci` stays on default features and does not require `pdfium.dll`.
- `just dev` first builds `ferrostage-app` with `gpui,pdfium`, stages the verified `pdfium.dll`, `LICENSE`, and `licenses/` into `target/debug`, then runs `cargo run -p ferrostage-app --features gpui,pdfium --locked`. Building first lets Cargo restore a collected target-cache directory before staging into a `target` symlink.
- `just build` builds only the `ferrostage-app` dependency graph with `gpui,pdfium` in locked release mode, stages the verified runtime/notices beside the executable, and assembles the distribution payload. Cargo's actual target directory is resolved, including `CARGO_TARGET_DIR`; the default executable is `target\release\ferrostage.exe`.
- `just release` produces `FerroStage-<version>-windows-x64-Setup.exe`, a portable ZIP, and SHA256 checksums under the actual target directory's `dist`. It does not create a Git tag or publish a GitHub release.
- `just install` builds/packages and silently invokes the same Setup for the current user, waits for it, and verifies the result. No wizard clicks are required. It does not install into Cargo bin.
- `just uninstall` invokes the registered native uninstaller. Settings and user PDFs remain intact.
- The optional `pdfium` feature links `pdfium-render =0.9.4` with only `pdfium_7881`. Runtime acquisition and complete notice verification remain as described below.
- URW Base35 fonts, if required for a fallback, belong under `assets/fonts/urw-base35/` and must carry a license that `cargo deny` accepts. The directory currently holds only a README.
- The app does not embed a telemetry endpoint. Settings stay in a local TOML file.

## Setup and portable use

Double-click Setup and follow the wizard. It installs to `%LOCALAPPDATA%\Programs\FerroStage` without administrator elevation. Desktop and Start menu shortcuts use the application icon; the Start menu entry makes the app discoverable in Windows Search. The installer registers version, icon, location and a native uninstall command in the current user's Installed apps entry. Launching the installed release application does not allocate a console.

Reinstalling the same version and upgrading preserve settings. Older versions are refused. Close FerroStage before an install/update or uninstall: locked-file checks request closure or fail instead of force-terminating a presentation. Uninstall through Windows Settings or `just uninstall` removes owned application files, shortcuts and registration while retaining settings, user PDFs and unrelated files in the installation directory.

The portable ZIP contains the same verified application/runtime files. Extract the complete archive and open `ferrostage.exe`; do not move only the executable away from its DLL and notices. The portable variant does not register shortcuts or an uninstall entry.

The application license declaration and full MIT/Apache texts are packaged separately from the original PDFium root `LICENSE` and `licenses/`. The NSIS notice is also retained. Do not overwrite runtime notices with the application license or omit notices from installed copies.

## Packaging toolchain and prerequisites

`packaging/windows/nsis-toolchain.json` pins NSIS 3.13 and the archive SHA256 published by the NSIS release-data project. The archive is acquired from Tauri's binary mirror and verified against that upstream hash; the isolated cache lives under `cache/nsis-3.13`. No global NSIS installation or WiX EULA acceptance is needed. Setup uses NSIS Modern UI and zlib compression. End users need none of these build tools.

Developer packaging requires the project's pinned Rust/MSVC toolchain, Windows PowerShell and normal download access for an uncached compiler/runtime. Stage after Cargo initializes its output directory; a collected target-cache symlink can otherwise make directory creation misleading. Package an explicit file inventory, never an entire Cargo release directory containing stale or unrelated products. Subprocess, hash and installer failures must propagate as failures.

Setup is unsigned; code-signing and Windows reputation qualification are not part of this task. The x64 application imports `VCRUNTIME140.dll` and requires Microsoft's [Visual C++ x64 Redistributable](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist). PDFium imports Windows system DLLs only. These system prerequisites are not copied from the developer machine into the package. Local native verification and clean-machine qualification are separate results. The Windows distribution CI uploads build artifacts for review and does not publish a public release.

Release filenames are `FerroStage-<version>-windows-x64-Setup.exe`, `FerroStage-<version>-windows-x64-portable.zip`, and `FerroStage-<version>-windows-x64-SHA256SUMS.txt`. The staging payload is `dist/payload-<version>`; `dist/release.json` records `version`, `payload`, `setup`, `portable`, `checksums`, `setup_sha256`, and `portable_sha256`. The paths in that receipt are local absolute paths, not portable download URLs.

Run `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/windows-distribution.ps1 -Action verify` to verify the generated payload/artifacts. `scripts/test-windows-distribution.ps1` runs focused packaging regressions; its explicit `-Lifecycle` mode exercises the real current-user installation and should run only when that local mutation is intended or on a disposable CI runner. Script install/uninstall actions accept `-Silent`; `just install` always supplies it for unattended installation. Double-clicking Setup still opens the wizard.

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
`scripts/stage-pdfium.ps1 -Destination <dir>` reads this manifest and reuses
`cache/pdfium-7881-win-x64` when `bin/pdfium.dll` already matches `dll_sha256`.
It calls `fetch-pdfium.ps1` only when that DLL is missing or the hash differs.
The script copies `pdfium.dll` and checks the destination hash. Without
`-DllOnly`, it also copies `LICENSE` and the whole `licenses/` directory and
checks every notice hash. `-CargoBin` ignores `-Destination` and writes to
`$env:CARGO_HOME\bin`, or `%USERPROFILE%\.cargo\bin` when `CARGO_HOME` is unset.
The staged runtime is the DLL plus, unless `-DllOnly`, the notice files. SDK
headers, static libraries, the archive, and build tools stay in the cache.
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

`just dev` and `just build` enable `gpui,pdfium` and stage the verified DLL beside
the executable. The release executable uses the Windows GUI subsystem, so Explorer
opens `target\release\ferrostage.exe` without allocating a console. `just dev` keeps
the console. `just dev -- check-config <path>` is the terminal settings check. An
interactive release `check-config` has no console. Redirected stdout from a release
`check-config` still prints the parse result, with the same exit code as the debug
binary.

Launch with `ferrostage --config <isolated-settings.toml> <pdf>`. The PDF argument
is optional. The library defaults to `pdfium.dll` beside the executable.
`--pdfium-library <pdfium.dll>` overrides that path. A missing runtime leaves the
shell available with an import error. Video still needs the separate media
preparation below. `just dev`, `just build`, `just release`, and `just install` leave `media` off.
The media-enabled build is `cargo build -p ferrostage-app --features gpui,pdfium,media --locked`.
The single PDFium worker supplies actual current/next/notes rasters to the presenter
workbench. The app includes settings, transactional reload, annotations and shared
public media frames; notes and media sources remain private to the presenter.
Media playback additionally uses the isolated helper and selected runtime below.

`vendor/README.md` and each maintained package's original notices document the
approved GPUI0.2.2 source from Zed v1.23.3 and the bounded pulp, metal, cosmic-text
and HarfRust repairs. Original OFL fonts under `vendor/fixtures` are regression
inputs, with their complete licenses; they are not runtime fallback assets.
Only the three exact package/version/license exceptions in ADR-005 are applied.
Full dependency gates and native notice requirements remain in force. Creating
an installer does not replace dependency checks or native desktop acceptance.

For local native tests set `FERROSTAGE_TEST_PDFIUM` to the verified DLL file and
run `just test-native` and `just clippy-native`. This variable belongs only to the
test harness. CI separately checks default pure-Rust tests and real Windows native
fixtures, including a separate missing-library process and a deliberately failing
normal fixture run. Performance runs use `cargo run --release -p ferrostage-render
--example poc2 --features pdfium -- <dll-file> <report.json>`; they report 20 warmups
and every one of 200 uncached worker renders per case. Hardware thresholds are
reported separately from hosted-CI correctness and do not imply GPU acceptance.

## Display detection recovery

If FerroStage reports that display devices and screen geometry disagree, or that
connections are changing, open **Settings > Displays** (or **Display settings**).
Check display power and cables, wait for switching to settle, then use **Detect
again**. If Windows still misses a screen, open **Windows Settings > System >
Display > Multiple displays > Detect**. For separate presenter and PDF screens,
choose **Extend these displays**, also available through **Win + P > Extend**.

Return to FerroStage, select **Detect again**, select the **Application window display**
and **PDF playback display**, then select **Apply**. Screen numbers belong to
FerroStage and may differ from Windows numbers. Detection failure stops
projection immediately; recovery does not restart the presentation automatically.
If the message says display monitoring could not start, restart FerroStage.
These controls never change the Windows display configuration automatically.

## Media runtime and helper

See ADR-009, ADR-010 and `packaging/media/runtime.json`. Only its49 executable/DLL entries (19 plugins) are distributed, with all17 original notices, source recipes and source URLs. Full upstream wheels include unselected components and must never be copied into the product.

Prepare reproducibly on Windows with MSVC x64 tools and Python available:

```powershell
./scripts/prepare-media-runtime.ps1
./scripts/media-native.ps1 -Action build
./scripts/media-native.ps1 -Action clippy
cargo build -p ferrostage-media --example ipc_stall --features native-tests --locked
$env:FERROSTAGE_TEST_MEDIA_HELPER = (Resolve-Path target/debug/ferrostage-media-helper.exe).Path
$env:FERROSTAGE_TEST_MEDIA_RUNTIME = (Resolve-Path cache/media-runtime).Path
just test-media
cargo build -p ferrostage-app --features gpui,pdfium,media --locked
```

The prepare script downloads only the three pinned archives, verifies complete SHA256/size, extracts only the allowlisted entries, verifies every selected entry, and generates import libraries from those exact DLL exports using existing MSVC tools. `-Offline -Archives <directory>` verifies already acquired archives. Build helpers use SYSTEM_DEPS_*_NO_PKG_CONFIG/LIB/SEARCH_NATIVE and require no installed GStreamer SDK. `-ImportLibraries` overrides the prepared import directory. Default `just ci` and GUI linking have no native media import dependency.

Place `ferrostage-media-helper.exe` beside `ferrostage.exe`; stage the runtime there as `media-runtime/{bin,lib/gstreamer-1.0,libexec,notices,recipes}` using `python scripts/stage-media-runtime.py --archives <verified archive directory> --output <executable-directory>/media-runtime`. The scanner is `libexec/gst-plugin-scanner.exe`, matching the helper environment. Staging rejects unselected DLL/EXE files already in the destination instead of silently including an earlier larger runtime. Include the manifest and source/license mapping. This development preparation is not installer publication or distribution authorization. Preserve dynamically replaceable LGPL libraries: runtime availability checks filenames, not exact installed hashes. Build/package verification still requires pinned originals. VC/Windows system runtimes remain system prerequisites.

The media-enabled GUI does not import GStreamer or GLib. Missing complete/partial media runtime is a localized media-slot failure, leaving PDFs and pure GIF/WebP available. The isolated native helper may fail without DLLs without preventing GUI startup. `scripts/check-media-imports.ps1` verifies the GUI PE import boundary. Runtime missing-library GUI smoke is separate from dependency inspection and background supervisor tests.

OpenH2642.6.0 supplies a software H.264 fallback on machines without a D3D11 video decoder. Its exact two-file closure, BSD notices and source recipe are retained under `packaging/media`; no full wheel or GPL decoder is distributed. Missing required decoders return a media-slot Unsupported error, including when playbin could otherwise continue with audio alone.

Background native tests always use fakesink audio and no OS window; the adversarial ipc_stall helper checks bounded off-UI stop/kill/reap and stderr draining. Product tests use stable `fixtures/media` originals, not active Trellis paths. Hardware audio latency,1080p60 pacing/VRAM/soak and the complete desktop matrix remain deferred by the user's task acceptance policy.
