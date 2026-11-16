# Projection status final review

## Fixed findings

- The new status container had a role and visible text but no accessible name. Added a stable `projection-status` accessibility id and `aria_label(summary)`; location and Restore already use the shared accessible button path. Main-session native UIA acceptance must confirm the emitted name.
- Recovery-latched worker failure prevented public readiness but did not mark the status as an output problem. The failed indicator now includes `reload.failed_worker`; the existing readiness regression asserts it.
- The initial failure copy claimed the previous view was retained for every graphics error. The implementer corrected it to generic Output problem because transition fallback can report an error while successfully drawing its destination. Drawn/pending identities carry the exact facts.
- The newly introduced Shell blank/Curtain rejection could discard phases that Audience had already committed as delivered. A real Audience event subscriber published a cover between Audience delivery and Shell's second deferred callback; after unblank and repaint, the regression failed with no settled receipt for the still-selected identity. Removed only those extra snapshot-stage/blank guards, preserving the existing window, generation, navigation, session, placement, reload and snapshot-identity checks. The regression now proves both status delivery and zero-dwell autoplay liveness after uncovering. Painting while already covered remains blocked at the Audience owner; the blank/Restore regression now exercises that real paint path instead of injecting an impossible covered Shell callback.

## Final verification

- `just ci`: PASS; formatting, strict workspace Clippy, **249 tests** and cargo deny.
- `just check-shell`: PASS; gpui,pdfium product build, strict product Clippy and **257 GPUI tests**.
- Focused projection/related regressions: **11 passed** after the race fix.
- `git diff --check`: PASS.

The first sandbox CI completed formatting/Clippy/tests but could not acquire cargo-deny's advisory cache lock. The complete final CI was rerun with cache access and passed, without policy changes or waivers. Final commands used the existing recipes with NoProfile PowerShell and `CARGO_TARGET_DIR=cache/perf-product-target`. CI ran before check-shell so the final executable is the GPUI product build. Logs: `runs/projection-review/ci-final.log`, `runs/projection-review/check-shell-final.log`, `runs/projection-review/projection-fixed.log`; the failing regression evidence is `runs/projection-review/delivery-race-before.log`.

## Remaining ownership

No open source findings. Main session owns normal-window native acceptance, observed-monitor evidence, status accessibility readback, private/public pixel checks and the real Beamer overlay label integration. Headless tests and an application draw receipt do not establish physical projector delivery. No peripheral display or hardware matrix was run by this review.
