# V14 core acceptance — 2026-10-15 08:03Z

Root alone used computer-use. Source/executable identity remains product-native-freeze-v14.json:609 source inputs aggregate9764bfa0d94764c314fd559d8df2b55e056fc8f6ef77882cdde89662fe5d4413; EXE4be903dc34b4bb47163816cc88a67caf8c9f9806772f931b209bf27a9d3dbcd1. Product/independent checks passed separately. Original screenshots, AX, launch/exit receipts, configurations and full logs are under research/native with v14 prefixes.

## Scope and completed observations

The user explicitly deferred further size testing in favor of core functionality. Exact dark800x600 on B125% and dark390x844 on C150% were already captured; dark1440x900 was not completed and is DEFERRED by user priority. Physical input disrupted earlier resizing and one focus attempt. The image named v14-dark-panel-returned-focus-usable shows an unexpected Jump modal and is NOT a successful focus-return receipt. The later clean keyboard sequence after desktop exclusivity demonstrates Tab entry, Shift+Tab to Cancel, Space cancel and usable Tab-to-Open fallback; it does not prove restoration to the original Displays button.

Three owned runs have exited:

| Run | PID | Result |
| --- | --- | --- |
| v14-light-small |171960| Light320x640 scrolled Displays titlebar Close: exit0, clean full logs, fresh post-exit empty inventory, unchanged config. Scoped F12 retest PASS. |
| v14-dark-matrix |126204| Narrow Start/Stop, standalone Audience close/restart, newUI application migration, conflict rejection, Presenting migration and no-play PASS as below. Later DC detection failure and AccessKit close panic: overall FAIL despite exit0. |
| v14-core-relaunch |80928| Same EXE relaunch restored detection, persisted no-play correctly loaded; newUI Apply B/C, Start, actual Presenting owner Close: exit0, clean logs, fresh empty inventory. Scoped PASS, does not erase previous failure. |

The immediate post-click inventories can contain transitional windows. Only files explicitly named post-exit-windows.json, captured after launcher completion, support empty-final claims. No remaining owned launcher session or app window.

## New-interface core receipts

- Dark390 C application/B PDF: Start created actual fullscreen Audience on B. Stop returned Ready/page1 and removed Audience; restarting created a new Audience, and closing that Audience returned the surviving Presenter to Ready.
- New panel selected application B while PDF was B: conflict text appeared and Apply was disabled. Choosing PDF C cleared the conflict; Apply moved Presenter to B with actual placement confirmation, preserving Ready/page and elapsed state. Draft configuration was unchanged before Apply. MonitorId values, not UI numbers, were written.
- At07:55:18Z Start B/C; PageDown selected displayed page2, W set white. At07:55:41Z the new panel opened with Presenting/page1(zero-based)/elapsed22239. Selecting PDF2(A) remained a draft until Apply. At07:56:18Z transaction committed Presenting/page1/elapsed59184 with actual Presenter B and fullscreen Audience A confirmed. Original v14-core-migrated-audience-white screenshot shows white on A; Presenter screenshot retains page2. Reopening at07:56:54Z reports elapsed95500: timer continued across migration.
- Selecting no-play and Apply at07:57:20.979Z committed Ready/page1/elapsed122003, audience=None. Screenshot retains page2; fresh inventory contains Presenter only; Start disabled. Reopening at07:57:47.400Z reports the identical elapsed122003 after26.42s. v14-core-no-play-config.toml contains B presenter and A/C none with full MonitorIds.
- v14-core-relaunch loads those saved no-play roles:3 displays detected, no Audience, Start disabled. Then actual panel selects PDF3 and applies; v14-core-relaunch-saved-config.toml reads B presenter/C audience. Start opens fullscreen C; owner Close removes both windows, exits0, clean stderr/stdout. This run proves no-play persistence/recovery and new target save, not a second relaunch of the new B/C save; V11's unchanged saved-B/C relaunch evidence remains separately reviewed reuse.

## F13 — desktop DC lifetime leak

At07:58:08Z, after about36m34s in the dark process, native topology detection returned GetDC(desktop) Windows error0. The new panel correctly displayed detection failure and disabled Identify, role selectors and Apply instead of using stale topology. Clicking Re-detect at07:58:40Z failed again. Screenshots v14-core-monitor-detection-failure/retry and full stdout preserve this observation. No physical topology action was requested/performed.

Source diagnosis: platform/windows.rs uses HWND::NULL.GetDC(). winsafe0.0.29 ReleaseDCGuard::drop checks hwnd.as_opt() before ReleaseDC, therefore skips release for this valid NULL-owner desktop DC. The watcher repeatedly enumerates every200ms. This is a concrete resource-lifetime defect; the elapsed time is supporting observation, not a measured exact exhaustion threshold. Fresh process80928 detected all3 immediately. Repair must preserve all-monitor geometry and safe first-party code; do not silently change dependency scope or weaken invalid-topology behavior.

## F14 — AccessKit close callback reaches destroyed HWND

Actual titlebar Close of the failed-detection panel at07:59:16Z exited0 and emptied windows, but full stderr contains accesskit_windows0.34.0/src/util.rs380:52 Result::unwrap() on HRESULT0x80070578 Invalid window handle, followed by panic in a function that cannot unwind. This is FAIL. V14's GPUI callback-owner regression did not cover the separately scheduled AccessKit provider; earlier reports explicitly excluded it. Preserve this evidence and investigate ownership/teardown within authorized GPUI scope before any repair claim. Do not disable accessibility or suppress all handle errors.

## Boundaries

V9 matching Identify, V11 real save failure/Retry and future-version read-only, and V13 chooser success/Cancel remain version-scoped independently reviewed reuse, not new V14 runs.07 Audience still displays a placeholder/white/black; real PDF raster is08 and is not claimed here. Gate B INCOMPLETE; C/D NOT_RUN;08–11 gated, P2 unchanged. Physical unplug/reconnect/ports/latency and real single/zero displays remain NOT_RUN. No commits, push, archive or add_session.
