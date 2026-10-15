# V16 core native acceptance — 2026-10-15

Root operated Computer Use after the user's explicit resume and separate F14 adoption approval. Serial implementation and independent review completed before launch. The source remained frozen throughout both runs:648 inputs, aggregate `3e208cf586dcf37ab0420228e8eb3fba8ee07c0b73a8597c318a7f6a219c35b8`; EXE `cf228c03655ddd45d25520bd88dab025c219af9bec9195314ce541dbb1cd89d1`. Input was the original `fixtures/notes-functional/pympress-notes.pdf`, SHA `e939e35f931c184fd93b2e9b95e560db73ba6a6f6dfccbb0963262f0b359dfeb`. No size adjustments or physical topology operations were performed.

## Observed results

| Core check | Result and exact boundary |
| --- | --- |
| Current topology / placement | PASS for this three-screen desktop. Complete MonitorIds, native sizes2560x1440 /2560x1600 /1600x2560, primary/rotation/positions match prior topology. Current GPUI IDs are65729/131073/65733; they are runtime IDs, not persisted identities. Confirmed requested/actual display and fullscreen observations agree. Native source acquisition now uses the repaired F13 desktop DC path. |
| Numbered list / Identify | PASS. Separate original screenshots `v16-identify-presenter-capture`, `v16-identify-on2`, `v16-identify-on3` show1/2/3 on the matching actual screens after UI-controlled application migrations. These are FerroStage numbers, not asserted Windows Settings numbers. |
| Draft / Cancel | PASS. Selected application2 remained draft; Cancel left original window and roles unchanged. Exact bytes before/after Cancel match. Launch file serialization changed byte hash but parsed settings are identical; this is not a failed Cancel. Reopened panel restores application1/PDF3. |
| Apply / conflict | PASS. Application moved1→2→3 only on Apply, preserving Ready/page1 without opening Audience. Same-screen app3/PDF3 showed conflict and disabled Apply; choosing a different PDF screen resolved it. All saved bindings use full MonitorId. |
| Start / physical-page navigation | PASS within07. Application C/PDF B starts actual fullscreen B Audience. Audience-focused PageDown traverses1→2→3→4 and PageUp returns3. These include both physical overlay pages of the five-page notes PDF. Presenter reflects3/5. Actual slide pixels and03A/03B notes UI are NOT_IMPLEMENTED, not validated by page numerals. |
| Presenter-only modal routing | PASS scoped. G from Audience opens Jump only in Presenter; Escape cancels it and remains Presenting/page3. Native log confirms Presenter activation before the modal. |
| Presenting migration | PASS. New panel changes application1/PDF2. Before migration page2(zero-based), elapsed87767ms; commit remains Presenting/page2 at121758ms. Original new Audience screenshot remains white on A. Later panel elapsed165136ms proves timer continues. |
| No-play | PASS. Apply commits Ready/page2 at191785ms, no Audience; screenshot retains3/5 and disables Start. Later panel at10:18:59.164Z still reports191785ms,23.071997s after the commit. Saved settings have PresenterB and no Audience, presenter_only. |
| No-play relaunch | PASS. Same EXE reloads saved no-play settings, detects3 screens and disables Start. Page resets to1 on fresh document load; cross-process page persistence was not requested or claimed. |
| Restore target / Stop | PASS. UI selects PDF3, Apply stays Ready; Start opens actual fullscreen C. Stop closes Audience and returns Ready. |
| Audience close / restart | PASS. Restart then actual Alt+F4 on current Audience returns surviving Presenter to Ready; subsequent Start works. |
| Presenter close | PASS for both bounded runs below, including a scrolled/queried settings panel in Ready and an active Presenting owner. Full logs, completed exit and fresh empty inventory are all required evidence. |

## Actual lifecycle and F14 evidence

| Run | Owned PID | Lifetime / result |
| --- | --- | --- |
| v16-core-main |42672|10:10:36–10:19:07Z; multiple application/Identify/Audience retirements with live UIA observations; titlebar Close in Ready settings. exit0,126 stdout lines, empty stderr, no panic/error/window-retirement diagnostics, fresh post-exit empty inventory. |
| v16-core-relaunch |58568|10:19:19–10:21:18Z; saved no-play, Start/Stop, standalone Audience close/restart, titlebar Close while Presenting. exit0,48 stdout lines, empty stderr, no panic/error/window-retirement diagnostics, fresh post-exit empty inventory. |

Both launches set `RUST_BACKTRACE=full` and preserve complete stdout/stderr plus launch arguments and before/after configs. A launcher console window also appeared in inventories; Root never automated it. Immediate post-close inventories may show transitional windows, so only the separately captured post-exit files establish final emptiness. All Root launcher sessions returned; no FerroStage or build process remains at closeout.

Before launching, Root ran the adopted-source-equivalent hidden-HWND regression:16 actual owned invisible windows destroyed,64 retained-context COM calls returned UIA_E_ELEMENTNOTAVAILABLE normally. The initial invocation failed on mbx cache permissions before tests; the cache-authorized rerun passed. Both receipts remain. This supplements the independently reviewed8 COM positive/negative/empty tests; it does not prove randomized concurrent conversion/destruction, HWND reuse, all provider methods or universal shutdown safety. V14's F14 FAIL remains immutable; V16 provides a bounded repair and scoped native close PASS.

## Observation limitations and retained failures

Two attempts to find Identify overlays as standalone list_windows entries returned none; the tool captures these transient overlays with their parent. The successful separate1/2/3 original captures are the visual evidence. One post-migration input reported `coordinate input geometry is unavailable`; Root discarded the prior state, selected the current returned Presenter, activated/refreshed it, and the one permitted retry succeeded. No product failure or successful input was inferred from that error. Some immediate accessibility labels lagged the visual frame; settled screenshots, subsequent observations, config read-back and native transaction logs are authoritative.

F13 native topology remained available during the bounded8.5-minute main run and relaunch. This is not a36-minute exhaustion/performance test; the retained real64-enumeration GDI red→green regression supplies resource-specific evidence. No screenshot alone substitutes for the logged complete MonitorInfo/actual placement contract.

Task07 still renders placeholders. Actual current/next PDF and private notes rendering/isolation belongs to08 and remains NOT_IMPLEMENTED/NOT_RUN. Physical unplug/reconnect/ports/latency and real single/zero displays remain NOT_RUN; size matrix is user-deferred, and other explicitly incomplete native matrix items retain their prior status. **Gate B INCOMPLETE;03/07 in_progress,08–11 dependency-gated, P2 unchanged.** No new task, commit, push, archive or add_session. All changes remain uncommitted for review.

`native-v16-audit.json` binds observed placement/timer/config/log data and post-run648-input/EXE/DLL/PDF read-back. `native-v16-evidence-manifest.json` seals original images and raw receipts. Neither is a replacement for manual/physical acceptance.
