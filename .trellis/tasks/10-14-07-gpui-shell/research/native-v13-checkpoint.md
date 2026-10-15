# V13 native checkpoint — five completed runs, F12 FAIL

Root exclusively operated the native desktop through computer-use. All five owned processes actually exited before source release at06:31:45Z. No physical topology operation was attempted. V13 is historical frozen evidence after release for V14; it is not necessarily the current executable.

## Identity and product checks

- Frozen609-input aggregate `77aa180c7f60bcc39de7315554d9781d5f2f39effd211078a1da43cf7ed40d4a`.
- EXE `d1164919dcdc09053df4fc6caae09be249329cccd0ad8bd482a67347561a6435`.
- Freeze `product-native-freeze-v13.json`, SHA256 `d3689820edf788e4e55d485b644f58007c4616e6ca49262de9aa057d5e86fb91`.
- Independent product review: UI77 (54 actual GPUI +23 pure), strict feature lint/typecheck/fmt/native build PASS; unchanged default150/deny explicitly reused. V12 owns the recurring poller; V13 additionally owns import-read and chooser foreground waits. See `product-independent-v12-review.md`.
- Every launcher verified source/EXE/DLL before launch. Root's final `native/v13-pre-release-identity.json` again verifies609 inputs and EXE before release. Independent reviewer also verified them before V14 work.

## Completed native runs

| Run | PID | Actual closure | Result |
| --- | --- | --- | --- |
| v13-main | 182160 |06:11:40Z titlebar Close from light800x600 Displays | exit0, empty windows, clean stdout/stderr: scoped PASS |
| v13-dark | 53500 |06:14:02Z titlebar Close from dark320x640 Displays with unapplied PDF3 draft | exit0, empty windows, clean logs: scoped PASS |
| v13-en | 184232 |06:23:25Z titlebar Close while Presenting, after standalone audience close and restart | both windows gone, exit0, clean logs: scoped PASS |
| v13-open | 183852 |06:26:41Z titlebar Close from Ready after actual native chooser import | exit0, empty windows, clean logs: scoped PASS |
| v13-light-small | 172512 |06:30:22Z titlebar Close from light320x640 scrolled Displays | exit0/empty windows, but ERROR: F12 FAIL |

All five have launch identity, stdout/stderr, exit, before/after config and final Sky window inventory. `native/v13-completed-runs-audit.json` verifies PID continuity, identities, config parsing and26 original screenshot hashes. First four logs contain no WARN/ERROR; the fifth contains exactly the recorded error. Every run logs poll cancellation before quit.

## New UI and geometry observations

- Through actual Displays controls, main changed B application/C PDF to C application/B PDF. Intermediate same-target conflict disabled Apply, then selecting B PDF enabled it. Apply created the C Presenter and retired B. No Swap shortcut or hand-edited profile substitutes for this transaction.
- At06:05:39Z C client converged1202.6666x912 →1000x754 →1002.6667x760 using the bounded correction, then strict placement confirmed at `[1740,-297.33334,1002.6667,760]`, display65735, usable `[1706.6666,-761.3333,1066.6666,1658.6666]`. `v13-c-initial-placement` at06:05:46Z shows the complete titlebar including Close before any manual movement. This resolves the missing initial full-frame observation from V11. Later deliberate resize operations are separately logged.
- C150% exact light390x844: Ready, top list and scrolled full target controls captured. Then actual newUI Apply returned to B application/C PDF. Final config read-back has those MonitorIds and matches launch bytes; intermediate C/B config was not separately copied and is not claimed as an independent read-back.
- B125% exact light800x600: Ready, top list and bottom choices/Apply/Cancel captured. Dark320x640: Ready/top/bottom/draft captured. English light1440x900: Ready and complete Displays panel captured. Later light320x640 also has Ready/top/bottom screenshots; these geometry observations do not turn that run's failed close into PASS.
- Panel content scrolls internally, labels wrap, target controls remain reachable and Apply/Cancel stay fixed in captured narrow cases. Screenshot outer dimensions are not client-size evidence; the corresponding `window bounds changed` logs establish exact logical clients.
- English labels include Displays, detected count, Identify displays, Application window display, PDF playback display, Do not play, Apply and Cancel. Generic monitor names are accompanied by resolution/current role; connection-scoped identity and GDI-ratio limitations are explicit. GDI1.0 is not the measured desktop scaling.

| Logical client | Light | Dark |
| --- | --- | --- |
|1440x900 | V13 en captured | NOT_RUN, feasible after repair |
|800x600 | V13 main captured | NOT_RUN, feasible after repair |
|390x844 | V13 main on C150% captured | NOT_RUN, feasible after repair; V11 389.6 is not exact390 |
|320x640 | V13 light-small captured, close FAIL | V13 dark captured, close scoped PASS |

This table establishes observed size/theme combinations only, not the complete keyboard/hover/pressed/disabled/error/loading/font/reduced-motion matrix. Remaining feasible state observations must continue after repair, separately from external limitations.

## Lifecycle and import observations

- English B/A Start created the actual A fullscreen audience (display131073). Audience Alt+F4 returned the existing Presenter to Ready; Start then created a fresh Audience. Closing Presenter during Presenting removed both and exited cleanly. The poller therefore still supports normal operation after a non-Presenter close.
- Main opened the actual native chooser and cancelled with Escape without changing Ready1/9. A subsequent chooser attempt exposed stale AX focus/index targeting; a screenshot-coordinate Cancel closed it. These tool errors were not marked product success or failure.
- Open run started Idle with disabled Start. Root clicked the observed native File name editing surface, typed the repository raster.pdf path and verified its actual File name value despite the tool continuing to report Search Box focus. Clicking the observed Open button successfully imported to Ready1/9. `v13-open-chooser-input.json` retains only relevant control read-back, not the incidental Documents listing. Ready screenshot and config/fixture read-back support success. This supersedes the temporary tool-blocked assessment, without deleting that history.
- Repository fixture SHA256 remains `25a7932f46ee8fcd4866011921c574b4f5b343af05b793bc397b7e93ed0df013`. No user PDF was written. This small fixture does not prove native quit at the precise instant of an in-flight read; controlled regression coverage and ordinary chooser completion are distinct.
- Main/dark/open/light-small before/after configs are byte-identical. English differs only in CRLF→LF serialization; parsed settings are identical, so it is not unchanged-byte Cancel proof. Main intentionally changed roles and restored B/C before exit. Dark's unapplied PDF3 draft leaves its B/A persisted roles unchanged.

## F12 — late hover callback after retirement

`native/v13-light-small.stdout.log:18` records at06:30:21.938645Z:

`ERROR : vendor\gpui\src\window.rs:1943: window not found`

The source is `platform_window.on_hover_status_change` calling `handle.update(...).log_err()`. The preceding line records owned poll cancellation at06:30:21.915720Z. Exit0 and no residual windows do not satisfy clean-retirement acceptance. V13 overall close acceptance remains FAIL, despite the first four clean runs and no repeated `0x80070578` in these five runs. F11's original V11 HRESULT evidence remains immutable; no universal elimination claim follows from a bounded run count.

After confirming every owned process exited and the exact V13 source matched, Root released the freeze for serial implement→check V14. The implementer must reproduce the late platform callback lifecycle, audit related callbacks at the same boundary and repair ownership rather than suppress logging. No sixth maintained package, dependency waiver or threshold change is authorized.

## Evidence reuse and remaining boundaries

V9 matching Identify1/2/3 and V11 newUI Presenting migration/page/white/running timer, no-play/page/frozen timer, real save-lock failure/Retry, future-version readonly and saved B/C relaunch remain version-scoped observations. The V13 independent source review explicitly checked their applicability. They are not relabelled as V13 runs, and old configuration/Swap evidence does not replace new-interface acceptance.

Physical unplug/reconnect/port/identity stability/latency and actual single/zero monitors remain NOT_RUN under the user's instruction. Live OS theme has no usable Settings surface in the recorded tool discovery; cross-window PDF drag is outside the supported within-window drag coordinates. Transient loading/Curtain and complete audience cursor behavior lack native proof and remain UNVERIFIED, not inferred from steady black/white screenshots or headless tests. The three remaining dark size combinations and feasible focus/control-state checks are pending work, not external blockers.

Gate B INCOMPLETE, Gate C/D NOT_RUN,08–11 gated and P2 unchanged. All new changes remain uncommitted; no push/archive/add_session. Root will continue after V14 implementation, independent check and new build identity.
