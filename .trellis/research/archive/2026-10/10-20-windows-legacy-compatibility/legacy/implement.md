# Execution plan

## Before execution

- [x] Diagnose screenshot and inspect source/pinned dependency/native import evidence.
- [x] Record user's removal of Windows 7 from scope.
- [x] Converge PRD, design and execution plan; curate implement/check context.
- [x] Present final summary including Win10 1903+ minimum and native qualification limits; receive subsequent approval. User approved implementation and a new installer after validation.
- [x] Run `task.py start` only after that approval.

## Implementation

1. Dispatch `trellis-implement` for installer guard, executable policy regression and matching README/project-plan/packaging documentation. Worker owns only these product/test files; it must preserve other agents' edits. Load before-dev guidelines using native injection or child-side fallback.
2. Use pinned NSIS WinVer macros for Windows major version plus build 18362; keep independent x64 and CRT checks. Add the smallest harness that exercises the same production predicate without mutating an installation.
3. Run boundary matrix and isolated red comparison against original Win11 guard. Verify exit propagation and actual positive acceptance, not only absent error text.
4. Main session updates the distribution spec and task evidence to match verified behavior.
5. Main session increments workspace patch version from 1.1.0 to 1.1.1 and refreshes only workspace lockfile package versions so the new installer is distinguishable from the reported artifact.

## Validation

1. `just release`: compile actual release graph, NSIS `/WX`, Setup, portable ZIP and checksums, resolving Cargo's real target directory.
2. `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/windows-distribution.ps1 -Action verify`.
3. `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-windows-distribution.ps1` including new OS-policy cases. Do not use local `-Lifecycle` because it replaces the user's installed application.
4. `just ci`; investigate failures without adding waivers or widening unrelated changes.
5. Observe rebuilt Win11 executable launch and actual PDF rendering with computer-use tooling; capture result/process status. Mark missing UI capability or missing Win10 host NOT_RUN.
6. Dispatch `trellis-check` over the full changed scope and verify its findings. Re-run affected checks only after new changes.
7. Verify diff and new planning files for whitespace and scope; report artifact links, checks and Win10 qualification checklist. Subsequent user authorization covers repack, local install, local commit and archive; no push/publication.

## Completion boundary

The code fix can be reported as implemented with qualification caveats after local checks. Full Windows 10 runtime qualification remains open until executed on a suitable host; it must never be marked PASS from policy fixtures or Win11 smoke.

## Execution receipt

- Implementation and independent Trellis check complete; reviewer corrected only synthetic NT Product value in the test harness and reran all OS cases successfully.
- `just ci`, native strict Clippy, all-feature dependency gate, `just release`, release verify and full distribution regressions passed. Version 1.1.1 artifacts and hashes are in `research/validation.md`.
- Native launch and PDF-ready state observed on Win11. Visual rendering acceptance NOT_RUN due Computer Use inventory limitation; Win10 actual acceptance NOT_RUN due unavailable host. AC5 remains partial.
- Root updated distribution spec and approval metadata. User subsequently requested repack, local install, local commit and archive. Repack and installation receipts are complete; final commit/archive follow the verified task scope. No push/publication.
