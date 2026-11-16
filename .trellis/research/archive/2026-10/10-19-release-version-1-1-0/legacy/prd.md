# Version 1.1.0 metadata and local distribution

Implementation approved by the user on 2026-10-19; final integrated/native validation remains pending.

Parent R7. Run last after functional integration. The user requested the application version 1.1.0; this is unrelated to thesis metadata, settings schema or third-party dependency versions.

## Acceptance
- V1: `[workspace.package].version` is 1.1.0 and all ten inheriting FerroStage crates resolve to that version. Cargo.lock local package records agree; no unrelated dependency update.
- V2: `cargo metadata --locked` succeeds and native application/build version consumers agree. Existing dynamic CARGO_PKG_VERSION and packaging metadata remain the source of truth; do not introduce a second hardcoded version.
- V3: After the bump, required CI/native checks pass and `just release` produces local 1.1.0 Setup, portable ZIP and checksums. Payload inventory, receipt, Setup file/product version and filenames agree; Windows four-field version may be 1.1.0.0.
- V4: Do not mistake installer compilation for a completed installation. This scope creates local release artifacts; no install, push, tag, upload or hosted release is required.

## Dependencies
All four functional children and parent integration checks precede version/release finalization. Relevant evidence: Cargo.toml:36, crate manifests with version.workspace, scripts/windows-distribution.ps1:67-72 and packaging/windows/installer.nsi version definitions.
