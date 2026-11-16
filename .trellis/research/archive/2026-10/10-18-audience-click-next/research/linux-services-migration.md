# Linux service dependency migration receipt

Date: 2026-10-18. Scope explicitly approved by the user: maintain the Linux
dependency chain rather than add advisory waivers. This receipt covers the three
service packages only; ADR-011 and the AT-SPI receipt cover integration policy
and the other dependency branch.

## Final maintained source

| Package | Version | Upstream files inventoried | Maintained files inventoried | Changed upstream files |
| --- | --- | ---: | ---: | ---: |
| ashpd | 0.13.13 | 82 | 87 | 62 |
| oo7 | 0.6.0 | 53 | 55 | 22 |
| notify-rust | 4.18.2 | 38 | 39 | 4 |

Each package retains original licenses, examples and notices. Its
`ferrostage-provenance.json` records the official archive URL/checksum, upstream
VCS identity and full original/maintained file inventories. All three archive
SHA256 values matched the corresponding original committed crates.io lock
checksums. Text hashes normalize UTF-8 CRLF to LF; binary/archive hashes use raw
bytes. The provenance file itself is not self-hashed. Maintained counts include
the patch description/diff and, for ashpd, the production flag adapter and its
portable test package. The diffs omit the separately recorded oo7 lock delta.

zbus 6 supplies D-Bus wire, names, derives, proxy and service APIs. oo7's keyring
file encoding separately uses zgvariant 2, preserving field order, little-endian
context, headers, version, crypto and original fixture/golden bytes. Portal
enumflags2 APIs retain unsigned 32-bit wire encoding and checked unknown-bit
rejection. The zbus-owned `backend::Builder::with_flags` argument necessarily
follows zbus 6's native RequestNameFlags bitflags type. Other public portal flag
arguments/results retain their enumflags2 API. No functionality, target or audit
scope is disabled and no advisory/lint waiver was added.

The original standalone locks are omitted for ashpd and notify-rust; the root
lock governs product builds. oo7 retains a regenerated lock for its standalone
golden regression with the maintained ashpd patch. The small ashpd wire test
package also retains its own lock. `Cargo.toml.orig` files remain unmodified
upstream historical source, not executable manifests.

## Executed validation

These checks passed during implementation on Windows with Rust 1.99. Linux
checks used `--target x86_64-unknown-linux-gnu` and do not execute Linux binaries.
Independent review subsequently corrected the two source drifts below; the
reviewer's final integrated gates govern final-state qualification.

| Check | Result |
| --- | --- |
| oo7 GPUI configuration: `--no-default-features --features async-std,native_crypto --lib` | PASS, Linux cross-check |
| oo7 default Tokio/native_crypto: `--lib --tests --locked` with maintained ashpd path patch | PASS, Linux cross-check |
| ashpd exact GPUI X11 portal set: `async-io,inhibit,notification,open_uri,file_chooser,settings,trash` | PASS, Linux cross-check |
| ashpd aggregate: `--no-default-features --features async-io,frontend,backend --lib` | PASS, Linux cross-check |
| ashpd alternative aggregate: `--no-default-features --features tokio,frontend,backend,tracing --lib` | PASS, Linux cross-check |
| notify-rust default backend and `--no-default-features --features z-with-tokio --lib` | PASS, Linux cross-check |
| `cargo test --manifest-path vendor/ashpd/tests/wire-compat/Cargo.toml --locked` | PASS, 3 executable tests |
| `cargo clippy --manifest-path vendor/ashpd/tests/wire-compat/Cargo.toml --all-targets --locked -- -D warnings` | PASS |
| Test harness format, complete maintained source hashes and original archive checksums | PASS |

The portable tests import the actual production `flags.rs`. They assert exact
u32 variant bytes, reject unknown bits and wrong signed types, and verify
optional dictionary omission and roundtrip behavior.

## Review corrections and receipt refresh

Independent review restored the unrelated PipeWire example constructor
`pw::spa::pod::Value::Object(obj)` after the mechanical Value migration, and
restored error propagation on the first DBusSecret tuple `serialize_element`.
After the reviewer confirmed both edits complete, only package patch diffs and
provenance JSON were regenerated. The example now matches upstream exactly and
is absent from ashpd's modified-file inventory. All DBusSecret tuple element
serialization calls retain `?`. Product source was not edited during refresh.

## Separate Linux qualification

The existing oo7 serialization test must execute on Linux, using the unchanged
ciphertext golden and this exact locked invocation from the repository root:

```text
cargo test --manifest-path vendor/oo7/Cargo.toml --config 'patch.crates-io.ashpd.path="vendor/ashpd"' --locked --lib file::unlocked_item::tests::serialization
```

The standalone Wayland cross-check stopped at the missing Linux sysroot and
pkg-config setup; it is not a source failure or native pass. GTK/GLib/PipeWire
native feature builds and live portal, Secret Service, keyring session and
desktop notification/action delivery remain Linux qualification gaps. Root
locked all-feature audit and integrated GPUI checks are coordinated separately;
this package receipt does not claim their results or close remote issues.
