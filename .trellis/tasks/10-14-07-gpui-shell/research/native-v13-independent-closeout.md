# V13 native independent closeout — F12 FAIL

Active task: `.trellis/tasks/10-14-07-gpui-shell`. Read-only evidence review; no native launch, desktop operation, source/specification/configuration edit or build by this reviewer. Root preserved the completed five-run evidence before V14 repair. The final92-file manifest and26 original screenshot hashes were independently verified with no mismatch.

## Findings (fixed)

None in this read-only review. V13's four initial runs have scoped successful close evidence. They do not establish an unconditional shutdown fix: the fifth run reproduces a different retired-window callback failure.

## Findings (not fixed)

**F12 — light320×640 Displays close, FAIL.** `native/v13-light-small.stdout.log:18` records `vendor/gpui/src/window.rs:1943: window not found` at06:30:21.938645Z, after the poll-cancellation record at06:30:21.915720Z. Root identifies the callback as `on_hover_status_change`. PID172512 exited0 and its final window inventory is empty; neither fact overrides the error. Root released V13 only after all five processes exited and assigned V14 regression/fix. This review makes no claim about unfinished V14 source.

Three size/theme combinations remain feasible after repair: dark1440×900, dark800×600 and exact dark390×844. They are validation work, not external blockers. V11 dark389.6×844 at125% cannot stand in for390×844; the current150% displays can represent the requested size exactly. The light320 layout observation can be retained, while its closing behavior remains failed.

N11 also requires individually recorded keyboard/focus, hover/pressed, disabled/error and observable loading states. Ready/Displays size screenshots do not establish every state. Existing version-scoped focus, disabled/read-only, conflict and save-Retry evidence remains usable through V13 where the path is unchanged; any unrecorded control state should remain pending. A narrow Presenting/Stop and new-panel keyboard Cancel/return-focus observation are feasible additions after repair. This does not invent an exhaustive state×size×theme Cartesian requirement.

## Verified identity and completed runs

Before V14 release, this reviewer independently matched all609 frozen source/contract inputs, the executable and unchanged PDF fixture. Root's `native/v13-pre-release-identity.json` separately confirms609 inputs/executable at06:31:45Z after the fifth run. After release, these are historical identity checks; the working source is not re-labelled V13.

- Source aggregate: `77aa180c7f60bcc39de7315554d9781d5f2f39effd211078a1da43cf7ed40d4a`.
- EXE: `d1164919dcdc09053df4fc6caae09be249329cccd0ad8bd482a67347561a6435`.
- Freeze SHA256: `d3689820edf788e4e55d485b644f58007c4616e6ca49262de9aa057d5e86fb91`.
- Original repository PDF SHA256: `25a7932f46ee8fcd4866011921c574b4f5b343af05b793bc397b7e93ed0df013`.

All five launch records bind this source/executable identity and the actual config-before hash. Each launch PID matches its exit receipt and has an empty final native-window inventory.

| Run | PID | Close outcome | Configuration read-back |
| --- | --- | --- | --- |
| v13-main |182160| exit0, no WARN/ERROR, cancellation logged | byte-identical after intentional B/C→C/B→B/C round trip |
| v13-dark |53500| exit0, no WARN/ERROR, cancellation logged | byte-identical after draft and panel close |
| v13-en |184232| exit0, no WARN/ERROR, cancellation logged | CRLF→LF only; parsed settings identical |
| v13-open |183852| exit0, no WARN/ERROR, cancellation logged | byte-identical |
| v13-light-small |172512| **FAIL F12**, despite exit0 and cancellation log | byte-identical |

## Native observations that remain supported

- **C placement:** new Displays controls migrated application to C. Strict confirmation at06:05:39.313097Z records actual1002.6667×760 client on display65735, inside the intended usable bounds. The06:05:46.841Z screenshot shows all titlebar controls and the full right edge; no intervening bounds move appears. This establishes the immediate new-UI C placement observation that V11's moved-before-capture screenshot could not. It is specifically this migration run, not a retroactive V11 startup PASS.
- **Nonowner and owner close:** English run captures actual A audience, audience Alt+F4 returning Presenter to Ready, Start creating the audience again, and Presenter titlebar Close removing both. Complete logs remain clean for that run. Audience capture is the07 page-number placeholder with no controls/notes; it is not08 PDF raster evidence or a complete cursor test.
- **Chooser:** v13-open starts Idle without a PDF argument, records the repository path in the native File name field, then shows raster.pdf/Ready/page1 of9 and restored Open focus. The stale AX Search Box focus label does not contradict the separate File name value/read-back and observed result. The earlier v13-main focus/index attempts are not successful import evidence; their cancellation was recovered. A small successful import does not prove shutdown occurred during an in-flight read.
- **Draft close:** dark320 and light320 preserve exact config bytes. This proves those closes did not persist a draft; only the dark run has clean-close evidence. Historical V9 Cancel/window identity and V11 Apply/Retry/future-schema evidence remain separately version-scoped.
- **Source applicability:** V13 changed Shell async-task ownership and its component contract, retaining the reviewed settings draft, MonitorId roles, Identify numbering, transaction confirmation, no-play/timer and save ownership paths. V9 Identify and V11 new-UI functional observations remain applicable through V13; future V14 changes require their own independent assessment.

## Exact size/theme observations

Each observed cell has Ready plus Displays captures and matching actual-client bounds in its run log. Screenshot dimensions alone were not used to infer logical client size.

| Logical client | Light | Dark |
| --- | --- | --- |
|1440×900| observed, v13-en, B125% | NOT_RUN |
|800×600| observed, v13-main, B125% | NOT_RUN |
|390×844| observed, v13-main, C150% | NOT_RUN; historical389.6 differs |
|320×640| observed, v13-light-small; **close FAIL** | observed, v13-dark, B125% |

Viewed C full-frame, narrow Ready/panel top/bottom, English full-panel, audience and successful chooser captures. Scrollable content exposes application/PDF selectors and Apply/Cancel; this is scoped geometric/interaction evidence, not completion of every design state. Chinese glyphs and English labels are observed in these captures; no blanket font/DPI/accessibility or full DESIGN QA PASS is inferred.

## Remaining boundaries

- Physical unplug/reconnect/port identity/latency and true single/zero-monitor hardware are explicitly excluded this turn: NOT_RUN, with no repeated user request and no use of disabled roles as substitute hardware.
- Current plans record no targetable OS Settings surface for a live theme event and no cross-window drag through the available in-window drag API. Those recorded tool limits remain scoped NOT_RUN; this reviewer did not re-probe the desktop or alter OS settings.
- Transient Curtain/loading frames and complete audience cursor behavior lack native capture proof: UNVERIFIED. Persistent black/white, placeholder output, automated ordering tests and automation cursor overlays do not substitute for these observations.
- Gate B remains INCOMPLETE; C/D NOT_RUN;08–11 remain gated and P2 unchanged. Source/automated PASS and the four clean native runs cannot override F12 or unmet gate items.

## Verification

Frozen V13 automated receipts (UI77, strict lint/typecheck/build, reused default150 and deny) remain the previously verified evidence; no Cargo check was rerun for this read-only audit. Native launch/exit/log/config/screenshot hashes are independently read back. Root's final checkpoint and PRD index retain the failure and leave complete combination checks unchecked, consistent with this review.

- Root native manifest92 SHA256: `e08f7bb83832a0e6ccbd23b0ea5abb6a489181baad8ef85f27bc224475d4aedf`.
- Root completed-runs audit/26 originals SHA256: `274d2edcad1145ea5c9a46c4cf7ebdd8058bc54c4af9593fae7ce9d9c587974b`.
- Root checkpoint SHA256: `eab0096ca3287e04370db6abb8f67c1c47c69de05e69f7b3e7fb7505dc9adbb2`.

`native-v13-independent-audit.json` preserves the separate run, identity, matrix and gap classifications. An initial read-only helper assumed every summary screenshot item had a `path`; selecting the actual screenshot metadata records corrected that audit-only schema error, and all26 hashes then matched. No evidence/product file changed during that retry. This review has no running command and performed no commit, push, archive or add_session.
