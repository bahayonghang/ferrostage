# Windows 10 and later installer compatibility

## Goal

Allow the existing x64 desktop application to install on Windows 10 version 1903 (build 18362) and later, including Windows 11, with an explicit minimum version and truthful native qualification evidence.

## Confirmed facts

- The supplied 1.1.0 Setup screenshot reports `FerroStage requires Windows 11 x64.`
- `packaging/windows/installer.nsi:123-125` rejects systems below Windows 11.
- `docs/PROJECT_PLAN.md:56` defines the original Windows 11 launch baseline; `docs/packaging.md:3` reflects it.
- The release uses one x64 GPUI/PDFium payload. Pinned Zed documentation supports Windows 10 1903+; the GPUI Windows backend uses post-Win7 interfaces. See `research/compatibility-audit.md` and the captured PE import lists.
- The user explicitly dropped Windows 7 and requested Windows 10 and above. Existing x64 architecture remains in scope; no 32-bit port is requested.
- Windows 10 1903 is the proposed concrete minimum within that request, matching the pinned frontend's documented baseline. It is not a claim that all Windows 10 releases are supported.
- The user approved the final plan and requested implementation, validation and a new installer. The task is now in progress; the repair release is 1.1.1.

## Requirements

- R1: Accept Windows 10 1903+ and Windows 11 in the installer OS gate, while rejecting older OS versions with an accurate message and the existing failure code.
- R2: Preserve the current x64 payload, GPUI/PDFium pins, PDF presentation behavior, speaker-note isolation, current-user installation, CRT preflight, and uninstall semantics.
- R3: Keep current support guidance consistent across README, project plan, packaging documentation and distribution spec.
- R4: Distinguish deterministic installer-policy tests, local native smoke, and actual Windows 10 acceptance; retain NOT_RUN for unavailable systems.

## Acceptance Criteria

- [x] AC1 (R1): A regression exercising the production OS policy rejects Win7/8/8.1 and Win10 build 18361, accepts build 18362 and a later Win10 build, and accepts Win11. The old guard must fail the new Win10 acceptance case in an isolated fixture.
- [x] AC2 (R1, R2): NSIS compiles the actual Setup with warnings as errors, the new version message names Windows 10 1903+ x64, and the independent architecture/CRT/installation checks remain intact.
- [x] AC3 (R2): Rebuilt Setup/portable artifacts pass payload, notice and checksum verification and focused distribution regressions; required `just ci` passes.
- [x] AC4 (R3): Active documentation states Windows 10 1903+ / Windows 11 x64 and prerequisites consistently. Historical archived evidence remains intact.
- [ ] AC5 (R2, R4): Observe app launch and PDF render on the available Windows 11 host using the rebuilt payload. Record Windows 10 installation, launch/render, presentation and uninstall as NOT_RUN if a suitable machine is unavailable, with a concrete qualification checklist.

## Constraints and scope

- The user subsequently authorized `just release` repack, local installation of 1.1.1, a local task commit and archive. Push and public release publication remain outside scope.
- No Windows 7/8/8.1 compatibility work, early Windows 10 backport, 32-bit port, GPUI/PDFium migration, optional media qualification, or dependency downgrade.
- No size/DPI matrix is required for this problem.

## Delivery status

Release 1.1.1 was regenerated with `just release` and installed successfully for the current user. Installed bytes, shortcuts, registration/version and preservation of two settings files were verified. All automated checks pass. AC5 remains partial: native Win11 launch and the earlier isolated PDF-ready title were observed, but Computer Use did not expose the new window for visual rendering acceptance. Windows 10 native qualification is also NOT_RUN. The user requested commit/archive after these limitations were reported; archive records the completed repair/install deliverable with native qualification still open, not as a passed visual/Win10 test. See `research/validation.md` for final artifacts and receipts.
