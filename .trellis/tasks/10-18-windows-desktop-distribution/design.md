# Windows desktop distribution design

## Integrated command and payload contract

Use one task because all deliverables depend on the same payload and installation identity. A small Windows PowerShell orchestration entry point under `scripts/` serves the just recipes. Reuse existing PDFium acquisition/staging, retain `just dev` behavior and default `just ci`. Check native exit codes immediately; expose stage start/pass/fail and elapsed time, with native output or heartbeat during long work.

| Command | Contract |
| --- | --- |
| `just build` | Locked release build of the app with `gpui,pdfium`; stage verified runtime/notices beside its executable and assemble clean distribution payload. |
| `just release` | Build once; generate versioned Setup, portable ZIP and verified SHA256 manifest under ignored `target/dist`. No release publication or Git mutation. |
| `just install` | Run the release pipeline, invoke the same Setup silently, wait and verify resulting installation without clicks; return failure accurately. Double-click Setup still opens its wizard. |
| `just uninstall` | Resolve owned current-user registration, run native uninstaller and wait. Missing installation is an explicit no-op. |

Resolve Cargo's actual executable through JSON compiler-artifact output or metadata, including custom target directories. Build the desktop dependency graph rather than unrelated workspace products. Keep `--locked`, normal caching and build-before-stage ordering; never clear the full target tree. Version comes from workspace Cargo metadata. Reject unsupported prerelease versions rather than silently truncating installer comparisons.

The explicit payload contains `ferrostage.exe`, pinned `pdfium.dll`, complete original PDFium `LICENSE`/`licenses/`, separately named FerroStage license and applicable installer notices. Preserve executable-adjacent runtime lookup and all sixteen runtime notice hashes. Never zip the entire Cargo release directory. Generate a relative-path/SHA256 inventory; validate generated absolute paths before cleanup, reject unsafe reparse targets and do not recursively remove user-selected directories.

## Setup technology

Use NSIS Modern UI with zlib compression, reusing `assets/icons/ferrostage.ico`. Pin a verified stable compiler archive, version and SHA256 under ignored cache; end users receive only the standalone Setup. Official NSIS snapshots disagree about latest version (3.12/3.13); verify a real stable archive and pin it before use. This is a technical acquisition check, not an unresolved product choice.

The installed WiX 7 was considered but introduces EULA/maintenance-fee conditions unnecessary for this requirement; do not auto-accept those terms. NSIS's zlib module uses the permissive zlib/libpng license; avoid introducing the LZMA module's separate terms. Retain applicable original installer notices.

## Installation and Windows integration

Author templates under `packaging/windows/`. Use `RequestExecutionLevel user` and current-user shell context. Install to `%LOCALAPPDATA%\Programs\FerroStage`; create `FerroStage.lnk` in actual current-user Desktop/Programs shell folders, including redirected folders. Set installed target, working directory and icon.

Use a stable owned key at `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\FerroStage`: display name/version/icon/location, publisher consistent with project metadata, quoted uninstall and quiet-uninstall strings and installed size. Write a native uninstaller beside the app. Keep registration and file scope aligned; do not modify PATH or PDF defaults.

Uninstall removes only explicitly owned files, shortcuts and keys, then removes an empty installation directory. Preserve `%APPDATA%\ferrostage`, PDFs and unknown files. Never use unrestricted recursive removal of the install root or registry-provided arbitrary paths. Keep application and PDFium root notices distinct to avoid `LICENSE` overwrites.

## Update, failure and rollback

Keep one app identity/location. Equal numeric version permits reinstall, higher version replaces the owned payload, lower version is rejected. Schema 1 uses the fixed, explicit payload file set; the initial migration is from Cargo installation, not a historical Setup layout. A future change that retires owned filenames must add an explicit bounded migration and lifecycle regression before changing this schema. Do not derive deletion authority from arbitrary paths in a mutable old inventory. Preflight the full payload and file locks; ask to close the app interactively or fail in silent mode. Do not kill a presentation. Only register successful version/create shortcuts after payload extraction succeeds. Test interrupted/failed extraction behavior; do not claim transactional rollback without implementing it.

Repository rollback affects only task scripts/templates/recipes/docs. The old Cargo version was intentionally uninstalled and is not restored automatically. Lifecycle acceptance ends by reinstalling the real final package, leaving the user a working application.

## CI and acceptance limits

Add Windows packaging CI using the same entry point and pinned compiler/toolchain, read-only repository permissions, inventory/checksum/lifecycle checks as runner support allows, and artifact upload. No public release publishing or signing secrets. Preserve existing native/default gates.

Local acceptance includes Setup UI, no elevation, shortcut launch, Search discovery and opening a real PDF. If native UI tooling cannot inspect this desktop, record NOT_RUN with a concrete reason; headless success cannot prove GUI acceptance. No size/DPI matrix is required. Inspect Windows/VC runtime prerequisites during implementation; no clean-machine claim based on this development PC. Unsigned Setup reputation prompts and signing remain documented limits. Media remains outside the unchanged `gpui,pdfium` distribution payload.
