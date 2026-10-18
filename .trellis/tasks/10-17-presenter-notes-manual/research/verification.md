# Implementation and verification receipt

## Implemented outcome

- Replaced staged page preview with validated direct JumpTo navigation. Removed thumbnail hover navigation, preview/commit/cancel UI and orphaned core preview state/render priority. Page dialog changes page only on explicit submission.
- Notes-first presenter: current slide left, labeled notes above next slide on the right, default notes share 0.65 with readable effective legacy minimum 0.55. Recent files stay on Idle. Thumbnail grid is on demand only.
- Seven shared semantic button roles, light/dark contrast coverage, explicit selected marks and pointer pressed feedback. Existing warm-neutral/clay design and system fonts retained; no placeholder product content introduced.
- Runtime tracing defaults to warn with RUST_LOG override. Real failures remain visible.
- Hidden thumbnails no longer schedule work. Retained 25 ms polling: BgraFrame equality is deep, so safe notification suppression would require a wider identity/lifecycle change. No CPU/FPS claim.

## Automated evidence

| Check | Result | Receipt |
| --- | --- | --- |
| Focused app/core/settings/UI | PASS: 1/31/40/33 tests | focused-tests-final.log |
| Full default CI | PASS: fmt, strict clippy, workspace tests, default deny | ci.log |
| GPUI/PDFium build and strict lint; GPUI suite | PASS after final click-cancellation fix, 123 tests | check-shell-reviewed.log |
| Real PDFium document/PDF/render suites | PASS, 40 tests | native-tests.log |
| Real perf-100 scheduling/retention | PASS | perf100-native.log |
| Full-feature dependency policy | FAIL on existing unmaintained advisories, bans/licenses/sources pass | deny-all-features.log |
| Whitespace diff check | PASS | git diff --check |

The full-feature policy failures are RUSTSEC-2026-0371 (zbus_names 4.3.4), RUSTSEC-2026-0370 (zvariant 5.15.0), and RUSTSEC-2026-0372 (zvariant_derive 5.15.0). Cargo.toml, Cargo.lock, deny.toml and maintained dependency packages are unchanged. No advisory waiver was added. This task does not claim that the full policy gate passed.

## Real PDFium work ledger

The fixture is fixtures/pdf/perf-100.pdf, verified PDFium is recorded in baseline.md, and target size is 960x540. The old-strip comparison is a request model exercised with the current four-page grid, not a timing run of the old executable.

| Case | Wanted requests | Thumbnails | Workbench retained bytes |
| --- | ---: | ---: | ---: |
| Old four-thumbnail request model | 12 | 4 | 17,510,400 |
| Grid closed | 8 | 0 | 16,588,800 |
| Sixteen-page grid open | 24 | 16 | 20,275,200 |
| Grid closed again | 8 | 0 | 16,588,800 |

Repeated unchanged scheduling submits no new jobs. Removing the modeled old strip reduces the workbench ledger by 921,600 bytes (900 KiB). These are ledger bytes, not total process RSS, unique CPU allocation or GPU VRAM. Native latency, idle CPU and frame-rate measurements are NOT_RUN.

## Native visual evidence and remaining gap

- User-supplied screenshot is the before reference.
- Isolated-config elevated launch confirmed native window creation/placement in logs. User explicitly confirmed the new FerroStage window was visible.
- New-build default stdout/stderr were empty during observed startup. This does not establish a complete open/start/stop/shutdown log cycle.
- Computer Use discovery initially exposed no FerroStage target. Desktop-helper launch then left the helper returning `Computer Use helper already has an active request`; JavaScript reset and reinitialization did not recover it. No PowerShell UI automation fallback was used.
- Real native screenshots, hover/pressed/focus appearance, both-theme size matrix and visual notes readability remain NOT_RUN. Actual GPUI rendered-bounds/input tests are separate passing evidence, not a screenshot substitute.
- Source control roles checked 7 / changed 7 / left 0. Native visual matrix checked 0; no claim of final visual acceptance.

On 2026-10-17 the user accepted the result and explicitly instructed that further visual checks were unnecessary, then approved the exact local commit plan. The native visual matrix remains NOT_RUN, with no further screenshot request required. This acceptance does not turn the full-feature dependency audit into a pass or authorize a policy waiver. The task remains in_progress for the recorded dependency-policy blocker. This receipt is included in the approved work commit; no archive has been performed.

Independent full-scope review: check-review.md reports no unresolved task-local code defect. Its final fixes cover click cancellation, actual Shell ticks across timer expiry, stale documentation and the orphan field found by native lint. Final executable SHA256 is 50C3EB632630493B1325C644FC9C373BB6CC4B297812DCB23E86AF6728CDD0B0. The final test instance was relaunched with the isolated config, see native/accepted-build-launch.json; observed default stdout and stderr are empty.
