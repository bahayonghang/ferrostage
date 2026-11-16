# Platform classification fix

## Changes

- Added `PlatformError::TopologyChanging { before, after }` for inconsistent device snapshots. Connection lists can remain identical when path, driver name or primary metadata changed.
- Extracted the existing before/after check into `combine_snapshots`, called by production `enumerate`. Native device/geometry acquisition, identities, scale, primary validation and GDI ownership remain unchanged.
- `IncompleteTopology` now exclusively describes actual device/geometry connection-set disagreement. Partial topology is still rejected.
- Added deterministic coverage for identity path, driver name and primary changes with identical keys; missing geometry remains rejected and stable observations retain the original output.
- Kept platform polling free of new logs. UI owns structured diagnostics on distinct failure transitions, avoiding repeated logs every 200 ms.

## Red / green evidence

Before changing the error variant, the extracted production check failed `changing_device_metadata_is_not_a_geometry_mismatch` with:

```text
device snapshot changes must not be reported as missing geometry:
IncompleteTopology { devices: ["DISPLAY-A"], geometry: ["DISPLAY-A"] }
```

After the fix:

- `cargo test -p ferrostage-platform --locked -- --nocapture`: PASS, 31 tests across 3 suites.
- `cargo clippy -p ferrostage-platform --all-targets --locked -- -D warnings`: PASS.
- `rustfmt --edition 2024 --check crates/ferrostage-platform/src/monitor.rs crates/ferrostage-platform/src/windows.rs`: PASS.
- `git diff --check -- crates/ferrostage-platform`: PASS.

The compiled native resource regression was invoked directly to retain unfiltered GDI evidence (concurrent RTK cargo log filenames collided):

```text
real-owner candidate: monitors=3 GDI before=1 after=1
production enumeration64: GDI before=1 after=1
test windows::tests::getdc_v15_real_owner_preserves_topology_and_enumeration_releases_gdi ... ok
```

## Limits

The original screenshot's transient Windows trigger remains unknown. This fix corrects a reproducible classification defect; it does not establish that hardware, clipping or DPI caused that original failure. Native coverage and DPI experiments belong to root-session evidence. No physical hotplug/clone-mode test, installed-app replacement or package publication was performed by this worker.
