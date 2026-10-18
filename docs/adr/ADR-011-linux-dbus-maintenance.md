# ADR-011: Maintain the Linux D-Bus downstream migration

Status: implementation authorized by the user on 2026-10-18; verification is recorded with the task and CI receipts.

## Decision

The user requested repair of all current remote CI failures, then explicitly chose expanding Linux dependency maintenance over advisory exceptions: “允许扩大维护范围，迁移 Linux 依赖链”. This extends ADR-008's original six-package boundary only for the downstream D-Bus consumers described here. It does not replace the pinned Zed v1.23.3 GPUI family, upgrade shared AccessKit types, change the Windows AccessKit repair, permit a new license exception, or authorize an advisory/lint waiver.

RustSec RUSTSEC-2026-0370/0371/0372 report the standalone zvariant, zbus_names and zvariant_derive packages as unmaintained. Their maintained replacement is zbus 6's wire/name/derive APIs; GVariant storage moves to zgvariant. Current published AccessKit/AT-SPI, desktop portal, keyring and notification consumers still request the old packages, including the latest available releases. The isolated resolver experiment and upstream metadata in the task establish why a lockfile-only update cannot remove the complete chain.

## Bounded source ownership

Keep existing compatible public package versions and vendor the downstream consumers needed by the current pinned GPUI graph: accesskit_unix, accesskit_atspi_common, atspi, atspi-common, atspi-proxies, atspi-connection, zbus-lockstep, zbus-lockstep-macros, ashpd, oo7 and notify-rust. Preserve optional connection/portal feature compatibility as well as the active GPUI feature union; any additional maintained downstream package must be listed in the final source receipt and justified by a concrete dependency/API edge.

Use the original registry release archives, retain their copyright/license and VCS metadata, and record archive SHA256 plus a final source delta inventory. Root `[patch.crates-io]` entries select those maintained paths, except that `atspi` selects its optional `atspi-connection` through a direct sibling path to avoid an unused global patch when only `proxies` is enabled. Do not modify or rename retired packages merely to evade advisory matching, suppress features, narrow the audit target/platform set, or ship a second GPUI/AccessKit family.

## Compatibility contracts

- D-Bus names, object paths, signatures, integer flags, messages, property access and error mapping retain their existing public representation. Adapt removed executor/builder/derive APIs to zbus 6's runtime model; preserve task ownership and cancellation behavior.
- Existing AccessKit 0.24.x shared tree/consumer types remain compatible with the pinned Linux adapter. The separately maintained Windows adapter remains unchanged.
- oo7 persisted GVariant data uses zgvariant, not the D-Bus wire serializer. Existing encrypted golden bytes and old keyring fixtures must remain unchanged and pass serialization tests.
- ashpd/notification public APIs and declared features remain available; encode/decode boundary adapters preserve integer signatures and invalid-flag handling.
- Keep root Cargo.lock reproducible and audit the full feature graph with the existing policy. Isolated upstream unit-test locks are permitted only where used by a deterministic CI test; they do not replace the root audit.

## Verification and acceptance

1. The root all-feature dependency graph contains none of the three retired packages, and `cargo deny --all-features --locked check` passes without an advisory exception or target filter.
2. Windows product compilation, strict lint and existing UI/native suites still pass; the dependency migration must not regress the current shipping surface.
3. Cross-target Rust checks compile the migrated pure-Rust Linux paths. The actual Ubuntu GPUI X11 consumer is separately compiled with `cargo check -p ferrostage-ui --features gpui,pdfium,media,gpui_platform/x11 --locked` and its native build prerequisites.
4. Run deterministic maintained AT-SPI serialization/type tests and oo7's existing GVariant encrypted-byte golden. A zero-test filtered invocation is not proof. Record exact commands and counts in the task/CI evidence.
5. Portal service, notification daemon, desktop accessibility and keyring native integration require an actual Linux session; compilation and format tests do not claim that desktop acceptance.

## Rollback and maintenance cost

This decision adds a local source-maintenance burden until upstream consumers adopt the new D-Bus API. Reverting it restores the prior application source but also restores the strict audit failure; no hidden exception is part of rollback. Track upstream adoption by comparing exact package requirements, replace maintained paths only after graph/API/format/consumer checks pass, and retain original source identity for review. Historical failed runs are not deleted or rewritten.
