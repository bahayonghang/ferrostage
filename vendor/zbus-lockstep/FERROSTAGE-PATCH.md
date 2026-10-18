# Maintained Linux D-Bus migration

Source: crates.io zbus-lockstep 0.5.2, https://crates.io/api/v1/crates/zbus-lockstep/0.5.2/download. Original Cargo.toml.orig and .cargo_vcs_info.json retain source provenance; upstream licenses are preserved.

User authorized extending maintained Linux sources on 2026-10-18. The local patch migrates zbus 5 / zvariant 5 / zbus_names 4 to maintained zbus 6 wire, names, derives and runtime features. AccessKit shared types and AT-SPI protocol/XML stay on their original versions. No advisory exception or renamed retired implementation is used. See [ADR-011](../../docs/adr/ADR-011-linux-dbus-maintenance.md) for maintenance and acceptance contracts.

The published archive omits its XML regression fixture. Restored unchanged from https://raw.githubusercontent.com/luukvanderduim/zbus-lockstep/9bda5264dfe1dc7eaa7738f839ff977315c31b84/xml/test_definition_file.xml into xml/ for the upstream tests.

`ferrostage-provenance.json` records the original archive hash, VCS identity, file changes and complete maintained file hashes.
