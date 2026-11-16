# Fix stack overflow opening display settings

## Goal

Diagnose and fix the native Windows stack overflow when opening Settings > Displays, with targeted regression and runtime evidence.

## Requirements

- Opening Settings > Displays must not overflow the native Windows main-thread stack in the normal debug development build.
- Preserve monitor discovery, topology, role selection, pending draft, Apply/Cancel and settings persistence behavior.
- Cover both hosts of the shared display panel and preserve other settings categories.
- Keep the approved GPUI dependencies and ordinary stack limits; do not mask the failure with larger stacks or release-only builds.

## Acceptance Criteria

- [x] Identify the failing call path with reproducible evidence on current source.
- [x] Add a meaningful regression exercising the failing display view boundary.
- [x] Verify the rebuilt native app can open Displays and return to other settings without terminating.
- [x] Relevant GPUI tests, strict lint and default `just ci` pass; record actual coverage separately from native checks.

## Notes

- Keep `prd.md` focused on requirements, constraints, and acceptance criteria.
- Lightweight tasks can remain PRD-only.
- For complex tasks, add `design.md` for technical design and `implement.md` for execution planning before `task.py start`.
