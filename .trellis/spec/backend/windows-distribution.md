# Windows desktop distribution

## 1. Scope / Trigger

Applies to just recipes, `scripts/windows-distribution.ps1`, NSIS templates under `packaging/windows`, distribution regression checks and Windows artifact CI. Desktop installation belongs to this boundary, not to the core state machine or a `cargo install` wrapper. Read `docs/packaging.md` for end-user guidance and native prerequisites.

## 2. Signatures

```powershell
./scripts/windows-distribution.ps1 -Action build
./scripts/windows-distribution.ps1 -Action release
./scripts/windows-distribution.ps1 -Action verify
./scripts/windows-distribution.ps1 -Action install [-Silent]
./scripts/windows-distribution.ps1 -Action uninstall [-Silent]
./scripts/test-windows-distribution.ps1 [-Lifecycle]
```

The public just recipes are `build`, `release`, `install`, and `uninstall`. `just install` always passes `-Silent` and requires no clicks; directly double-clicking Setup opens its wizard. Script install/uninstall actions support explicit silent mode. Lifecycle tests mutate the current user's installation and require coordination with other native validation.

## 3. Contracts

- Build the actual `ferrostage-app` graph with `--release --features gpui,pdfium --locked`. Resolve metadata version/target directory and compiler-artifact executable. Honor `CARGO_TARGET_DIR`; run Cargo before runtime staging through a cache-managed target symlink.
- Share an explicit verified payload across Setup, portable ZIP and installation. Never zip the entire release directory. Keep the exact pinned DLL and sixteen original PDFium notices from `docs/pdfium-runtime.json`.
- Preserve PDFium root `LICENSE` and `licenses/`. Package application declaration/MIT/Apache texts as `FERROSTAGE-LICENSE.txt`, `FERROSTAGE-LICENSE-MIT.txt`, and `FERROSTAGE-LICENSE-APACHE.txt`; installer notices live under `installer-licenses/`.
- `inventory.json` schema 1 records version and relative `files` entries with `path`/`sha256`. Verify approved membership and bytes, not merely inventory self-consistency. Unknown installed files are allowed but never become uninstall targets.
- `dist/release.json` records `version`, `payload`, `setup`, `portable`, `checksums`, `setup_sha256`, `portable_sha256`. Paths are local, absolute artifact references. Payload is `dist/payload-<version>`; release names share `FerroStage-<version>-windows-x64-` and end in `Setup.exe`, `portable.zip`, `SHA256SUMS.txt`.
- `packaging/windows/nsis-toolchain.json` pins version, archive URL/hash, extracted directory and notice hashes. Compiler acquisition may use a mirror only if its bytes match the pinned upstream hash. Cache under `cache/nsis-<version>`; use zlib and retain notices. Do not accept WiX EULA as an implicit build step.
- Setup uses normal-user privileges, current-user known shell folders, `%LOCALAPPDATA%\Programs\FerroStage`, and `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\FerroStage`. Desktop/Programs shortcuts point to the installed GUI with its icon; native uninstall commands are quoted.
- Same version permits reinstall; newer replaces owned payload; downgrade is refused. Do not kill an app to obtain file access. Uninstall deletes explicitly owned files/shortcuts/keys, leaves settings/PDFs/unknown files, and removes the install directory only if empty.
- Schema 1 fixes the explicit owned file set. Future filename retirement requires an explicit bounded migration and lifecycle regression, rather than trusting deletion paths from a mutable old inventory. There was no previous Setup layout in the initial Cargo-to-desktop migration.
- No runtime download from the application, media closure expansion, PATH modification, PDF-default takeover, automatic Git tag or public release publication.

## 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Cargo/native command nonzero | Stop with failing stage/status; no successful release receipt |
| Unsupported version | Fail before creating a misleading numeric installer version |
| Compiler/DLL/notice hash mismatch | Reject before packaging/installation |
| Missing/extra payload file or bad checksum | Reject; do not merely trust inventory text |
| Target directory override | Resolve Cargo's real output rather than stale default output |
| Cleanup target outside owned root or unsafe reparse point | Refuse mutation |
| Running app/locked owned file | Request closure/retry or fail silently with nonzero status; do not force-kill |
| Setup fails partway | Do not register a successful new version over incomplete payload; report failure |
| Existing newer version | Refuse downgrade and preserve installed app |
| Uninstall with unknown files/settings | Remove owned payload only; leave unknown data intact |
| App not installed | Uninstall reports explicit no-op |

## 5. Good / Base / Bad Cases

Good: Setup installs current-user files, desktop/Start entries and one uninstall entry; shortcut launch opens the real PDF-capable GUI without a console. Base: unchanged release reuses Cargo caching and preserves notice hashes. Bad: `cargo install` scatters the app and DLL into Cargo bin, or a portable archive silently omits notices.

## 6. Tests Required

Verify missing/corrupt payload, complete notices and distinct licenses, checksum tampering, child-process failure propagation, target override and safe cleanup. On an explicitly controlled machine, verify fresh install, same-version reinstall, forward update, downgrade refusal, lock handling and native uninstall preserving settings/PDF/unknown-file canaries. End local acceptance with the requested real final version installed. Run `just ci`, relevant native build/lint and focused distribution checks.

Native Setup UI, icon/shortcut launch, Search discovery and real PDF-open acceptance are separate from compilation/headless tests. Record unrun UI or hosted CI checks honestly; do not substitute window-size/DPI matrices for core behavior. Inspect system imports and document VC prerequisites/signing limits; local success is not clean-machine qualification.

## 7. Wrong vs Correct

Wrong: recursively delete the registered `InstallLocation`, zip `target/release/*`, then report success based only on the installer process starting.

Correct: validate owned paths, package the verified inventory, wait for Setup's exit, verify installed bytes/shortcuts/registration and delete only owned files during uninstall.
