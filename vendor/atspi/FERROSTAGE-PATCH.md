# Maintained Linux D-Bus migration

Source: crates.io atspi 0.29.0, https://crates.io/api/v1/crates/atspi/0.29.0/download. Original Cargo.toml.orig and .cargo_vcs_info.json retain source provenance; upstream licenses are preserved.

User authorized extending maintained Linux sources on 2026-10-18. The local patch migrates zbus 5 / zvariant 5 / zbus_names 4 to maintained zbus 6 wire, names, derives and runtime features. AccessKit shared types and AT-SPI protocol/XML stay on their original versions. No advisory exception or renamed retired implementation is used. See [ADR-011](../../docs/adr/ADR-011-linux-dbus-maintenance.md) for maintenance and acceptance contracts.

The optional `atspi-connection` dependency selects `../atspi-connection` directly. The application enables only `proxies`, so a root patch for the inactive connection package would emit an unused-patch warning. The direct optional path preserves the maintained connection feature without enabling it in the application.

`ferrostage-provenance.json` records the original archive hash, VCS identity, file changes and complete maintained file hashes.
