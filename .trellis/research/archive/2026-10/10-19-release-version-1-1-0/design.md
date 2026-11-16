# Single-source version design

Cargo.toml workspace package version currently reads 1.0.0; all ten project crates inherit it. Change that authority to 1.1.0, regenerate Cargo metadata/lock entries through Cargo without broad dependency upgrades, and inspect that only local package version records changed. Do not edit maintained vendor package versions or settings schema_version.

The distribution script obtains ferrostage-app's version through cargo metadata and passes APP_VERSION to NSIS. Reuse that route so inventory, receipt, ZIP/Setup names, Windows VIProductVersion and Installed Apps registration agree. No source-code hardcoded string sweep; distinguish historical docs/examples/dependency versions from active app metadata.

Build/package after the bump, resolve Cargo's actual target_directory and use the normal distribution pipeline. A current installed 1.0.0 app is not changed by merely building 1.1.0. Report local artifacts separately from installation. Roll back only the local workspace/lock version hunk if a version step fails; do not revert completed functionality or unrelated changes.
