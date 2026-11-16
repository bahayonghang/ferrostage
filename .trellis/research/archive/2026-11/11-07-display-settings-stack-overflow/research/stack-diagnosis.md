# Display settings stack diagnosis and regression

## Root cause

The unoptimized Windows render path reserves more than the executable's 1 MiB main-thread stack in four simultaneously active view-builder frames, before any nested button or GPUI work. This is finite stack-frame accumulation from by-value GPUI `Div` temporaries, not evidence of recursion. Other settings categories do not call the shared display body.

Evidence came from the actual debug library/executable produced by `just dev`, not source size estimates. `dumpbin /headers target/debug/ferrostage.exe` reports `100000 size of stack reserve` and `1000 size of stack commit` (hex).

| Function | Baseline reservation | Final reservation |
| --- | ---: | ---: |
| `Presenter::render` | `0x39750` = 235344 | unchanged |
| `Presenter::settings_panel` | `0x718B0` = 465072 | `0x607C0` = 395200 |
| `display_settings::view::panel` | `0x9ED0` = 40656 | unchanged |
| `display_settings::view::body` | `0x4F0E0` = 323808 | `0xB660` = 46688 |
| Extracted `detection` | inside body | `0xFB20` = 64288 |
| Extracted `overview` | inside body | `0x162D0` = 90832 |
| Extracted `monitor_rows` | inside body | `0x1E500` = 124160 |
| Extracted `settings_recent_files` | inside settings_panel | `0x11CA0` = 72864 |

Baseline four-frame total: `0x103FB0` = 1064880 bytes, already 16304 bytes over the 1048576-byte stack reserve. Final render/settings/panel/body plus the largest extracted display section total: 842048 bytes. The separate button and activation frames remain `0x91F0` and `0x81A0`. These sums describe selected compiled frames, not a complete runtime high-water measurement.

The baseline body prologue was:

```text
_RNvNtNtCscgruQd7FC21_13ferrostage_ui16display_settings4view4body:
0000000000000000: push rbp
0000000000000001: mov eax,4F0E0h
0000000000000006: call __chkstk
000000000000000B: sub rsp,rax
```

Each measured large frame follows this `mov eax,<reservation>; call __chkstk; sub rsp,rax` sequence. The final body immediate is `B660h`; the final settings-panel immediate is `607C0h`.

Reproduction commands (PowerShell; use Cargo's resolved target directory):

```powershell
$dumpbin = 'C:/Program Files/Microsoft Visual Studio/18/Enterprise/VC/Tools/MSVC/14.51.36231/bin/Hostx64/x64/dumpbin.exe'
& $dumpbin /headers target/debug/ferrostage.exe | Select-String 'stack'
& $dumpbin /disasm:nobytes target/debug/deps/libferrostage_ui-c030c87031404dc2.rlib |
    Select-String '^_RNv.*(4view(4body|5panel|7actions|9detection|8overview|12monitor_rows)|9Presenter14settings_panel|9Presenter21settings_recent_files|6Render6render)(B\w*)?:$' -Context 0,5
```

The feature-specific artifact hash is local evidence, not a stable project interface. Default `just ci` builds a different non-GPUI library; choosing the newest library irrespective of features would inspect the wrong artifact.

## Fix boundary

Move display detection, topology overview and monitor rows into sequential section builders, retaining all original children, ordering, IDs, roles, styles, commands and eligibility. Move General-only recent-file construction out of `settings_panel`, taking and returning the original `settings-content` element so no layout wrapper appears. This prevents that inactive category's temporaries from consuming the Displays path's stack. No stack-size/linker/dependency/profile changes.

The first display-only decomposition was insufficient: both native Settings > Displays and the constrained regression still overflowed. Temporary test-stage diagnostics showed fixture initialization and the standalone host passed, then the embedded host overflowed. Further extracting only the monitor heading was also insufficient and was removed. The measured remaining 465072-byte settings caller led to the final recent-file extraction. No temporary diagnostics remain.

## Regression and checks

The Windows-only `shell::tests::display_settings_both_hosts_fit_windows_main_thread_stack` reruns the existing GPUI `displays_design_both_hosts_fit_three_rows_and_cancel_only_allocation` regression in a child process with `RUST_MIN_STACK=1048576`. Rust's normal test stack is 2 MiB and had masked the production failure. Isolating the child allows an overflow abort to fail one assertion rather than aborting the entire parent suite. Both successful exit and `1 passed` are required, so an accidentally empty test filter cannot pass the guard. Other platforms retain the existing both-host interaction regression without imposing the Windows stack reserve.

The exercised test creates an actual Presenter window on TestPlatform, renders standalone and embedded display hosts through the real Presenter render path, checks three monitor rows/action geometry, dispatches actual details/role/cancel input and checks draft/persistence behavior. Native OS/GPU observation is recorded separately in `native-reproduction.md`.

- RED, original production source: `cargo test -p ferrostage-ui --features test-support --lib --locked display_settings_both_hosts_fit_windows_main_thread_stack -- --nocapture` failed. Child printed `thread 'shell::tests::displays_design_both_hosts_fit_three_rows_and_cancel_only_allocation' (...) has overflowed its stack`; parent assertion failed. No production edits existed for this red run.
- GREEN, final source, same command: 1 passed, 274 filtered out; 0.92s.
- Full GPUI: `cargo test -p ferrostage-ui --features test-support --lib --locked`: 275 passed; 1.99s. Includes settings categories, actual input, fonts/history, monitor recovery/drafts and the child stack guard.
- Strict GPUI and application lint: `cargo clippy -p ferrostage-ui -p ferrostage-app --features ferrostage-ui/test-support,ferrostage-app/gpui,ferrostage-app/pdfium --all-targets --locked -- -D warnings`: exit 0, no issues.
- `cargo fmt --all` and `git diff --check`: pass.

## Sibling sweep

- Exactly two shared display-panel call sites exist: full Settings in `presenter/settings_view.rs` and standalone in `presenter.rs`. Both use the fixed body and are exercised by the child regression.
- All settings categories share the formerly oversized caller; the General-only extraction preserves its original root and children. Full GPUI tests cover General fonts/history and all categories, so this related change has direct regression coverage.
- Compiled presenter view inventory also contains substantial finite frames: workbench `0x484E0`, page browser `0x3DB30`, home `0x3BB30`, crop `0x26CA0`, tools `0x23300`, setting row `0x1CC30`. They do not call the failing embedded display body and no corresponding failure was established; leave them unchanged. This inventory does not establish a complete stack high-water qualification of every app flow.
- Default CI and final native debug retest are owned by the coordinating session and recorded separately. No window/DPI/hotplug matrix was added.
