# ADR-001: Pin GPUI to a Zed tag

## Status

Accepted for the skeleton. The dependency is not linked yet.

## Context

crates.io `gpui` 0.2.x is not the Zed UI the plan builds on. GPUI Kit 0.7.3 (published 2026-10-12) vendors `gpui-pre` 0.3.9. The matching Zed release is tag `v1.23.3`.

## Decision

Record the pin as Zed `v1.23.3` and `gpui-kit` 0.7.3. Do not add either crate to the default build until a task actually opens a window. A hard dependency clones native GPU backends and makes `cargo test` too slow for the bootstrap.

`ferrostage-core` must never depend on GPUI.

## Consequences

`ferrostage-ui` exports `ZED_GPUI_TAG` so the pin is tested. The first UI task adds the real dependency and updates this ADR if the tag moves.
