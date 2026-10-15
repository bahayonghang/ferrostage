# Task07 native acceptance — in progress

Updated: 2026-10-15T01:05:06.227113+00:00

## Initial startup regression

- Source/executable: `native/initial-freeze.json`;602/602 source hashes read back unchanged (`native/freeze-readback.json`). Exe SHA256 `9452899cda26e8b1820c74c34b84cd531472a9ef431dd1598190e71ea3fff591`.
- Isolated `native/a-light.toml`, explicit existing PDFium DLL; no initial PDF, no user configuration touched.
- First PID30816 exited before a window could be selected. Two Sky enumerations returned no FerroStage windows; initial stdout/stderr preserved. Exit code was not captured, so not inferred.
- Controlled retry PID89132 returned101 within8s. `native/run-a-light-retry.result.json` and stderr record `cannot read ferrostage_ui::shell::Shell while it is already being updated`, entity_map.rs156. Actual GPU discovery reports RTX5090LaptopGPU; no rendered window or GPU quality/performance pass.
- Native N01 FAIL; N02–N13 NOT_RUN. The implementation agent is repairing the reproduced07 startup path. Initial freeze evidence retained; subsequent code requires a fresh build and native evidence.
- User cannot perform physical unplug/port-change this turn. Those Gate B items remain NOT_RUN; single/zero-display hardware also unavailable. Gate B incomplete;08–11 are not activated.
- No commit, push or archive.

## Startup repair checkpoint — 2026-10-15T01:11:38.078125+00:00

The actual GPUI TestPlatform regression reproduced the same panic before the fix (`product-startup-red-retry.log`). The first offline attempt only failed for missing bit-set and is not counted as the red assertion. After replacing render-time controller reads with minimal prepared presenter data,19 harness tests pass (`product-startup-green.log`), including first draw, model-notification refresh and replacement draw. Full test-support deny passes with the same three approved exceptions. Native retest awaits v2 executable; N01 is not yet marked PASS.

## v2 native retest — 2026-10-15T01:14:43.143395+00:00

- Freeze `product-native-freeze-v2.json`, source602/602 matched; executableSHA `b96cb72ea54ddacacd9491090a5f92ef8950f7fdf720758c2d5f2dc6986bb396`, lockSHA `2557e10b37fa5cfe63e78ce19965b8bbbd83438bdd540eed50405fbbc3dbc23e`. DefaultCI/all-features strictlint/nativebuild and headless19 tests passed before runtime.
- PID168776 exited -1073740791 (0xC0000409); diagnostic PID172680 reproduced the same result with RUST_BACKTRACE=full. Launch parents retained WaitForExit, ruling out early host termination for these attempts. Log names `native/v2-a-light.*` and `native/v2-trace.*`.
- Three-screen UUID mapping succeeded and the B presenter WindowId(1v1) was created. No placement-confirmed line or captured rendered window.
- New panic is `cannot update Shell while it is already being updated` (entity_map.rs164). Full trace establishes tick TopologyEvent handling -> retire_pending -> close/remove_window -> synchronous on_window_closed -> weak.update at shell.rs167. First-draw read fix reached window creation, but topology cancellation cleanup still reenters. It is not the3-second timeout branch; no failed-placement conclusion is established.
- Freeze released for a targeted real-GPUI close-callback regression/fix; retain retired-window identity checks and confirmation gate. Add cancellation/confirmation observations to identify the topology transition and actual pending-window state. Root will use normal interactive window startup in the next attempt; prior Hidden console startup is recorded, not assumed equivalent visual evidence.
- N01 remains FAIL; all visual/interaction results remain NOT_RUN. No active FerroStage process/window after failure.

Corrected 2026-10-15T01:17:42.804877+00:00: exact trace line912 is topology-event cancellation, not poll_transaction timeout. GPUI last-window automatic quit also needs an explicit application-owned exit rule so pending/retired window cleanup and zero-screen waiting do not terminate the process. Native placement is still unverified.

### v3 close-callback regression — 2026-10-15T01:20:20.712450+00:00

Actual GPUI callback red tests cover topology cancellation, timeout rollback and replacement cleanup; all3 reproduced entity_map.rs164 in `product-v3-close-red-retry.log`. The initial sandbox/target failure and subsequent tracing identifier compilation failure are retained separately, not counted as behavior assertions. After App::defer and the shared Explicit-quit installation, `product-v3-close-green-2.log` passes22 headless tests, including replacement Live/page/timer continuity, current audience close Stop, and current presenter quit branch. TestPlatform.quit is a no-op; native survival/shutdown remains NOT_RUN. Topology before/after and placement cancellation diagnostics are added without changing confirmation thresholds. Final CI/lint/build andv3 native freeze are pending.

## v3 native lifecycle success and Open failure — 2026-10-15T01:23:48.436732+00:00

- `product-native-freeze-v3.json`:602/602 source readback matches. Exe SHA `fa8a85611c42a1039f172534c3b1200cfc525cb857459ef0d10fa1e3d359d74b`. PID175144 launched with Normal interactive window style and a live WaitForExit parent; session32441.
- The watcher initially sends Changed with the same complete monitor list. The pending B presenter is cancelled, observed at requested/actualdisplay65731, windowed; its deferred closure does not quit. ReplacementWindowId(1v3) is confirmed and commits Idle, audienceNone. Actual B display bounds[0,0,2048,1152], presenter bounds[474.4,208,1100,760]. No physical topology transition is inferred.
- Sky returned one presenter plus its separate console. Only the specifically titled presenter was selected/activated. `native/v3-a-light-idle-visible-0.jpg` and matchingJSON capture the actual light Claude UI, Chinese text and disabledStart/Open-PDF reason. An earlier unactivated capture showed occluding unrelated content; it was excluded and the agent-created files removed, never used as UI evidence.
- Ctrl+O on the focused actual presenter shows `window not found`, no chooser. `native/v3-open-dialog.*` preserves this N02 failure; the filename is an attempted-step label, not a claim that a dialog appeared. Root requested a real GPUI window-event regression.
- Clicking the actual current presenter title-barClose exits PID175144 with0; `native/v3-a-light.exit.json`. Launch exec session completed; subsequent process readback finds no PID175144. Actual explicit-exit behavior passes for this Idle case; worker/poller thread internals and saved-role relaunch are not independently inferred.
- N01 startup/Bplacement/Idle/noAudience PASS onv3. N02 chooser FAIL. N03–N11 andN13 NOT_RUN; N12 only this clean-Idle-close subcase passes. Full GateB remains incomplete. Freeze released for targeted action/owner correction.

## User stopped Computer Use — 2026-10-15T01:29:43.941377+00:00

- V4 source602/602 hashes matched. Freeze `product-native-freeze-v4.json`; exeSHA `0e435312372a914db92e7dcb1509c28ad6c739ab3db660514a205879bcb10615`, sourceSHA `208d6c5ec1069ab5e323cb0112c2191a902509f0f0ff47a534ef2ea87114c052`. Implementation reports26 headless tests,145 default CI tests, all-feature/all-target strictlint and nativebuild passed; independent final07 review has not run.
- Root launched isolated a-light Idle via `native/run-native.ps1`, PID167888, exec session30311. `native/v4-a-light.launch.json` binds config,argv,exe andsource. The process was still waiting when the launch tool returned; its exit is not verified.
- At the first V4 Computer Use selection/activation/capture call, the tool reported physical Escape by the user and explicitly required stopping Computer Use. No V4 screenshot/interaction result is accepted. No further UI calls, forced process cleanup or action retry was performed. The implementation agent was interrupted to preserve the frozen source.
- Resume: run the requested disk/task/process audit; inspect PID167888/exact executable and session30311 if available, along with v4-a-light stdout/stderr/exit files. Do not launch a duplicate or infer the process is closed. Reconfirm V4 hashes, then resume N01/N02 Ctrl+O/button import, remaining available native cases, and final independent07 check. Existing physical unplug/port-change limitation still applies.
- Parent remains incomplete; GateA technicalPASS, GateB incomplete, GateC/D NOT_RUN;08–11 planning and dependency-blocked. No commit/push/archive/add_session.

### Resume 2026-10-15T01:35:17.400830+00:00
- User explicitly resumed Computer Use. Current07 and v4 source602/exe/PDFium hashes match (`native/v4-resume-identity.json`).
- PID query is sandbox-limited (CIM access denied); do not infer exit from empty Get-Process. Sky found existing v4 Presenter61084300, active and interactive; session30311 remains pending. Reused this window, no duplicate launch.
- Ctrl+O now opens actual native chooser (`native/v4-open-dialog.json`); import and remaining native matrix IN_PROGRESS. Existing Idle error at resume retained as observation, not attributed without reproduction.
- Physical unplug/port tests remain NOT_RUN by user's prior response. No commit/archive.

### Native checkpoint 2026-10-15T01:38:49.876409+00:00
- v4-a-light normal titlebar close: exit0 receipt, PID167888/session30311 completed. Chooser opened and Cancel restored Presenter; dialog field automation returned stale/mismatched accessibility and did not enter a path, so chooser import remains UNVERIFIED (not product failure).
- v4-a-cli running PID66348/session54803, same freeze, own long-Chinese-name PDF. CLI import Ready/page1 of9/no audience PASS. Start confirmed actual A display131073/fullscreen; native audience page1, PageDown page2, B full black, Escape removed audience and returned Ready/page2 (original images `native/v4-a-*`, `v4-escape-ready-page2`).
- Curtain transition not directly captured; automated gate and placement log are separate evidence. Timer continuity not visually displayed by07. Remaining matrix IN_PROGRESS; physical cases unchanged NOT_RUN.

### Native checkpoint 2026-10-15T01:44:50.635312+00:00
- Swap PASS for actual PresenterA/AudienceB roles, new HWNDs43124864/51186720, placement logs confirmed; isolated a-light.toml roles read back. Page2 and blank state preserved; B toggled restored page2. Timer UI absent07, not visually claimed.
- Audience Ctrl+O opened chooser at PresenterA (evidence v4-audience-open-presenter-dialog); Presenter Open button simultaneously disabled is a policy inconsistency under independent review. Modal Cancel preserved Presenting. Text injection remains UNVERIFIED: native key d did enter filename, but type_text cleared selection without inserting path; helper focus/index metadata stale.
- Real captured sizes801x655,391x655,322x640: primary action visible, long title truncates, warnings wrap. Exact DESIGN size matrix not yet complete. Tab→Open, ShiftTab→Start visible rings PASS. FAIL: focused Start + Space advances page2→3; Return no action/Ready. Red native evidence v4-space-focused-start-advanced-page3/v4-enter-focused-start-no-action. Independent checker notified, product freeze still held.
- Root normal-close PID66348/session54803; see v4-a-cli.exit.json for returned code. Remaining native matrix and fixes IN_PROGRESS.

### Review-driven recovery correction 2026-10-15T01:49:10.821980+00:00
- Independent read-only F3 verified in recover_presenter: monitor-membership-only early return can keep fullscreen/stale mapped Presenter after Audience topology loss. Existing G7 already requires safe windowed recovery. Implementer authorized targeted real WindowRecord/TopologyEvent red→green within07, reusing replacement confirmation. Startup presenter_start_fullscreen and post-failure recovery must be distinguished; Unavailable/zero topology waits for a valid strict map and never restarts audience. No new OS/vendor APIs. Physical evidence remainsNOT_RUN.
- Idle rawEnglish observation trigger remainsUNKNOWN. Checker found Idle Next/Previous/Black/White can emit raw core errors; local invalid-stage no-op regression added to scope, no core contract redesign.
- Framework generic window-not-found diagnostics also occur after retired/closed windows. Existing successful lifecycle/exit0 is retained, but origin is not uniquely located; do not declare logs fixed or suppress them via vendor edits.

### Input/modal review checkpoint 2026-10-15T01:54:02.765234+00:00
- product-v5-red.log returned exit101 with5 failing realGPUI regressions: focusedOpen Enter/Space, modalClose activation, PresentingOpen guard, activation-stage recheck and Idle actions. This is expected pre-fix evidence, not a passing check. Jump and topology red logs are tracked separately by implementer.
- F4 Root confirmed globalBackspace/Previous preempts Jump raw editing; add modal key priority regression including conflicting numeric global binding.
- F6 Root confirmed Stop/Swap discard modal but coreStop intentionally preserves preview. Authorize07 coordinator cancellation of abandoned Jump preview, then assert subsequentNext modifies committed page; generic04 core semantics unchanged. Escape modal boundary also requires consistency.
- Independent checker remains read-only waiting for newfreeze; only implementer owns product repairs. Native remains closed; no duplicate build or desktop session.


### 2026-10-15T02:06:59.315885+00:00 — v5 native input checkpoint (Root)
- Frozen product v5 exe `3516cca7cf432cf1e4dace74f1a065c19509d10be1c4bb54f93a7b9e0c382fa8`, 602 inputs verified; independent review manifest `04da229f448f7937b85c62ec3459e0ae619e0ad150e85bd27ded61e2fdf762b5`.
- `v5-input` PID107776/session93123 normal title-bar shutdown returned exit0 (`research/native/v5-input.exit.json`). No live process from this run remains.
- Actual native PASS: saved swapped roles relaunch PresenterA/AudienceB; focused Start Space starts without advancing1; audience Ctrl+O refuses at activated Presenter with localized warning/no chooser; Jump23→Backspace2 clears invalid error; preview2 keeps audience1; Return commits audience2; preview3 Escape retains Presenting2; PageDown then3; focused Stop Return ends Ready3. Screenshots/JSON prefix `v5-*` in child research/native.
- New FAIL F7: Settings click→Tab(close-modal)→Return removes focused control; accessibility focus becomes window region0; subsequent Tab does not focus a control or display a ring. Same region0 observed after Enter Stop. `v5-modal-close-tab-focus-lost` evidence. Needs real-GPUI red regression and focus restoration repair after native matrix, then targeted native recheck.
- Remaining native C vertical / dark / system / en-US / conflicting shortcut / error recents / actual config lock Retry in progress next. Physical unplug/ports and actual single/zero screens remain NOT_RUN by user availability. GateB incomplete;08–11 not started.
