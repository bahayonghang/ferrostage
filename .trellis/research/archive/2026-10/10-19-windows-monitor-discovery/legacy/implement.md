# Implementation and validation

1. Preserve baseline/native observations and investigate the two IncompleteTopology branches. DONE: ten healthy probe samples; real GPUI baseline logs show all three monitors/mapping OK; coverage and DPI experiments negative.
2. Add and RUN a regression on the existing UI that observes Unavailable then same-topology Changed and fails on the stale error. Then implement typed monitor-error ownership, locale projection and shared recovery instructions. Cover startup, manual refresh, unrelated error preservation, and projection unavailability.
3. Distinguish changing snapshots from actual incomplete topology, with deterministic identity-change/same-keys tests; no speculative acquisition or identity changes.
4. Run targeted platform/UI tests (including native GDI resource count), `just ci`, actual desktop build/clippy, GPUI tests and all-feature dependency policy. Run the rebuilt monitor probe and a separate-config real desktop smoke; inspect messages/settings recovery using native accessibility if available. Preserve NOT_RUN for unavailable physical/native acceptance.
5. Dispatch independent Trellis check, fix scoped findings and rerun only affected checks. Update specs and task evidence. Do not commit/push/install unless user asks; retain task for review if acceptance is limited.

User requested direct repair and task creation in the same message, overriding an additional skill-only approval loop. Implementation does not require a new scope decision.

## Result
Implementation and coordinated checks are complete. See `research/validation.md` for results, artifacts and limits; `research/independent-check.md` for independent review. Red/green evidence covers the original stale message and sibling Apply/shortcut/stale-draft paths. The full GPUI suite caught and verified repair of a new footer regression without weakening assertions. Setup/portable artifacts are built and verified. User's running installed application remains untouched; source changes are not committed. The original transient OS trigger remains unconfirmed.
