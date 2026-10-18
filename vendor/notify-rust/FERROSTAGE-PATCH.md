# FerroStage maintenance: notify-rust 4.18.2

Source: the official crates.io `notify-rust` 4.18.2 archive, upstream commit
`7841a687bea2212396a0c269ef1092b78ef3a270`. Its MIT/Apache-2.0 licenses and all
upstream notices/examples/tests remain. `ferrostage-provenance.json` records the
archive and complete source hashes. `Cargo.toml.orig` is unchanged historical
upstream source; `Cargo.toml` is the executable standalone manifest.

The approved Linux migration uses maintained zbus 6 with proxy/service support,
its wire/Variant API for notification hints and deferred-error match-rule
builders. The public `async` runtime maps to `default-rt`; Tokio and the alternate
dbus backend remain available. The existing synchronous notification API still
uses zbus's supported block_on helper. Notification payloads, handles, actions,
signal matching and desktop bus names retain their existing behavior.

Linux cross-target `cargo check --lib` with the default `z` and alternative
`--no-default-features --features z-with-tokio` backends passed on
2026-10-18. Native desktop notification/action delivery remains a separate Linux
runtime qualification. No backend is disabled and no advisory waiver is added.
The root product lock governs product builds; the archive's standalone lock and
Cargo's extraction marker are omitted.
