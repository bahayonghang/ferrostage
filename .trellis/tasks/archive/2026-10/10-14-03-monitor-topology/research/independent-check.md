# Task03 independent Trellis check

Reviewed 2026-10-14 on main, HEAD ab78a84d96c7cf4a0bbc8dfd4f8e45da180bafb3, with shared uncommitted work preserved. Scope: core MonitorId, platform resolver/native inventory/watcher, integration tests and task03 documentation. Parent coordination and other tasks were not edited. Technical review PASS after the correction below; full task03 acceptance remains pending manual hardware operations.

## Findings (fixed)

- **Files:** crates/ferrostage-platform/src/windows.rs and windows/tests.rs.
- **Issue:** winsafe 0.0.29's bitflag `has(required)` checks any overlapping bit. The adapter combined POSITION, DISPLAYORIENTATION, PELSWIDTH and PELSHEIGHT and incorrectly treated that as an all-fields check. A mode without POSITION could pass and expose its unvalidated coordinates. This contradicted the complete-geometry contract.
- **Fix:** extracted the existing native-mode conversion into a private helper for input-level testing, then replaced the check with full mask equality. No public API, module ownership, dependency or policy changed. A regression accepts a complete mode with extra flags and rejects each required field missing independently. All other adapter `has` calls use single bits and require no change. The backend directory spec records the exact wrapper semantics.
- **Evidence:** independent-regression-before.log exited101 with `accepted native mode without DM(0x0020 32)`; independent-regression-after.log exited0. The earlier sandbox attempt failed at Cargo's build lock before any test ran and is retained separately. Fresh CI, explicit type-check and native probe were then executed against the corrected code.

## Findings (not fixed)

- No unresolved task03 code defect was found after the correction.
- **Required manual acceptance remains NOT_RUN:** physical unplug/replug and <=1s event timing, port-change identity recovery, same-model displays, missing-identity hardware, actual DPI/rotation changes, and GPUI identity/coordinate/window placement. The user cannot operate displays this turn; no hardware changes were attempted. Keep task03 in_progress and the manual acceptance boxes open. Task07 owns actual Unavailable-to-Curtain/Stop/window destruction and recovery without auto-start. These results do not complete parent AC-2/AC-3 or Gate B.
- **Pre-existing, outside task03:** platform/src/lib.rs still contains the unused SHADOW_TMP_SUFFIX and PDF-write comment. The parent baseline already assigns their removal to task10. They were not introduced by task03 and were left intact under the scope/surgical-change rule; no current task03 path writes a PDF.

## Reviewed contracts

- Resolver: first binding per ID remains authoritative, including None; first valid saved role wins; absent saved Audience never triggers a heuristic; saved Audience is excluded from Presenter fallback. Zero/single-monitor rules, primary/ID fallback, area ties, ignored IDs and warnings follow parent design. Any identity collision disables both saved and heuristic Audience, and ambiguous IDs cannot become Presenter. Unit/property tests and actual settings-TOML integration exercise these paths.
- Native boundary: adapter/child enumeration strings are copied before the wrapper advances its reused buffer. Attached non-mirroring connections must match geometry one-to-one, and the second device inventory must agree. Per-monitor errors, duplicate/missing connections, invalid geometry/scale and required-field failures are typed errors; a truly empty complete set is valid. This is a checked multi-query observation, not an atomic OS transaction.
- Identity and scale: device-interface paths are ConnectionScoped; name/resolution is Degraded. No EDID, port-invariance or effective-DPI claim is made. DEVMODE provides physical geometry. The current-context GdiDeviceCapsRatio must be validated in task07's real GPUI process.
- Watcher: 200ms polling plus stable-change coalescing, immediate initial Changed, immediate typed Unavailable on errors, diagnostic-only last_known, and Changed on stable same-topology recovery. Stop/Drop cancel the wait and join; an in-flight synchronous OS query is not interruptible. Pure fake-time tests cover coalescing, reversal, empty topology and error recovery; native evidence only covers the initial event and cancellation.
- Dependency boundaries: core production roots remain thiserror/tracing, settings is only a platform dev-dependency, and winsafe is exact-pinned/Windows-only. No OS/UI imports entered core, no window is created, and unsafe_code=forbid remains unchanged. Existing core-tree.log is still applicable because the relevant manifests are unchanged.

The API review checked the exact cached winsafe implementation and the Microsoft contracts for [EnumDisplayDevicesW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaydevicesw), [EnumDisplayMonitors](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaymonitors), and [EnumDisplaySettingsW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaysettingsw). No product setting or process DPI awareness was changed.

## Verification

| Check | Result | Current evidence |
| --- | --- | --- |
| Formatting / lint | PASS | independent-just-ci.log: fmt and workspace/all-targets clippy with -D warnings |
| TypeCheck | PASS | independent-typecheck.log: cargo check --workspace --all-targets --locked, exit0 |
| Workspace tests | PASS | independent-just-ci.log: 93 passed, zero failed/ignored |
| Gate A contracts | PASS | Same fresh run: core21 + settings38 + platform22 + integration2 = 83 |
| Dependency policy | PASS | Same fresh CI: cargo deny advisories/bans/licenses/sources; existing duplicate-version warnings remain non-failing |
| Native static probe | PASS | independent-native-static-probe.log: ten stable three-screen samples, four-field validation, first Changed, joined stop; exit0 |
| Context / whitespace | PASS | independent-context-validation.log: implement17/check14; independent-diff-check.log: exit0, line-ending notices only |
| Physical hotplug / identity mutation / GPUI | NOT_RUN | stability.md manual protocol; cannot infer these from static or fake-input evidence |

Before edits, all62 code/manifest hashes and all8 inspected dependency-source hashes matched the implementation evidence. Current fingerprints are independent-code-sha256.json (62 files), independent-dependency-source-sha256.json (9, adding the bitflag macro), and independent-native-binary-sha256.json. Only windows.rs and windows/tests.rs differ from the original code fingerprint set. The final hash recheck found zero mismatches across all62 files and confirmed the probe executable hash. Earlier 92-test CI and original probe records are historical, not validation of this correction.

No commands remain running. No commit, push, archive or journal action was performed. Downstream technical use is supported; outstanding manual gates retain their original scope.
