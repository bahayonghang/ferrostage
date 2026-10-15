# V14 native checkpoint — closed runs, core defects found

## Superseding update — 2026-10-15 08:03Z

Read native-v14-core-results.md first. All3 owned runs exited; final inventories empty. Core migration/page/white/timer, no-play safe stop/frozen timer, conflict handling, narrow Start/Stop, Audience close/restart and no-play persisted relaunch have scoped PASS evidence. The dark run additionally exposed F13 desktop-DC leakage and F14 AccessKit close panic despite exit0, so overall native acceptance FAIL / Gate B INCOMPLETE. Remaining sizing is explicitly deferred by the user's latest priority; do not resume it ahead of these functional defects. The following07:28Z notes are retained history, not a live-process claim.

Root alone operates Computer Use. Independent V14 review passed with15 related retirement regressions and no source change; see product-independent-v14-review.md and its15-item manifest SHA418c5f3ed6abfd7f89a94b22cce2e22c21f69abbef6941b0991653ea92261b46. Full UI82/current strict checks and unchanged default150 reuse remain separately recorded. Native evidence below is scoped to completed observations, not a complete Gate B PASS.

Freeze: source609 aggregate9764bfa0d94764c314fd559d8df2b55e056fc8f6ef77882cdde89662fe5d4413; EXE4be903dc34b4bb47163816cc88a67caf8c9f9806772f931b209bf27a9d3dbcd1; freeze SHA3f00b403a9668d3a841e0ba15564d28ad804934f8b740c3d4f1803b82da41a1c. Launcher checks each source, EXE, DLL and refuses an existing FerroStage process. The first launch authorization review timed out without executing; the one permitted retry succeeded.

## Completed v14-light-small

- PID171960, launched07:18:16Z, actual titlebar Close exited0 at07:20:50Z.
- B125% client exactly320x640 at[474.4,184.0], confirmed by shell geometry log at07:19:48.487453Z and original Ready screenshot.
- Pointer-only zero-delta scroll over Start produced its visible hover shade while stage remained Ready and focus stayed on the application. This supports hover only, not held/pressed state.
- New Displays showed three numbered screens/current roles, then was scrolled through the app/PDF targets to its bottom; Apply/Cancel remained visible. No draft change or Apply occurred.
- Actual titlebar Close from this scrolled panel is the F12 trigger. Full stdout contains no WARN/ERROR/window-not-found/0x80070578; stderr is empty. The Shell poll-cancel log precedes normal exit. This run passes the scoped F12 retest.
- The first file named v14-light-small-final-windows.json was captured immediately after the Close click, before process exit, and still contains the transitional window. It is preserved; do not use it as an empty-final claim. The later v14-light-small-post-exit-windows.json is the fresh post-exit empty inventory.
- Config before/after bytes are identical, SHA23b646a92a5ec6aed608c9d732dc8a13096dc138a89e9d220d87adde1386a186.

## Live v14-dark-matrix

PID126204 launched07:21:34Z on the same freeze, starting B application/A PDF. Launcher session77408 is still running. No closed-run result or final configuration claim yet.

Exact dark1440x900/800x600/390x844 remain pending. Early native resize attempts were disrupted by physical desktop input: Sky requested re-observation, one foreground-occluded capture showed unrelated content and was not saved, and geometry changed between keyboard steps. These attempts are not product layout failures or accepted size evidence. Root refreshed/activated only the returned FerroStage window and ended Size mode; no unrelated app was operated. At07:27Z Root requested a short period without mouse/keyboard interaction before continuing. Current live client is1024.8x760 on B; this is not one of the required target sizes.

## Remaining scope

Complete the three dark size cells, narrow Presenting/Stop, new-panel keyboard Cancel/usable returned focus, and additional owner/nonowner close cycles with full post-exit logs. Reuse V9/V11/V13 only within the independent V14 applicability table; do not relabel old runs as V14. Physical unplug/reconnect/port/latency and real single/zero displays stay NOT_RUN. Existing live OS-theme/drag tooling and transient Curtain/loading/complete cursor evidence limits stay explicit. Gate B INCOMPLETE;08–11/P2 boundaries and no commit/push/archive/add_session remain unchanged.
