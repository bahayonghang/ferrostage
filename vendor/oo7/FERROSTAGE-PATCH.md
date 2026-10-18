# FerroStage maintenance: oo7 0.6.0

Source: the official crates.io `oo7` 0.6.0 archive, upstream commit
`9070389f33bec2e47048384e2fdbd7aab64e0df7` (`client` subdirectory). The original
MIT license, fixtures, tests and examples remain. `ferrostage-provenance.json`
records archive and complete source hashes. `Cargo.toml.orig` retains the
original upstream workspace manifest; `Cargo.toml` is the maintained executable
standalone manifest.

The approved migration preserves oo7's public API and existing crypto backends.
D-Bus uses maintained zbus 6 wire/names/derives and proxy/service features.
`async-std` maps the zbus runtime to `default-rt`; Tokio remains available. Builder
errors are returned by build, property object-path arrays use owned paths to meet
zbus 6's DeserializeOwned requirement, and the formerly separate wire error is
now zbus's unified Error.

Keyring file serialization independently uses maintained zgvariant 2, including
the keyring/encrypted-item/unlocked-item/Mac Type derives and little-endian
position-zero context. It does not use D-Bus wire encoding for files. Keyring
headers, version, field order, crypto, attributes and old fixtures are unchanged.
The upstream `file::unlocked_item::tests::serialization` ciphertext golden bytes
are unchanged, so its assertion verifies compatibility of the encoded plaintext
as well as encryption. Run the deterministic test on Linux from the repo root:

```text
cargo test --manifest-path vendor/oo7/Cargo.toml --config 'patch.crates-io.ashpd.path="vendor/ashpd"' --locked --lib file::unlocked_item::tests::serialization
```

The committed standalone lock was generated with exactly that ashpd patch and
contains maintained zbus 6 and zgvariant 2, with no retired zvariant/zbus_names
packages. It serves this test; the root lock governs product builds.

On 2026-10-18 Linux cross-target compilation passed for GPUI's
`--no-default-features --features async-std,native_crypto --lib` and the default
Tokio/native_crypto `--lib --tests --locked`. Windows cannot execute these Linux
binaries; live Secret Service/session/portal interaction and the Linux golden
test execution remain separately reported qualifications. No advisory waiver or
feature removal is part of this patch.
