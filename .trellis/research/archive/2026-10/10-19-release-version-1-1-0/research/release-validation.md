# Version 1.1.0 local release validation

Completed 2026-10-19 after the parent reported functional integration review without blocking findings. Final native UI acceptance is parent-owned and is not inferred from this packaging receipt.

## Scope and version authority

- Changed only workspace package version in `Cargo.toml`, 1.0.0 to 1.1.0, and regenerated `Cargo.lock` through `cargo metadata --offline --format-version 1`.
- Structured comparison with the preserved pre-change lockfile confirms exactly ten FerroStage package `version` changes. Package membership, all external packages and every other local package field are identical. No dependency upgrades.
- `cargo metadata --no-deps --format-version 1 --locked` reports all ten workspace packages at 1.1.0.
- Existing `CARGO_PKG_VERSION`, distribution metadata and NSIS APP_VERSION routes remain the authority. No product logic, vendor, settings schema or external thesis changes in this phase.

## Sequential checks

| Command | Result | Elapsed |
| --- | --- | --- |
| `just ci` | PASS: fmt, strict default Clippy, 223 tests (0 failed/ignored), cargo deny | 28.886s |
| `just check-shell` | PASS: actual GPUI/PDFium build, strict native Clippy, 205 UI tests (0 failed/ignored) | 42.079s |
| `cargo deny --all-features --locked --offline check` | PASS: complete native dependency policy | 4.537s |
| `just release` | PASS: locked release desktop build, verified PDFium/notices, NSIS Setup, portable ZIP and checksums | 32.647s |
| `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/windows-distribution.ps1 -Action verify` | PASS: payload, Setup, portable and checksum consistency | Separate verification |
| Independent artifact/ZIP/SHA256 and executable version probe | PASS: all 22 inventory files and ZIP members match; CLI exit 0 | Separate verification |
| `git diff --check -- Cargo.toml Cargo.lock` | PASS | Final scoped diff |

The previous unrelated Cargo process finished before this sequence. No validation Cargo commands ran concurrently. Desktop release compilation itself took 17.54s; the packaging runner measured its full build stage at 18.1s and NSIS at 3.2s.

## Artifacts

Cargo metadata reports `D:/Documents/Code/Rust/Exp/ferrostage/target`; its target-cache symlink resolves to:

`D:/mbx-cache/targets/v1/b228c6f37bcb27af7d1a2d6821dcfcb308164a0bd33ec7c00848c486151f07e0`

The following files are under that resolved directory's `dist/`:

| File | Bytes | SHA256 |
| --- | ---: | --- |
| `FerroStage-1.1.0-windows-x64-Setup.exe` | 10,467,073 | `f0a5ed434f278ca2a24e5b6aeefb9245708f4190c1d82a8860202655237107ce` |
| `FerroStage-1.1.0-windows-x64-portable.zip` | 10,358,066 | `0af218f23748ab8cce520a2a8eef17121e93847cf97726ddb30e1fe146b43c65` |
| `FerroStage-1.1.0-windows-x64-SHA256SUMS.txt` | 215 | `f583a60656c74834e6fa03dd8e8c9d8e99f6e68e85287eecc3835b53a766df02` |

`dist/release.json` and `dist/payload-1.1.0/inventory.json` both report 1.1.0. The ZIP inventory is identical to the payload inventory; its exact member set contains the 22 inventory files plus `inventory.json`. The distribution verifier also checks approved payload membership and pinned runtime/compiler notices, beyond self-consistency.

`dist/payload-1.1.0/ferrostage.exe --version` returns exactly `ferrostage 1.1.0` with exit code 0. Its SHA256 is `0fcfd2a354afa743b806a059fbb4a2a8bcc37921e3d8db0c0d5a4feec9319ba0`.

Setup Windows resources: fixed numeric FileVersion and ProductVersion both 1.1.0.0, FileVersion string 1.1.0, ProductName `FerroStage Setup`. The NSIS template does not emit a ProductVersion string, so that string is empty. An initial verifier incorrectly expected both strings to be 1.1.0.0; correcting it to inspect fixed numeric fields confirmed the existing packaging contract. No product fix or warning waiver was needed. The application itself has an icon resource but no VERSIONINFO resource; its blank Windows version strings are existing behavior, while the real executable CLI version is verified above.

## Evidence and boundaries

Machine-readable compact receipt: [release-validation.json](release-validation.json).

Full local logs and metadata are retained at `C:/Users/lyh/AppData/Local/Temp/ferrostage-1.1.0-validation-20261019/`: `just-ci.log`, `just-check-shell.log`, `deny-all-features.log`, `just-release.log`, `distribution-verify.log`, `artifact-verification.log`, per-command result JSON, full/workspace Cargo metadata, preserved Cargo baselines and the independent verification script.

NOT_RUN: installation, installer lifecycle, signing, clean-machine qualification and hosted CI. No Setup execution, install, uninstall, Git commit, push, tag, archive or publication occurred. Native behavior and standing readability remain evidenced by the parent task rather than this version-only phase.
