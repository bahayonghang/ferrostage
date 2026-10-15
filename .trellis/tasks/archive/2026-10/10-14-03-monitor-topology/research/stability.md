# Windows monitor evidence and remaining manual protocol

Date: 2026-10-14. Branch main, HEAD ab78a84d96c7cf4a0bbc8dfd4f8e45da180bafb3; current post-review uncommitted source hashes in independent-code-sha256.json and native executable hash in independent-native-binary-sha256.json. Machine inventory is in parent research/native-environment.json (Windows10.0.26300, Core Ultra9 275HX; hardware metadata only). Probe target x86_64-pc-windows-msvc, Rust1.99.0, dev profile.

## Executed static observation

`cargo run -p ferrostage-platform --example monitor_probe --locked` was rerun after the independent DEVMODE required-fields correction and exited0. Ten queries at 200 ms gaps returned identical complete three-screen lists. Initial watcher Changed equaled that list; explicit stop joined. All four required mode fields and per-monitor physical-cap-width / DEVMODE checks passed. Raw fields and diagnostic enumeration durations are in independent-native-static-probe.log; durations are not a formal performance or unplug-latency benchmark. The previous native-static-probe.log and its fingerprints remain historical pre-correction evidence.

| Connection | Physical native size | Physical position | GdiDeviceCapsRatio | Rotation | Primary |
| --- | --- | --- | --- | --- | --- |
| DISPLAY2 | 2560×1440 | (0,0) | 1.25 | 0° | yes |
| DISPLAY1 | 2560×1600 | (-2560,-169) | 1.4997071 | 0° | no |
| DISPLAY5 | 1600×2560 | (2560,-1142) | 1.4995314 | 270° | no |

All three IDs use actual device-interface paths, confidence ConnectionScoped. All names were OS-reported Generic PnP Monitor. No EDID is claimed. Default pure resolution chose primary DISPLAY2 Presenter, DISPLAY1 Audience (equal-area tie broken by identity sort against DISPLAY5), remaining DISPLAY5 ignored. This is a static no-saved-record observation, not a UI/window test. Current process DPI awareness is unmeasured; the ratio is current-context device-capability scaling, not an effective-DPI claim. Existing portrait and mixed ratios are not rotation/DPI-change tests.

## NOT_RUN manual operations

The user explicitly cannot perform physical operations in this turn. No cable, display configuration or power state was changed by the agent. Keep these acceptance boxes open.

1. Static before/after: run `cargo run -p ferrostage-platform --example monitor_probe --locked` and save the output for each state. Physically change a connection, rerun, compare device-interface IDs, confidence, connection_name, geometry and primary. Changed IDs require explicit rebinding; never infer matching from enumeration order or friendly name.
2. Event observation: run `cargo run -p ferrostage-platform --example monitor_probe --locked -- --watch`. After its static prelude and “Observing for 30 seconds” message, unplug/replug the intended display. Capture physical action times independently (e.g. one video showing the operation and probe timer); compare against `watch_elapsed_ms` event records. Require actual Changed/Unavailable within ≤1s, complete identity sets and safe recovery. The harness never reports timing PASS automatically and creates no audience window. Repeat to cover add/remove, rapid changes and OS-query errors where reproducible.
3. Repeat on same-model monitors, a missing-device-path/identity device if available, changed DPI and rotation, and a port swap. Record actual observed source/confidence. Missing-path/collision fixtures already pass automated tests but are not hardware evidence.
4. In 07's actual process, verify ID→GPUI DisplayId uniqueness and physical/logical mapping under its real DPI awareness. Configuration with an absent saved Audience must produce no replacement, and collision must refuse Start. Then Gate B checks actual focus, window destruction ≤1s, presenter windowed fallback and no auto-start on reconnect. The task03 probe cannot verify window behavior.

Automated role safety can be rerun with `cargo test -p ferrostage-platform --test settings_topology` and the topology unit suite. These are fixture-based contract tests and do not replace steps1–4. Task03 remaining manual identity/hotplug acceptance and parent AC-2/AC-3/Gate B remain pending.
