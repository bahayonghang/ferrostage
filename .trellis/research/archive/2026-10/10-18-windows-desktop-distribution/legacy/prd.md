# Windows desktop build, release and installation

## Goal

Users double-click a Setup executable to install FerroStage, then open the application from its desktop shortcut, Start menu, or Windows Search without a terminal or Rust toolchain.

## Background

The user requested build/release/install improvements, Windows registration, desktop/search integration, and prior-version removal. The follow-up explicitly requires a Setup program. Current-user installation was proposed and the user agreed while confirming Setup.

- `justfile:44` prebuilds workspace members before the featured desktop app; `justfile:58` uses Cargo installation and `justfile:60` stages only the DLL, omitting notices. No release recipe exists.
- Release GUI subsystem and embedded icon already work: `crates/ferrostage-app/src/main.rs:7` and `crates/ferrostage-app/build.rs:17`.
- Complete runtime notice verification exists at `scripts/stage-pdfium.ps1:73`. `docs/packaging.md:3` documents that no installer is produced. `.github/workflows/ci.yml:1` defines CI only.
- The user-authorized legacy uninstall is complete; `legacy-uninstall.json` records removal of Cargo registration, executable and matching pinned DLL, and no remaining PATH match. Settings were preserved.

## Requirements

| ID | Outcome |
| --- | --- |
| R1 | Versioned Windows x64 Setup.exe with installation and uninstallation UI; end users need no Cargo, installer compiler or terminal commands. |
| R2 | Current-user installation under `%LOCALAPPDATA%\Programs\FerroStage`, without elevation. Desktop and Start menu shortcuts use the existing icon and enable Windows Search discovery. |
| R3 | One current-user Installed apps entry with version, icon, location and native uninstall command. Support same-version reinstall and forward upgrade; reject downgrade. Remove owned installation files/shortcuts/registration on uninstall; preserve settings, user PDFs and unrelated files. |
| R4 | `just build` builds/stages the desktop application; `just release` generates Setup, portable ZIP and SHA256 checksums from the same payload; `just install` invokes Setup silently, waits and verifies without any manual clicks; `just uninstall` invokes the registered uninstaller. Double-click Setup retains the interactive wizard. Remove Cargo-bin installation. |
| R5 | Ship pinned PDFium DLL, all original PDFium notices, FerroStage license and applicable installer notices without filename collisions. Exclude SDKs, caches, build tools and unrelated/stale products. |
| R6 | Resolve Cargo output including target-directory overrides/cache symlinks; build before staging, reuse incremental artifacts, propagate failures and expose stage/progress output. |
| R7 | Update both READMEs, packaging docs and contributor commands. Add Windows packaging CI with downloadable artifacts, without public release/tag publishing. |

## Acceptance criteria

- [x] AC1 (R1,R4): Produce real `FerroStage-<version>-windows-x64-Setup.exe`, portable ZIP and valid SHA256 manifest. `just install` installs without any manual clicks. All commands consume the same verified payload; failed build/runtime validation cannot yield a successful release receipt.
- [ ] AC2 (R2): Setup installs on this Windows machine without elevation. Desktop/Start shortcuts have the correct target/icon and open a real GUI without a console. Record Windows Search discovery and normal-window PDF-open evidence separately from compilation.
- [x] AC3 (R3): Correct single Installed apps entry and quoted uninstall command. Reinstall/upgrade preserve settings and valid shortcuts, downgrade is refused. Locked files/running app prompt closure or fail, without process killing or false success.
- [x] AC4 (R3,R4): Native uninstall and `just uninstall` remove owned files/shortcuts/registration and preserve settings/PDF canaries and unrelated files. Reinstall the real final version at the end. Cargo bin stays clear.
- [x] AC5 (R5): ZIP and installed inventories match approved payload; every DLL/notice hash passes, application and runtime licenses remain distinct and intact.
- [x] AC6 (R6): Check target-directory override/cache-staging behavior and failure propagation. Unchanged builds retain Cargo caching; no build-speed claim without timing evidence.
- [x] AC7 (R7): Documentation matches commands. Meaningful packaging/lifecycle regressions, `just ci`, relevant native build/lint gates and `git diff --check` pass. Distinguish local checks from unrun hosted CI/native UI acceptance.

## Scope

Windows 11 x64; preserve the existing `gpui,pdfium` product behavior. Default-feature CI stays independent of native runtime downloads. The optional media helper/runtime remains separately prepared; this task does not silently enable media or expand its distribution.

No default-PDF takeover, Explorer context menus, auto-start, updater, PATH modification, all-users installation, macOS/Linux installer, signing-certificate purchase or public publishing. Document unsigned-installer behavior and Windows/VC prerequisites honestly; local success is not clean-machine qualification. No GPUI/vendor migration, policy waivers or unrelated UI work.

## Planning status

PRD converged with the explicit Setup request. Design, execution checklist, research and context manifests accompany it. Final-summary approval is required before task start and product edits.

## Acceptance status

Implementation and final silent local installation passed scoped validation. AC2 is partial: desktop/Start shortcut metadata, shell discovery, native GUI launch and real PDF rendering passed; actual Windows Search UI remains NOT_RUN due to native helper targeting limits. See `research/validation.md` for evidence and qualification boundaries.
