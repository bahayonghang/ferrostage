# Maintained Linux accessibility migration

Date: 2026-10-18. The user explicitly authorized extending the maintained Linux dependency sources instead of adding advisory exceptions.

## Scope and source identities

The registry versions remain accesskit_unix 0.22.1, accesskit_atspi_common 0.19.1, atspi 0.29.0, atspi-common / atspi-proxies / atspi-connection 0.13.0, and zbus-lockstep / zbus-lockstep-macros 0.5.2. `linux-atspi-source-origins.json` records every original archive URL, archive SHA-256, and upstream VCS identity. Original license files, Cargo.toml.orig and VCS metadata are preserved. The lockstep XML test fixture omitted from its published archive was restored from that exact upstream commit; its URL is recorded in the package patch receipt.

All eight packages use maintained zbus 6.0.0 instead of the retired zvariant / zvariant_derive / zbus_names implementations. zbus_xml is the original registry release 6.0.0, without another local package. AccessKit 0.24.1 and accesskit_consumer 0.38.0 are unchanged, as are the pinned GPUI family and Windows AccessKit package. Root patches also integrate the separately owned ashpd / oo7 / notify-rust migrations. No advisory policy or dependency audit target was narrowed.

## Bounded implementation changes

- Migrate D-Bus wire/name imports, Variant / OwnedVariant types, derive attributes, codec context construction, and merged error conversion implementations.
- Preserve completed message validation: zbus 6 builder setters return the builder and report invalid input at build, so obsolete intermediate `?` / unwrap calls are removed.
- Preserve Qt AT-SPI event compatibility with `deserialize_ignore_signature`, the direct zbus 6 replacement for `deserialize_unchecked`. Existing malformed-data rejection and Qt compatibility tests remain enabled.
- Adapt zbus_xml member iterators without changing XML/signature checks.
- Adapt object-server insertion's returned unused-interface Option to the previous inserted boolean semantics.
- Let maintained zbus own connection tasks. AccessKit's background event loop uses zbus::block_on; the removed per-connection executor/tick adapter and its now-unused dependencies are removed. The optional AT-SPI peer listener uses maintained zruntime task scheduling and retains P2P/default features.
- Keep async-io and Tokio public feature choices. Raise the maintained packages' declared MSRV to 1.89, matching zbus 6, below the project's Rust 1.99.

The atspi-connection patch is intentionally unused in the actual GPUI graph, which requests only atspi's proxies feature. It is retained to keep the vendored atspi default connection/P2P API compatible. Cargo reports that unused patch; the full graph audit still passes.

## Executed validation

| Check | Result | Evidence |
| --- | --- | --- |
| Actual Linux target check of AccessKit and default AT-SPI connection/P2P/proxies | PASS | linux-atspi-default-check.log |
| Same chain with Tokio accessibility feature | PASS | linux-atspi-tokio-check.log |
| Strict Linux clippy for all seven implementation packages | PASS | linux-atspi-clippy.log |
| Existing AT-SPI protocol/serde/wire/XML/Qt regressions | PASS, 1141 tests | atspi-common-tests.log |
| Existing AccessKit translation/filter/cache-event regressions | PASS, 7 tests | accesskit-atspi-tests.log |
| Existing lockstep XML extraction/macros | PASS, 23 tests | zbus-lockstep-tests.log |
| Full product cargo deny --all-features --locked --offline check | PASS | linux-migration-deny.log; advisories, bans, licenses, sources all ok |
| Root cargo fmt --all -- --check | PASS | executed after formatting each maintained source package |
| Planned product feature expression gpui,pdfium,media,gpui_platform/x11 | PASS | cargo metadata --no-deps accepted it |

The Linux target checks use Rust's installed x86_64-unknown-linux-gnu standard library on Windows. They execute rustc against the Linux cfg/dependency path, without linking or executing a Linux binary. They are not Linux runtime accessibility evidence. Unit tests above execute on the Windows host and exercise portable wire/translation code. Actual screen-reader registration, activation, P2P discovery and event delivery need native Linux follow-up. The new Ubuntu product compile CI gate separately verifies the real GPUI consumer.

## Reproduction

The isolated `linux-dependency-check` manifest exists only for migration evidence. Its local relative vendor paths must follow the task's depth if the task is archived. Its checked-in lock keeps the default and Tokio probes reproducible.

```powershell
cargo check --manifest-path <task>/research/linux-dependency-check/Cargo.toml --target x86_64-unknown-linux-gnu --locked
cargo check --manifest-path <task>/research/linux-dependency-check/Cargo.toml --target x86_64-unknown-linux-gnu --no-default-features --features tokio --locked
cargo clippy --manifest-path <task>/research/linux-dependency-check/Cargo.toml --target x86_64-unknown-linux-gnu -p accesskit_unix -p accesskit_atspi_common -p atspi-common -p atspi-proxies -p atspi-connection -p zbus-lockstep -p zbus-lockstep-macros --lib --locked -- -D warnings
cargo test --manifest-path vendor/atspi-common/Cargo.toml --lib --locked --config 'patch.crates-io.zbus-lockstep.path="vendor/zbus-lockstep"' --config 'patch.crates-io.zbus-lockstep-macros.path="vendor/zbus-lockstep-macros"'
cargo test --manifest-path vendor/accesskit_atspi_common/Cargo.toml --lib --locked --config 'patch.crates-io.atspi-common.path="vendor/atspi-common"' --config 'patch.crates-io.zbus-lockstep.path="vendor/zbus-lockstep"' --config 'patch.crates-io.zbus-lockstep-macros.path="vendor/zbus-lockstep-macros"'
cargo test --manifest-path vendor/zbus-lockstep/Cargo.toml --lib --locked
cargo deny --all-features --locked check
```

Standalone package Cargo.lock files are retained for the three test commands. No source, feature, test or audit waiver was added to manufacture a passing gate.
