# ADR-001: Pin GPUI to a Zed tag

## Status

Accepted. Task07 product integration of the explicitly approved maintained tagged source is in progress; native acceptance remains pending.

## Context

crates.io `gpui` 0.2.x is not the Zed UI the plan builds on. GPUI Kit 0.7.3 (published 2026-10-12) vendors `gpui-pre` 0.3.9. The separately allowed Zed release is tag `v1.23.3`; these are not identical upstream revisions. Task07 resolved the tag to `96a1b1b5389efff473fc2306f7f8c690f2e1a417`, while pre0.3.9 records `93d745668805029476e6d7fbb19652116fed51de`.

## Decision

Record the pin as Zed `v1.23.3` and `gpui-kit` 0.7.3. Do not add either crate to the default build until a task actually opens a window. A hard dependency clones native GPU backends and makes `cargo test` too slow for the bootstrap.

`ferrostage-core` must never depend on GPUI.

## Consequences

`ferrostage-ui` exports `ZED_GPUI_TAG` so the pin is tested. The first UI task adds the real dependency and updates this ADR if the tag moves.

Task07's original isolated routes failed policy. The user subsequently approved the final independently reviewed maintenance set in [ADR-008](ADR-008-gpui-dependency-gate.md): base tag v1.23.3 / revision96a1b1b5389efff473fc2306f7f8c690f2e1a417, local GPUI0.2.2/pulp0.22.3/metal0.33.0/cosmic-text0.19.0/harfrust0.12.0 patches, and three exact license exceptions under ADR-005. Product feature wiring and full product audit/build must pass before shell implementation proceeds. No crates.io gpui0.2 route, unrelated source upgrade or lint/advisory waiver is authorized.
