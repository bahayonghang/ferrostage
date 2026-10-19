# Research: Windows monitor discovery clipping and recovery ownership

- Query: Diagnose `IncompleteTopology` in installed FerroStage, preserving monitor completeness and GDI release; evaluate nested WinSafe enumeration, desktop clipping, DPI, and recovered error state.
- Scope: mixed (current source, archived native evidence, pinned dependency source, Microsoft API contracts)
- Date: 2026-10-18

## Findings

### 1. Desktop DC clipping is contractually possible but not reproduced

`crates/ferrostage-platform/src/windows.rs:123` acquires `HWND::GetDesktopWindow().GetDC()` and passes that non-null DC into `EnumDisplayMonitors` at line 128. The Microsoft contract says a non-null DC enumerates only monitors intersecting the DC's visible region. `None` for the clipping rectangle does not remove that DC's own visible-region clipping. Thus this code does not have a general all-monitor guarantee when the desktop is occluded by ordinary top-level windows. If all desktop regions are excluded, geometry can be empty while the independent device query contains active monitors, causing `combine` to return `IncompleteTopology` at line 206.

Root reported a raw native comparison of the existing desktop GetDC and candidate GetDCEx(CACHE|WINDOW): both returned all three monitors at baseline, with full-screen top-level STATIC coverage, and with owned desktop child STATIC coverage. Explicit CLIPCHILDREN also retained three monitors in that experiment. Raw DPI contexts -1, -2, -3 and -4 each returned three monitors. The current actual GPUI run also logged complete three-monitor topology and valid mapping. Therefore the ordinary-occlusion hypothesis is NOT established for this user and does not justify an enumeration change. This researcher ran no native reproduction. Historical `.trellis/tasks/archive/2026-10/10-17-presenter-notes-manual/research/baseline.md:8` recorded devices present and geometry empty during a sandboxed launch; it did not establish the cause.

### 2. Do not revert to the old NULL-owner acquisition

Archived `.trellis/tasks/archive/2026-10/10-14-07-gpui-shell/research/product-v15-getdc-diagnosis.md` records a genuine earlier leak: production `HWND::NULL.GetDC()` leaked one GDI object per enumeration, 2 to 66 over 64 queries. The real HWND owner fixed it, with 1 to 1 after 64 queries on that three-monitor desktop. The source of the leak still exists in pinned winsafe 0.0.29: `src/user/guards.rs:218` checks `self.hwnd.as_opt()` before invoking `ReleaseDC`, skipping valid NULL screen owners. A guard name alone does not establish release.

That prior fix proved full MonitorInfo equality on its actual three-monitor test desktop. Its independent review explicitly noted that the duplicated post-fix candidate ceased being an independent all-screen oracle and that real application DPI/coverage verification remained needed. It did not prove occlusion independence on every desktop.

### 3. Existing-API candidate was investigated but is not justified as a product fix

Pinned winsafe 0.0.29 provides `HWND::GetDCEx(&HRGN, co::DCX) -> SysResult<ReleaseDCGuard>` at `src/user/handles/hwnd.rs:504`. Candidate:

```rust
HWND::GetDesktopWindow().GetDCEx(
    &winsafe::HRGN::NULL,
    co::DCX::CACHE | co::DCX::WINDOW,
)
```

This keeps a real non-null owner so its RAII guard releases on normal and early-return exits, while requesting the window rectangle and omitting child/sibling clipping flags. `WINDOW` and `CACHE` are available at `src/user/co/consts.rs:473-474`; no wrapper/dependency/product unsafe code is needed. Callback DCs remain scoped to enumeration, and all existing device/geometry checks stay intact. The reported old-versus-candidate probes do not distinguish behavior, so preserve current acquisition unless a deterministic failing case demonstrates a benefit. Native equality and stable GDI counts alone prove safety on that desktop, not that this API change fixes the screenshot.

`CreateDC`/`CreateIC` are not wrapped in this cached winsafe version (only CreateCompatibleDC is). CreateCompatibleDC yields a memory DC and is not an all-monitor replacement. `GetWindowDC(NULL)` documentation distinguishes primary-screen behavior, so it is not a clearly stronger alternative. A NULL-owner GetDCEx would retain the same guard-release bug.

### 4. Nested WinSafe iterators are not currently an established cause

Pinned `src/user/iterators.rs:6-11` stores one DISPLAY_DEVICE and one index per iterator instance. Current `devices()` copies the adapter connection and primary flag before creating the nested monitor iterator (`windows.rs:58-59`), and copies monitor ID/name strings before advancing (`windows.rs:73`). No returned borrowed DISPLAY_DEVICE escapes. Nested iterators do not share this buffer.

WinSafe consults LastError on end-of-list (`src/user/iterators.rs:28-35`). Current source already clears LastError before adapter and monitor enumeration and after nested enumeration (`windows.rs:47,61,91`), addressing stale nested errors. Do not redesign this path without contradictory native evidence.

### 5. DPI is a verification dimension, not a proven root cause

`EnumDisplaySettings(CURRENT)` always returns physical pixels independent of DPI virtualization (Microsoft contract). The GDI caps ratio is intentionally process-dependent and called `ScaleSource::GdiDeviceCapsRatio` (`monitor.rs:24-29`); ratio 1 is not evidence that Windows scaling is 100 percent. Search found no product call to SetProcessDpiAwareness/SetThreadDpiAwarenessContext. A PerMonitorV2 manifest exists in vendor GPUI but workspace disables its default features, so manifest presence alone does not establish that the final executable embeds it. Root should measure the actual executable/thread context and avoid assuming the CLI probe equals the application context.

No process or thread DPI settings should be changed as an incidental fix for an unproven DPI cause. Preserve native size/position, scale provenance, identity and full set equality in the corrected probe.

### 6. The confirmed root of a persistent recovered banner is UI error ownership

`Shell::new` stores an initial platform error using raw Display text (`crates/ferrostage-ui/src/shell.rs:253-256`). Watcher Unavailable also assigns `self.error = Some(error.to_string())` at line 1905. Successful Changed only writes monitors and topology_valid at lines 1899-1902; it never clears that error. The platform watcher correctly emits recovered Changed and clears its own error (`watch.rs:167-171`), so the stale message belongs to UI state. Its existing `error_is_immediate_diagnostic_only_and_same_topology_recovery_is_published` regression (`watch/tests.rs:32`) already proves failure followed by successful identical topology produces a recovery event. A UI regression should use IncompleteTopology followed by Changed(original monitors); before repair this leaves valid=true plus the stale English banner.

Do not simply clear all `self.error` on Changed: the same field holds unrelated import/render/settings/action errors (for example shell.rs:644-645, 1566-1567). Introduce scoped typed monitor-error ownership or another explicit ownership mechanism, localize using the current locale at presentation, and clear only that monitor error on successful recovery. Preserve a separate unrelated error through failed/recovered topology events. Test startup error and watcher error separately; test a recovery to unchanged monitors as well, because it must bypass the existing early return while topology_valid is false (shell.rs:1883).


### 7. A legitimate transient snapshot mismatch can produce exactly this banner

`windows.rs:29-39` reads devices, geometry, then devices again. If the first and last owned device maps differ, it returns IncompleteTopology. This comparison includes the path, friendly/driver name and primary flag as well as the connection key; its error payload only reports the map keys. Therefore even matching displayed connection lists need not mean the underlying snapshots matched. A real connection/driver transition can correctly cause a temporary failure. After two stable successful watcher samples (roughly 200 ms apart, excluding OS query time), recovery is published.

This is a reliable possible explanation for the screenshot, not proof of its exact original Windows trigger. The screenshot contains only generic Display text and no structured map/phase evidence. Fix the confirmed sticky UI state and localized guidance, preserve immediate unavailable projection policy, and retain structured diagnostics for any recurrence. No new retry suppression or stale-topology acceptance is warranted.
## Files Found

- `crates/ferrostage-platform/src/windows.rs`: two device snapshots, geometry DC acquisition, complete-set validation and typed failure.
- `crates/ferrostage-platform/src/windows/tests.rs`: prior native GDI leak regression; currently duplicates GetDC candidate and production and needs an occlusion-independent oracle for the new defect.
- `crates/ferrostage-platform/src/monitor.rs`: PlatformError and documented physical geometry / scale provenance.
- `crates/ferrostage-platform/src/watch.rs`: fail-closed topology event state machine and recovered Changed publication.
- `crates/ferrostage-ui/src/shell.rs`: initial monitor state, shared display panel, manual retry, and sticky global error.
- `.trellis/tasks/archive/2026-10/10-14-07-gpui-shell/research/product-v15-getdc-diagnosis.md`: measured previous leak and bounded real-owner fix.
- `.trellis/tasks/archive/2026-10/10-14-07-gpui-shell/research/product-independent-v15-review.md`: prior fix limits and pending actual application context verification.
- `C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/winsafe-0.0.29/src/user/{guards.rs,iterators.rs,handles/hwnd.rs,handles/hdc.rs}`: exact pinned release, enumeration and RAII contracts.

## External References

All read live from Microsoft Learn on the research date. Paraphrases below are limited to the points relevant to this repair.

- [EnumDisplayMonitors](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaymonitors): NULL HDC plus NULL clip enumerates all monitors; non-null HDC restricts enumeration to its visible region and supplies monitor callback DCs.
- [GetDC](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdc): NULL owner obtains screen DC; common DCs require same-thread ReleaseDC.
- [GetDCEx](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdcex): WINDOW selects full window rectangle, CACHE requests cached DC; CLIPCHILDREN/CLIPSIBLINGS exclude child/sibling regions; release is on the acquiring thread.
- [CreateDCW](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-createdcw): named display device DCs and DeleteDC lifetime; not exposed by pinned winsafe.
- [GetWindowDC](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getwindowdc): nonclient painting API, NULL-owner primary-monitor qualification.
- [EnumDisplaySettingsW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-enumdisplaysettingsw): mode dimensions are physical pixels regardless of DPI virtualization.
- [Mixed-mode DPI](https://learn.microsoft.com/en-us/windows/win32/hidpi/high-dpi-improvements-for-desktop-applications): thread contexts can differ, and callbacks may run with the window's DPI context.

## Related Specs

- `.trellis/spec/backend/quality-guidelines.md`: no unsafe first-party code; every enumeration releases DCs; real Windows GDI regression and complete monitor/DPI verification required; stale topology cannot hide resource exhaustion.
- `.trellis/spec/backend/error-handling.md`: typed library errors; do not swallow failures or add unsafe error shortcuts.
- `.trellis/spec/frontend/state-management.md`: shared display panel, one binding authority, current complete topology before Apply, failed topology disables projection.
- `.trellis/spec/guides/index.md`: verify suspected findings against actual source before priority decisions.

## Caveats / Not Found

- Researcher ran no native application/probe and made no desktop changes; only Root may claim new native reproduction or candidate acceptance.
- Physical hotplug, clone/mirror mode and different hardware are not newly verified here.
- A clone connection with multiple active monitor devices currently becomes AmbiguousConnection; that is separate from the screenshot's IncompleteTopology and is not evidence to weaken completeness.
- PlatformError::IncompleteTopology is used both for device-versus-geometry mismatch and before/after device snapshot changes; logs should retain structured fields instead of displaying only the generic string.
- Product code was not edited. Only this task's research output was written.

