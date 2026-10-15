# Native core defects F13/F14

## 1. Root cause categories

F13 is an implicit assumption and cross-layer lifetime contract failure: a valid desktop DC returned by a safe API was assumed to be released by its guard. winsafe0.0.29's guard instead skips ReleaseDC when its HWND owner is NULL. The200ms monitor poll accumulates DCs. V14 exposed failure after about36minutes; this is not an exact exhaustion limit. The V15 actual native regression distinguishes resource leakage from transient enumeration failure: old production enumerate64 grows GDI2→66, while the non-null desktop-owner candidate16 stays2→2 and preserves full MonitorInfo equality for all3 current monitors. The corrected production path64 stays1→1. Native GUI DPI context remains a separate verification.

F14 is a separate cross-layer lifetime gap. AccessKit's COM providers can upgrade their weak context and call ClientToScreen on the stored HWND directly, outside GPUI's action-receiver and platform-callback guards. V14 dark stderr records util.rs380 Invalid window handle panic during titlebar Close. Exit0 and empty windows do not constitute a successful close. An in-flight upgraded context must also be covered by any repair.

## 2. Why earlier repairs were insufficient

The V8/V12/V13/V14 repairs cover Shell task ownership and GPUI Window event callbacks. They cannot fix the unrelated desktop-DC guard or cancel a direct COM provider method in another package. Repeating short clean native runs could miss both the long-poll resource growth and asynchronous provider race. Preserve prior scoped PASS results and failure logs rather than discarding either.

## 3. Prevention mechanisms

| Priority | Mechanism | Status |
| --- | --- | --- |
|P0|Actual per-process GDI-count red/green test with full topology equality|V15 red and green recorded; independent review pending|
|P0|Real application revalidation of monitor mapping under GPUI DPI context|Pending V15 native, no dimensions matrix needed|
|P0|Separate direct COM provider race analysis and executable candidate regression|Investigation/proposal in progress; no additional package adopted|
|P0|Full post-exit stdout/stderr plus source identity and final inventory|V14 three runs sealed,125files/45original screenshots; dark is FAIL|
|P1|Real TeX notes fixture with page/overlay/private-canary checks|Root compiled and visually inspected5pages; product PDFium test pending|

## 4. Systematic expansion

Audit the acquisition/release pairing used by repeated monitor enumeration, including early error exits, without broad platform refactoring. For F14, identify every caller of the failing coordinate conversion and its existing UIA error behavior; clearing an accessibility tree or moving adapter destruction earlier is insufficient if a provider already holds a strong context. Do not suppress all native handle errors or disable accessibility. A dependency maintenance expansion requires a concrete bounded proposal under ADR-008 before adoption.

## 5. Knowledge capture

Root updated backend/quality-guidelines.md with actual native resource and real notes-fixture evidence requirements; frontend/maintained-dependencies.md now distinguishes the direct COM provider boundary. No generated spec template exists in this repository. User deferred further dimension testing and prohibited commits/push/archive/add_session; those boundaries persist. F14 remains unresolved until a tested authorized repair exists.
