# Final release-metadata steps

1. Confirm functional child acceptance and current clean/scoped work state. Read backend windows-distribution spec. Inspect Cargo metadata and all inheriting workspace manifests.
2. Set workspace version to 1.1.0. Use Cargo to refresh the lockfile without upgrading external dependencies, then inspect the exact package-name/version diff. Do not mass-replace every 1.0.0 string.
3. Verify `cargo metadata --no-deps --format-version 1 --locked` returns 1.1.0 for all FerroStage packages. Run `just ci` and `just check-shell` against the changed version.
4. Run `just release`; inspect the resolved target_directory/dist output, inventory/receipt versions, Setup/ZIP names, Windows binary version metadata where available and SHA256 verification. Missing signing/clean-machine/installer-runtime coverage stays explicitly NOT_RUN.
5. Report the local 1.1.0 artifacts and relevant evidence. Do not install or publish without a user request. Update live version documentation only where it actually represents the current release; retain historical records.
