# Windows 10+ installer policy

## Decision and boundaries

Use Windows 10 1903 (build 18362) as the minimum for the existing x64 desktop release, following the exact pinned Zed installation documentation. Windows 11 is accepted by the same lower-bound policy. The user approved this concrete floor, implementation, validation and a new installer; the task is in progress.

Change the installer OS predicate and its explanation, not the application architecture. Preserve the current independent RunningX64 check, failure code 10, CRT prerequisite, current-user paths, locking, rollback and uninstall behavior. No dependency/feature changes or additional runtime downloads are needed for this bounded fix.

## Installer mechanism

Use NSIS WinVer's major-version and build lower bounds (`AtLeastWin10` and `AtLeastBuild 18362`) in the existing install preflight. Check compiler macro names against pinned NSIS 3.13. Reject with `FerroStage requires Windows 10 version 1903 (build 18362) or later, x64.` before payload changes. Do not merely use AtLeastWin10, which would accept unsupported early Win10 releases.

Keep the production OS check in one small reusable NSIS include only if that is necessary to execute the identical policy in isolated regression fixtures. Otherwise, exercise the exact guard extracted from the production template and assert fixture extraction succeeds exactly once. Do not maintain a second PowerShell implementation of the version predicate. Fixtures may supply synthetic WinVer version/build state; production must have no test override or OS bypass.

## Test architecture

Compile a no-install NSIS harness around the identical production policy, execute it silently for older Windows, Win10 boundary cases and Win11, and inspect exit codes. The harness writes no shortcuts, application files or registry entries. Keep fixture generation and execution in the existing distribution regression boundary. Run a red comparison using an isolated copy of the original predicate; never revert/stash shared work.

This tests policy semantics only. It does not emulate Windows APIs, graphics drivers, OS loader behavior, CRT installation, or PDF rendering on Win10.

## Documentation and qualification

Update current README, project plan baseline, packaging documentation and backend distribution spec with the exact floor and x64 architecture. Preserve archived task evidence and ADR history. Existing runtime dependencies, notice hashes and licensing rules remain intact.

Rebuild the actual release via the resolved Cargo target directory; verify Setup, portable ZIP and checksums. Observe the rebuilt application opening/rendering a PDF on the available Win11 host, using isolated application settings where possible. Do not run the current-user installer lifecycle locally without a separately authorized install operation. Hosted disposable-runner lifecycle remains available in the existing workflow.

Actual Win10 acceptance requires a Win10 machine/VM: confirm version/build and x64, install required x64 VC runtime, install Setup, launch from shortcut, open/render a PDF, present/flip/end with audience notes isolation, and uninstall while preserving settings/user PDFs. Record unavailable checks as NOT_RUN. Do not claim that Win11 smoke or synthetic version tests prove Win10 runtime support.

## Rollback

The rollback boundary consists of the OS guard, its focused regression and matching current documentation. Existing published binaries are not replaced or uploaded by this task. No GPUI maintenance delta or application data migration is introduced.
