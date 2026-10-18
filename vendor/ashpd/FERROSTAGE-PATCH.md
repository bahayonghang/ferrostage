# FerroStage maintenance: ashpd 0.13.13

Source: the official crates.io `ashpd` 0.13.13 archive, upstream commit
`a849791fb9e48efbd32a9c980eb769e1250162a2` (`client` subdirectory). The original
MIT license, notices, examples and optional features are retained. See
`ferrostage-provenance.json` for archive and complete source hashes.
`Cargo.toml.orig` is the unchanged upstream workspace manifest, retained as source
provenance; the executable standalone manifest is `Cargo.toml`.

The approved Linux dependency migration uses zbus 6's maintained wire, names,
derive, connection and service APIs. It does not rename or repackage zvariant.
The public runtime feature `async-io` enables zbus's replacement `default-rt`;
`tokio` remains available. The minimum dependency Rust version is now 1.89.

Changes are limited to wire paths/Variant names, deferred-error builders, owned
property deserialization, checked enumflags2 integer adapters, zbus 6's native
RequestNameFlags bitflags type and the new object-registration return type.
`backend::Builder::with_flags` therefore accepts zbus 6's RequestNameFlags
directly. Other public portal flag arguments, fields and results retain their
existing enumflags2 types. Dictionary keys retain their basic string signature.
The portal response-result expressions now propagate failures before reading a
session handle. No portal is disabled and no native library requirement removed.

`flags.rs` retains exact unsigned 32-bit D-Bus representations. Checked parsing
rejects unknown enumflag bits, matching the former zvariant and enumflags2 serde
behavior; optional fields still omit `None`. The standalone `tests/wire-compat`
crate imports this production module and has its own minimal test lock. Run:

```text
cargo test --manifest-path vendor/ashpd/tests/wire-compat/Cargo.toml --locked
```

All three wire compatibility tests passed on Windows on 2026-10-18. Linux
cross-target `cargo check --lib` passed for the exact GPUI X11 feature set
`async-io,inhibit,notification,open_uri,file_chooser,settings,trash`, the complete
`async-io,frontend,backend` aggregate and `tokio,frontend,backend,tracing` aggregate.
The standalone Wayland check needs a Linux sysroot/pkg-config and was not
completed here; GTK/GLib/PipeWire native feature builds and live portal service
behavior are separate Linux qualification checks, not implied by compilation.

The root product lock is authoritative for product builds; the original
standalone archive lock and Cargo's registry extraction marker are omitted.
