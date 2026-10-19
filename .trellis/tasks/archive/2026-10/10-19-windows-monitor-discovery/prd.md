# Windows monitor discovery and recovery

## Goal
Resolve the installed Windows application's monitor discovery error (`device and geometry connection sets disagree`) and give Chinese-speaking users actionable recovery guidance in Settings > Displays.

## Requirements
- R1: Establish the cause using current native evidence before changing discovery. Preserve complete topology, stable MonitorId bindings and immediate projection shutdown on real unavailability.
- R2: Fix the proven discovery defect without stale-monitor fallbacks, ignored missing screens, unsafe code or new dependencies.
- R3: Localize platform monitor errors in Chinese and English; clear a recovered monitor error without hiding unrelated errors.
- R4: Both display configuration surfaces must explain recovery using the existing Detect again action, Windows display settings when needed, and explicit role reassignment/Apply.

## Acceptance criteria
- AC1 (R1/R2): A deterministic failure-to-recovery regression fails before the fix and passes afterward. Discovery distinguishes unstable before/after device snapshots from actual device/geometry mismatch; native checks retain all three currently connected monitors without GDI resource growth.
- AC2 (R3): Initial and watcher errors are localized; recovery removes only the corresponding monitor error. Unavailable topology still disables projection.
- AC3 (R4): Settings > Displays and the standalone display panel show localized, actionable recovery steps and the existing retry control.
- AC4: Relevant regression tests, workspace CI and native product build/lint pass; actual native checks and unavailable automation are reported distinctly.

## Scope and constraints
Platform enumeration, presenter monitor error lifecycle, locale catalogs, shared display panel and narrowly related documentation/tests. No identity/schema/role policy changes, unrelated UI redesign, GPU/size matrix, commit or publication. User explicitly requested direct implementation in the same request; proceed through planning without another authorization loop.

## Confirmed evidence
- The platform probe currently enumerates all three monitors in ten consecutive samples.
- `windows.rs::geometry` uses a desktop-window DC; completeness is checked by `combine`.
- `Shell::topology_changed` records `Unavailable` using raw English Display text and does not clear that message on successful `Changed`.
- Top-level and desktop-child coverage and DPI contexts -1/-2/-3/-4 did not reproduce missing monitors. The original transient trigger cannot be reconstructed from the screenshot. No speculative DC/API replacement is authorized by the evidence.
