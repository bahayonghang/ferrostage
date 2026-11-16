# Native baseline

Repository: `D:/Documents/Code/Rust/Exp/ferrostage`, base `26ec851`, clean before task creation. The installed executable is `C:/Users/lyh/AppData/Local/Programs/FerroStage/ferrostage.exe` (still running with the user's defense PDF). No user settings, PDFs, installed executable or screen configuration were changed.

## Current inventory

`cargo run -p ferrostage-platform --example monitor_probe --locked` exited 0 before product edits. Ten consecutive samples contained DISPLAY1, DISPLAY2 and DISPLAY5. Screens: built-in 2560x1600 at (-2560,-169), primary 2560x1440 at (0,0), rotated external 1600x2560 at (2560,-1142). Enumeration took about 5.4-9.5 ms. Initial watcher and cancellation PASS. Physical replug/port/rotation changes NOT_RUN.

A separate-config baseline debug process logged all three MonitorInfo entries, complete GPUI identity mapping, presenter window creation and confirmed placement. Its temporary settings used Chinese and no recent user documents. `baseline-app.log` retains the original startup evidence. The process was launched hidden to avoid disturbing the existing user session; therefore this establishes native startup/mapping, not visual UI acceptance. CloseMainWindow returned false for that hidden process, so only the owned PID was stopped. No clean-exit acceptance is inferred.

## Competing hypotheses tested

Native Python ctypes probes called Windows APIs directly with correct pointer signatures and released all acquired DCs. They never changed system display configuration.

| Probe | GetDC(desktop) | GetDCEx(desktop, CACHE/WINDOW) | Result |
| --- | --- | --- | --- |
| DPI thread contexts -1, -2, -3, -4 | All 3 screens | Not needed | No missing monitor |
| Three owned top-level opaque cover windows covering all screen bounds | All 3 before/during/after | All 3 before/during/after | No discriminating reproduction |
| Owned desktop-child cover; explicit CLIPCHILDREN also sampled | All 3 before/during/after | All 3 before/during/after | No discriminating reproduction |

The temporary cover windows were destroyed in finally blocks. These negative results do not prove clipping can never happen; they do rule out treating it as the demonstrated cause here. No DC acquisition change is justified by this evidence.

## Confirmed source cause

`Shell::new` and `Shell::topology_changed(Unavailable)` put raw PlatformError text into the shared `Shell.error`. `Changed` restores `topology_valid` without clearing that message. The watcher already publishes recovery to the same monitor list after a stable interval. A single transient failure can therefore leave a persistent banner after detection is healthy.

`windows::enumerate` has two different causes behind the same message: device/geometry keys differ, or the two device snapshots differ in any Device field. Identity/name/primary changes can produce the latter with identical connection-key lists. The screenshot has no structured diagnostic fields, so its original Windows event cannot be reconstructed. Distinguish these causes for future diagnosis rather than inventing one.
