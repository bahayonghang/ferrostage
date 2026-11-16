# Windows 10 installer review

## Scope and result

Reviewed the complete tracked diff against PRD, design, execution plan, distribution/quality specs and pinned NSIS 3.13 WinVer implementation. No product correctness issue or dependency change found. Native Windows 10 qualification remains NOT_RUN; the main session owns the Windows 11 visual smoke and final artifact delivery.

## Findings (fixed)

- File: `scripts/test-windows-distribution.ps1:35-36`
- Issue: The synthetic GetWinVer implementation returned `Product=0`, which NSIS treats as non-NT. Real Windows desktop editions return `VER_NT_WORKSTATION=1`. Current `AtLeastWin10` masks the NT bit and the build predicate reads Build directly, so existing policy outcomes were unaffected, but the fixture represented the wrong OS family.
- Fix: Return Product 1 explicitly, preserving the exact production guard and WinVer comparison macros. Only test code changed; no production installer rebuild is required.

## Findings (not fixed)

- Task metadata: `design.md:5` still states final plan approval is pending, while PRD/implementation notes and the user approval establish execution authorization. Reported to the main session for its task closeout update; no product behavior is affected.
- Windows 10 native install/launch/render/presentation/uninstall remains NOT_RUN because no suitable host was available. This is a qualification limitation, not a failing policy test; the concrete checklist is retained in `docs/packaging.md`.

## Review evidence

- `packaging/windows/installer.nsi:123-128`: rejects below Windows 10 or build 18362, with failure code 10 and the exact minimum-version message, before payload extraction/replacement. The independent architecture guard and VC runtime preflight are preserved.
- `scripts/test-windows-distribution.ps1:19-21`: extracts exactly one production guard; no duplicated production predicate or production test override. The fixture has no install payload, shortcut or registry operations. Success reaches an explicit exit-code-zero suffix; rejection aborts with code 10. The isolated original predicate exactly matches the removed Win11 guard and rejects Win10 build 18362.
- Pinned `cache/nsis-3.13/Include/WinVer.nsh`: AtLeastWin10 compares masked major/minor; AtLeastBuild compares the retrieved build. Product 1 selects the real NT workstation state. Production retrieval remains GetWinVer.
- README files, project baseline, packaging guidance and distribution spec consistently state Windows 10 1903/build 18362+ x64, including Windows 11. No contradictory current baseline found. Cargo.lock changes only ten workspace package versions from 1.1.0 to 1.1.1.

## Verification

- Lint: PASS. Main-session `just ci` fmt/clippy and native GPUI/PDFium clippy logs inspected; no repeated large build was necessary for this test-only correction.
- TypeCheck: PASS via the main-session Rust clippy/release builds; PowerShell AST parse independently passed after the correction.
- Tests: PASS. Reviewer ran `powershell -NoProfile -ExecutionPolicy Bypass -File scripts/test-windows-distribution.ps1 -OsPolicyOnly` after the correction: eight compiled/executed cases, exit 0 overall. Original guard rejects Win10; current guard rejects Win7/8/8.1 and build 18361, accepts build 18362/19045 and Win11. Each fixture compiled with NSIS `/WX`.
- Main-session `just ci` (235 tests), full distribution suite, actual 1.1.1 release and artifact verification: PASS; saved logs inspected. All-features locked/offline dependency policy check: PASS.
- `git diff --check`: PASS after the correction.
- Local installer lifecycle: NOT_RUN as planned; no installation/uninstallation performed by reviewer. Windows 11 visual smoke remains owned by the main session and is not claimed here.
