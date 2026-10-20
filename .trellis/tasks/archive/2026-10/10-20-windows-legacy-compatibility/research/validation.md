# Windows 10+ installer repair validation

## Outcome

Implemented the approved Windows 10 1903+ / Windows 11 x64 installer policy, regenerated release 1.1.1 with `just release`, and installed it locally after subsequent user authorization. Code and automated artifact checks pass. Runtime qualification has explicit caveats below. Local commit and task archive were also requested; no push, public publication or uninstall was requested or performed.

## Root cause and change

- `packaging/windows/installer.nsi:123-128`: replace the original Win11-only guard with Win10-major and build-18362 lower bounds. Unsupported systems retain exit code 10 and receive the concrete minimum-version message. Independent x64 and CRT checks are unchanged.
- `scripts/test-windows-distribution.ps1`: compile and execute the exact production guard extracted from the sole Install section using pinned NSIS 3.13 and real WinVer comparisons. Synthetic NT workstation values exist only in isolated, no-install fixtures.
- Isolated old-guard comparison reproduces Windows 10 build 18362 rejection (exit 10). Corrected guard accepts that boundary and later tested OS versions.
- README.md, README.zh-CN.md, docs/PROJECT_PLAN.md, docs/packaging.md and the distribution spec consistently state the approved floor.
- Workspace release is 1.1.1. Cargo.lock changes only the ten local workspace package versions; third-party dependencies and runtime hashes are unchanged.

## Checks

| Check | Result | Evidence |
| --- | --- | --- |
| `just ci` | PASS, 33.0 s; 235 tests across 20 test executables, zero failures | `just-ci.log` includes fmt, strict default Clippy, tests and deny |
| Native GPUI/PDFium strict Clippy, app/UI all targets, locked/offline | PASS, 22.4 s | `native-clippy.log` |
| `cargo deny --all-features --locked --offline check` | PASS | `all-features-deny.log` |
| `just release` | PASS, 28.0 s; release graph build, NSIS `/WX`, payload and archive verification | `release.log` |
| Distribution `-Action verify` | PASS | `verify.log` |
| Requested subsequent `just release` repack | PASS, 13.7 s, actual release/payload/Setup/ZIP verification | `release-repack.log` |
| Current-user Setup `/S` installation | PASS, 9.2 s; installed payload/shortcuts/registry and settings preservation | `install-receipt.json` |
| Full distribution regression suite, without `-Lifecycle` | PASS | `distribution-tests.log` |
| Full distribution regressions against the final repack, corrected NT fixture | PASS | `distribution-repack-tests.log` |
| Independent full-scope Trellis check | PASS after one test-fixture accuracy correction | `check-review.md` |
| OS-policy fixtures after reviewer correction to NT Product=1 | PASS, all eight compiled/executed cases | Reviewer's focused `-OsPolicyOnly` rerun recorded in `check-review.md`; no production rebuild needed |
| Whitespace and PowerShell parse | PASS | Reviewer checked script AST; root final whitespace check includes new task text |

The initial full distribution regression log predates the test-only Product correction. The reviewer reran all eight OS-policy cases after that correction, and the main session subsequently ran the entire distribution suite against the final requested repack with the corrected fixture. Production binaries were unaffected by this test edit.

## OS policy results

| Fixture | Expected and observed exit |
| --- | --- |
| Original Win11 guard, Win10 build 18362 | 10 (red comparison) |
| New guard, Win7 / Win8 / Win8.1 | 10 each |
| New guard, Win10 build 18361 | 10 |
| New guard, Win10 build 18362 | 0 |
| New guard, Win10 22H2 build 19045 | 0 |
| New guard, Win11 build 22000 (NT major 10) | 0 |

These are executable installer-policy tests on the available host, not old-OS emulation or native OS acceptance. Full distribution checks additionally covered missing notices, corrupt runtime, forged inventory, unrelated payload, corrupt app license, same-version executable mismatch, checksum tampering, native child failure propagation, Cargo target override and cleanup boundaries.

## Native runtime evidence and limits

- Host: Windows 11 x64 build 26300.
- Launch used the freshly rebuilt `target/dist/payload-1.1.1/ferrostage.exe`, explicit isolated task settings, and `fixtures/beamer-timing/overlays-timing.pdf`.
- Actual release EXE SHA256: `159cccd4671184329ec37b82751208dd74fd2add11043238b7a8e6f2b8bca0cc`.
- PDF fixture SHA256: `8e40489ef4fe594585b19e64c20fd22e958aeb94b801e4b5a7ad7016c59dedd9`.
- Initial hidden launch was not usable for visual acceptance; its owned test process was terminated. A visible launch using the approved native verification plan produced a responding window titled `overlays-timing.pdf — 准备就绪 — FerroStage`, as recorded in `native-observation.json`. Its stdout/stderr logs contained no startup errors.
- **Native launch and PDF-ready state: observed. Visual PDF rendering: NOT_RUN.** Computer Use repeatedly omitted the newly launched window while listing the pre-existing installed instance. Fresh inventory, app inventory and a kernel reset did not resolve the capture limitation. No screenshot or UI input was fabricated from a raw handle. The owned smoke process was terminated after this tool limitation; graceful UI shutdown is NOT_RUN.
- At initial delivery the pre-existing installed window was left running. At the subsequent explicit install request no FerroStage process remained, so Setup could upgrade the registered 1.0.0 installation without killing an application. Silent installation exited 0; `Test-DistributionInstallation` verified exact release bytes, shortcuts and registry version 1.1.1. Two existing settings files retained their hashes through installation.
- The installed 1.1.1 executable was launched through Computer Use and a responding `FerroStage` process/window title was observed through OS process metadata (`installed-launch.json`). Computer Use still omitted the new window from its inventory. Installed launch is observed; installed visual rendering remains NOT_RUN. The installed application is left running for the user.
- **Windows 10 actual install, launch/render, presentation and uninstall: NOT_RUN**, because no suitable Windows 10 host was available. Follow the qualification checklist in `docs/packaging.md` before calling this runtime-qualified on Win10.
- Hosted CI, clean-machine qualification, signing and optional media qualification were not run by this task.

## Deliverables

All artifact paths are under Cargo's resolved `target/dist` directory:

| Artifact | Bytes | SHA256 |
| --- | --- | --- |
| `FerroStage-1.1.1-windows-x64-Setup.exe` | 10510536 | `11bd9c8b9d51a93ab2c475de6576cd1e43563dc2769c2430fcb367113419239b` |
| `FerroStage-1.1.1-windows-x64-portable.zip` | 10399679 | `bcc8819be1591f49cc26c3458453db1da52813a8dd3e27bbf4f9d6d657e7ef52` |

The table identifies the final requested repack, which differs from initial packaging bytes because Setup/archive timestamps are regenerated. The matching `FerroStage-1.1.1-windows-x64-SHA256SUMS.txt` and `release.json` were generated and verified; `install-receipt.json` binds installation to these final artifacts. Setup remains unsigned as documented. The user requested local commit and archive after disclosure of native qualification caveats; archiving this repair/install task does not mark visual rendering or Windows 10 native checks as passed.
