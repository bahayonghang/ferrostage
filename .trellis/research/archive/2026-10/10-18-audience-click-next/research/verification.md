# Implementation and verification

Historical feature-only checkpoint, before the subsequent delivery request. Current delivery state is recorded in delivery-check-review.md and delivery-plan.md.

Date: 2026-10-18. The user approved default-on behavior and implementation after the planning summary. Product implementation, independent check, planned spec updates and scoped native smoke are finished. Changes remain uncommitted on dev; no push/archive was performed.

## Delivered behavior

- A completed left click on the audience playback client area, including letterbox margins, routes through the existing Next intent and content-step mapping. The final step does not wrap.
- Settings > Displays exposes `单击播放窗口切换下一页` / `Click playback window to advance`. Optional v1 leaf `display.audience_click_to_advance` defaults true, applies to the current window and persists through the established settings store.
- Disabled, non-Presenting, modal, black/white, Curtain, busy, pending and retired-window input is ignored. Audience still receives only projection data; its new payload-free outgoing event is checked by Shell after the window update lease ends.
- Setting-only edits preserve window identity/page/timer/roles; real placement edits still enter the display transaction. Core, vendor, dependencies and the public projection type are unchanged.

## Automated and review evidence

| Check | Result | Evidence |
| --- | --- | --- |
| Settings tests | PASS, 44 tests | Implement agent focused suite and `check-ci.log` |
| Default `just ci` | PASS | `check-ci.log`; fmt, strict clippy, workspace tests, default deny |
| GPUI/PDFium app build | PASS | `check-product.log` |
| Strict product all-target lint | PASS | `check-product.log`, `check-lint-final.log` |
| GPUI real-input suite | PASS, 143 tests | `check-ui-final.log`; 7 new input/settings tests |
| Independent whole-change review | PASS after one test-strengthening fix | `check-review.md` |
| Formatting and tracked/untracked whitespace | PASS | final coordinator readback/check |
| Full-feature offline dependency audit | FAIL, existing advisories | `check-deny-all-features.log`; see limitation below |

The reviewer strengthened presenter isolation to click the actual rendered slide preview rather than window chrome. The implementation's first frame-publication test expectation was corrected to respect the established complete-old-frame contract, with an additional matching-new-raster assertion. No runtime workaround or weakened assertion was used to bypass those boundaries.

## Native observations

Used the computer-use skill's supported `@oai/sky` API against this task's actual FerroStage windows. The observations are the screenshots emitted by the tool calls in this chat, not saved screenshot files or a browser recreation. `native/launch-inputs.json` binds the initial executable, PDFium runtime, PDF and isolated config hashes.

Product executable: `target/debug/ferrostage.exe`, SHA256 `0881EFAF1F00F9F54B8AC13EDBC265A2A451D2BC564DEA04270EF338A97E3060`. Independent review later changed test-only source; no product source/executable changed during native QA. The final executable hash matches the initial/native/check freeze.

Fixture: `fixtures/beamer/overlays.pdf`, actual three-physical-page Beamer with two logical frames. Isolated task config: `research/native/settings.toml`; the initial config deliberately omitted the new leaf, set playback windowed and disabled hot reload. No normal user config or PDF was modified.

| Observation | Result |
| --- | --- |
| Initial missing-leaf config, audience ID 3479540 | First frame, first overlay visible; no setup required |
| One completed audience click | First frame gains Second overlay; no logical-frame skip |
| Click black letterbox margin | Second logical frame appears |
| Click again at final frame | Final frame remains; no wrap or automatic stop |
| Keyboard Left returns to first-frame second overlay | Baseline restored for guard checks |
| Black blank -> click -> unblank | Same second overlay; no hidden advance |
| White blank -> click -> unblank | Same second overlay; no hidden advance |
| Settings open -> audience click | Same second overlay; modal guard verified |
| Displays setting row | Correct Chinese label/helper text, default On and saved feedback visible |
| Turn toggle Off, close settings, click audience | Same second overlay; disk leaf readback false |
| Reopen settings | Off state retained |
| Turn toggle On, close settings, click audience | Advances to second logical frame; disk leaf true |
| Audience identity through both toggles | Same ID 3479540, no window recreation |
| Restart after saving; change only isolated fullscreen flag | Existing saved click=true retained and reloaded |
| Fullscreen audience ID 14615210, one click | First frame advances from first overlay to second overlay |
| Exit fullscreen test process | Exit code 0; run logs empty; no FerroStage process remains |

The host exposes three screens. The native windowed check used the existing selected audience screen; it does not establish the one-monitor exception. No screen role, size, DPI or OS setting was changed. The final isolated config remains saved as click=true/fullscreen=true for reproducibility.

## Requirement / acceptance trace

| Criteria | Evidence |
| --- | --- |
| AC1 | Real input tests for completed/non-left/outside/presenter-preview input; native Beamer step and margin clicks |
| AC2 | Real settings pointer and Space control test, store readback/fresh session, unchanged window/state assertions; native Off/On same window and saved restart |
| AC3 | Actual event tests for modal/blank/Curtain/load/preferences/window transactions, pending/retired/non-Presenting; native modal/black/white checks; reload source evidence below |
| AC4 | Real audience event with overlay/notes map, final-step and timer assertions; native real compiled Beamer sequence |
| AC5 | Parser/store/future/save-failure/Retry tests; native bool disk readback |
| AC6 | Windowed and fullscreen native click behavior executed. Physical first-inactive-click and one-monitor-only native cases are explicitly NOT_RUN |

## Remaining qualification limits

1. Full-feature deny is blocked by existing unmaintained advisories RUSTSEC-2026-0371 (`zbus_names` 4.3.4), RUSTSEC-2026-0370 (`zvariant` 5.15.0), RUSTSEC-2026-0372 (`zvariant_derive` 5.15.0). Cargo.lock/manifests/vendor/policy are unchanged. Bans/licenses/sources pass. No waiver or unrelated dependency migration was added.
2. The tool activates its target before injecting a click. Native observations therefore do not independently prove the OS's first physical click on an inactive window. No such PASS claim is made.
3. One-monitor windowed-exception native input was not exercised on this three-screen host. Existing policy/lifecycle tests remain intact; no destructive/disruptive screen reconfiguration was attempted.
4. Reload input suppression is source-verified through `busy() -> reload_busy() -> reload.pending.is_some()`. Actual input tests exercise other busy owners; no new reload-specific native/input fixture was run.

These are recorded acceptance/qualification gaps, not unimplemented product branches. No performance, exhaustive window-size/DPI, other-platform or physical-display qualification is claimed. Task remains in_progress pending any separately requested follow-through; no commit or archive is implied by this report.
