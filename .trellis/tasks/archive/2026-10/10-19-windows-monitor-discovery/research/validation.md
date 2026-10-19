# Final validation

Status: resolved with caveats. Confirmed error ownership/classification defects are fixed; the original transient Windows trigger cannot be reconstructed from the screenshot. Changes are uncommitted. The running installed application was not replaced.

## Cause and fix

An initial or watcher discovery failure used the shared operation-error string, so successful recovery restored valid topology without removing its banner. Apply preflight, stale draft validation and Start/Swap shortcuts could independently copy derived display errors into the same shared slot. Dedicated typed discovery ownership and live panel validation now remove only the corresponding recovered failure while preserving unrelated errors. All PlatformError variants are localized; unstable device snapshots are distinguished from actual device/geometry mismatch.

No native acquisition, identity, role policy, topology completeness check or settings schema was weakened or changed. Partial/unknown topology still stops audience output. Current three-screen inventory is healthy. Negative coverage/DPI probes do not justify claiming a hardware/clipping cause.

## Checks

| Check | Outcome |
| --- | --- |
| Original stale banner red/green | Assertion failure observed before repair; passes after repair |
| Apply, stale draft, Start/Swap sibling regressions | Failures observed before each repair; final monitor filter 9 passed |
| Platform suite | 31 passed across 3 suites; platform clippy passed |
| Real Windows GDI regression | All 3 monitors; 64 production enumerations, objects 1 -> 1 |
| `just ci` | PASS; 203 tests, 18 suites, 0 failures/ignored; default fmt/clippy/policy passed |
| Final `cargo fmt --all -- --check` | PASS |
| Product clippy, gpui/pdfium, all targets, locked/offline, -D warnings | PASS |
| Complete GPUI test-support suite | 174 passed, 0 failed/ignored, 1.32 seconds |
| All-feature dependency policy, locked/offline | PASS (existing non-fatal warnings) |
| `just release` | PASS; release build, payload, NSIS Setup, ZIP and checksums verified |
| Rebuilt native monitor_probe | 10/10 samples contain all 3 screens; watcher initial snapshot/cancellation PASS |
| New release process with isolated settings | All 3 screens, complete mapping, confirmed presenter transaction, no discovery error/panic |

The first full GPUI run caught a new status row pushing the embedded Apply button out of view (173/174). The fix reuses the existing detection-status slot as a wrapping localized Alert and preserves the healthy count path. Existing visibility assertions were kept; the final full suite passes. The original failing log is retained as `gpui-tests-before-footer-fix.log`.

## Sibling sweep

Startup, watcher delivery, manual detection, Apply preflight, stale draft, Start shortcut, Swap shortcut, Identify and both modal-open paths were traced. Startup/watcher ownership, Apply validation and shortcut duplication were repaired. Manual detection/Identify/open already use fresh state or recheck eligibility and do not duplicate detection failures. Both display hosts reuse the same localized recovery instructions and existing retry action.

## Release artifacts

- Setup: `target/dist/FerroStage-1.0.0-windows-x64-Setup.exe`
- Setup SHA256: `c652643c883293cf37da51e94ebadcc134e20d84590b7de6f4ce6725730a0673`
- Portable: `target/dist/FerroStage-1.0.0-windows-x64-portable.zip`
- Receipt/checksums: `target/dist/release.json`, `target/dist/FerroStage-1.0.0-windows-x64-SHA256SUMS.txt`

The new release executable hash differs from the still-running installed executable. Packaging did not run installation, close PID 97348, touch user PDFs or change Windows settings. A separate hidden smoke process used task-owned settings and was stopped afterward; this is startup/mapping evidence, not visible interaction or clean-shutdown acceptance.

## Recovery steps

FerroStage Settings > Displays > Detect again, select Application screen/PDF playback screen and Apply. If Windows misses a screen, Windows Settings > System > Display > Multiple displays > Detect; select Extend for separate presenter/PDF screens. Return and detect/assign/apply again. Start presentation explicitly after recovery. A watcher-start failure requires restarting FerroStage; a successful synchronous detection cannot replace the watcher.

## Limits

Original OS trigger: not reproduced. Physical replug/port changes and alternate hardware: NOT_RUN. Native settings mouse interaction/screenshot acceptance: NOT_RUN; both hosts/locales and actual keyboard/mouse dispatch were exercised in the GPUI test platform instead. No screen-size matrix, installation, commit or publication is claimed. The active task remains available for review/commit and any subsequent installed-app acceptance.

## Follow-up: installed on explicit user request

The user subsequently requested direct installation of the verified repair. Setup SHA256 matched the recorded trusted artifact; the complete release was revalidated before installation. No installed FerroStage process was running at this point, so no process shutdown was necessary. Silent Setup completed successfully (2.4 seconds).

Installed at `C:/Users/lyh/AppData/Local/Programs/FerroStage`. The installed executable SHA256 is `cb19d4c1c7fb814ddca2230a62a74b646e4a326d75c98e1dd9c9a39225a3ac41`, identical to the repaired release payload. Full installed payload, uninstall registration and Desktop/Start shortcuts passed verification. The user's settings file hash was unchanged across installation. The earlier not-installed statements above describe the original implementation stage and are superseded by this follow-up. Source remains uncommitted; no new native visual/physical acceptance is claimed.
