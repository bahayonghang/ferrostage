# Final validation

## Outcome

The real final `just install` completed with exit 0 and zero manual installation clicks. FerroStage 1.0.0 is installed under `C:\Users\lyh\AppData\Local\Programs\FerroStage`. The installed app was launched through the desktop `.lnk`, then its native Open dialog loaded `fixtures/pdf/perf-100.pdf` and displayed real current/next PDF rasters in Ready state. No presentation was started. The window remains open for the user.

`final-installation.json` contains final artifact paths/hashes and installed metadata. NSIS Setup binaries and ZIP timestamps may change on each packaging invocation; use that final receipt rather than intermediate worker hashes.

## Gates

| Check | Result and evidence |
| --- | --- |
| Legacy removal | PASS; `../legacy-uninstall.json`, Cargo executable/DLL removed before implementation |
| `just ci` | PASS exit 0; fmt, strict default Clippy, workspace tests and cargo deny; only existing informational deny warnings |
| `just check-shell` | PASS exit 0; actual native build, strict native Clippy and 166 headless UI tests |
| Release pipeline | PASS; actual GUI dependency graph, pinned PDFium and all notices, NSIS `/WX`, ZIP extraction/inventory/checksums/receipt validation |
| Final `just install` | PASS exit 0; recipe explicitly passes `-Silent`; metadata 0.3 s, Cargo build 1.70 s (stage 2.2 s), NSIS stage 2.5 s, install stage 2.1 s; no manual clicks |
| Installed versus release | PASS; exact trusted inventory, owned file hashes, registry and both shortcut targets/icons/working directories |
| Focused negative checks | PASS; missing notices, corrupt DLL, inventory hash override attempt, stale extra payload, corrupt app license, tampered checksums, child exit propagation, target metadata override, unsafe cleanup bounds |
| Native lifecycle | PASS; fresh install, same-version reinstall, forward synthetic upgrade, downgrade refusal exit 5, locked exe/shortcut install and uninstall refusal exit 4, native uninstall and final restore |
| Partial-install failures | PASS; isolated NSIS source-copy fixture injects failure after replacement; fresh failure leaves no successful registration/owned payload, upgrade failure restores previous inventory (exit 8). Production installer has no test hook |
| Uninstall ownership | PASS; removes owned files/registration/shortcuts, preserves original settings hashes and settings/PDF/unknown-file canaries. Recovery uninstaller survives earlier failure and is deleted only after checked deregistration |
| Desktop native smoke | PASS via computer-use; screenshot shows installed welcome window, native Open dialog, then `perf-100.pdf` current/next rasters. No terminal window is required |
| PDF read-only invariant | PASS; fixture SHA256 remained `395D8BB26AD5F8F7CA5245901ABDCB8DE1FCCD2BE5FF7E30482E54F776526CB6` |
| Shell application discovery | PASS; `Shell.Application` AppsFolder contains FerroStage pointing at the installed executable, and Start menu `.lnk` exists |
| Setup UI | PASS observation; actual MUI Welcome screenshot captured during initial quoting regression. Default `just install` now suppresses this UI as required by user refinement |
| Parser/whitespace/manifests | PASS; independent PowerShell parsing, tracked/untracked whitespace checks and Trellis manifest validation |
| Independent Trellis review | PASS; reviewer independently reran focused checks/release verification and installed registration/file checks; no remaining code blockers |

No measured build-speed improvement is claimed: the structural change avoids the unrelated workspace prebuild and reuses Cargo caching. The final timing above is a warm-build observation, not a before/after benchmark.

## Review corrections

- Bare NSIS `/S` is essential; quoting every argument as `"/S"` opened the wizard. The process wrapper now quotes whitespace paths and preserves raw NSIS switches/final `_?=` semantics.
- Checked shortcut/key removal and lock preflight prevent false uninstall success.
- Installation verification compares against the trusted release inventory rather than accepting internally self-consistent stale installed bytes.
- Keep the recovery uninstaller until checked deregistration; check its final deletion separately.
- Schema 1 owns a fixed file list. Future filename retirement requires explicit bounded migration and regression, not deletion authority from arbitrary mutable inventory paths.

The reviewer's first focused-test attempt overlapped the final release rebuild removing its old receipt. It exited before mutation, then passed after the receipt became stable; this was coordinated verification timing, not a product failure.

## Qualification limits

- Windows Search UI: NOT_RUN. Ctrl+Escape visibly opens Start, but the native helper exposes no targetable Start/Search window and subsequent input reactivates Explorer. AppsFolder discovery and the actual Start shortcut were verified. No unsupported UI Automation or Windows-key workaround was used.
- Hosted workflow: NOT_RUN locally; `.github/workflows/windows-distribution.yml` is configured to build, verify, run lifecycle on a disposable Windows runner, and upload artifacts. No public release/tag was created.
- Clean-machine qualification and signing: NOT_RUN/deferred. Setup is unsigned. The executable imports `VCRUNTIME140.dll`; Setup checks the VC++ x64 prerequisite and Windows x64/11, and docs link Microsoft's redistributable. No developer-system DLL was silently redistributed.
- One interrupted initial test left `C:\Users\lyh\AppData\Roaming\ferrostage\distribution-test-b14c973dab6b42a9b07be7ccbe19cde7.txt`, contents `preserve settings canary`, SHA256 `7D2B945112200B94C9ED075720FC954441278BBC5183CA3057B0EFF479D3AAC2`. Automatic approval rejected its exact-path deletion with only `blocked by policy`; no bypass/retry was attempted by the main agent. This marker is not application configuration and does not affect the app.

## Workflow state

Implementation, scoped review, local lifecycle validation and final local installation are done. Windows Search UI remains explicitly unverified. The user approved the exact Phase 3.4 commit plan and requested archival plus a session journal record. Commit/archive/journal execution follows this validation snapshot; no push is authorized.
