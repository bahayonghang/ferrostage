# Native V9 checkpoint — 2026-10-15

Root exclusively operated the real Windows application with the computer-use skill. `native/v9-main.launch.json` binds PID51832, source609 aggregate `e0ac2709f9c307ae5f1f7627486c54e985058e9d3c6de3639f87a546b302f8cf`, executable `2b050737e90a9cf5a1966034937ffee0e83f5af5b703396889308c9cd08ffe4e`, the isolated `v8-light.toml` configuration and unchanged repository PDF/PDFium. This is V9, not historical V6. The run began04:32:05Z and actual Alt+F4 ended it04:38:57Z, exit0. Final Sky inventory contains no FerroStage window. Source freeze was released only after observed exit for the defect below.

## Actual observations

- New Displays panel detects3 screens. Stable sorted labels:1=B/primary2560x1440,2=A2560x1600,3=C rotated1600x2560. Actual GPUI scale factors are1.25/1.5/1.5; GDI ratio1.0 is explicitly diagnostic and not DPI proof.
- Draft same-target application2/PDF2 disables Apply and shows the conflict explanation. Cancel returns to Ready1 with identical native window51126550 and config SHA `750d381829ffe9992daacb627d4a84ad624e0f32e971ec7699e06e4c1bf9d5fc`. Next Tab focuses Open. `v9-draft-conflict*`, `v9-cancel-*` preserve screenshots/AX/read-back.
- UI Apply application2/PDF1 really replaces the Presenter on A/display131073. Ready/page1 remain, no audience exists; settings read-back stores MonitorId roles, not numbers. `v9-a-app-ready` plus timestamp04:37:48 logs.
- Identify1,2,3 were visibly captured on B,A,C in separate rounds after moving Presenter through the new panel. `v9-identify-presenter-capture`, `v9-identify-two`, `v9-identify-three`. Each popup contains only the matching number/name. The tool's `list_windows` omits these transient popups; the target window capture includes overlapping transient screenshots. Do not mislabel this as one simultaneous three-screen capture. Popup expiry observed before subsequent actions.
- UI Apply application3/PDF1 reaches C/display65735 but fails the usable-size requirement below. Clean actual close and empty window inventory follow. Complete stdout/stderr have no ERROR/panic/window-not-found/invalid-handle diagnostic in this run; exit0 alone is not the basis for that observation.

## F9 — portrait target clips the application window (FAIL)

At04:38:21 the C replacement reports actual bounds `[1708,-273.33334,1280,912]` on display bounds `[1706.6666,-761.3333,1066.6666,1706.6666]`, scale1.5. `native/v9-c-app-ready-0.jpg` and `v9-identify-three-0.jpg` visibly omit the right title-bar edge. On preceding B→A migration,1100x760 became1320x912. Correct target-display identity is insufficient when the actual window exceeds that display. Root dispatched a minimal regression-backed placement repair to the existing implementer; independent check must follow serially, with a NEW freeze before native retest.

## Remaining

Presenting migration/no-play/timer, new-panel save failure/Retry/relaunch, future-schema readonly and full logical-client size/theme matrix have not run on V9. No old Swap/config evidence substitutes for those. Physical unplug/reconnect/ports, actual single/zero-display hardware and latency remain NOT_RUN under user instruction. Gate B INCOMPLETE;08–11 remain dependency-gated. All changes stay uncommitted.
