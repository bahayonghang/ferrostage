# Root execution of the isolated F14 native regression

2026-10-15T08:41:06Z. After the implementer returned all commands and handed off the frozen V15 product, Root ran the separately compiled research candidate's `root-hidden` mode. `product-v15-a11y-root-hidden.json` and `.log` are the original command receipts; exit0, one test PASS, 12 filtered out.

All16 cycles create a test-owned invisible STATIC HWND, retain the real AccessKit Context/providers on a worker, destroy the HWND on its owning thread, and then query the four actual COM coordinate methods. BoundingRectangle, ElementProviderFromPoint, RangeFromPoint and GetBoundingRectangles all returned UIA_E_ELEMENTNOTAVAILABLE normally. The test creates no visible/activated window, uses bounded channel waits, and does not interact with another application's window.

This is deterministic post-destruction coverage with a strong Context held across native destruction. It is not randomized conversion/destruction overlap, HWND reuse coverage, an external UIA client, or full FerroStage shutdown acceptance. The isolated candidate is still not linked to the product. V14's captured F14 panic remains FAIL, and V15 still uses the unpatched approved dependency graph. The product gate is unchanged.

The independent reviewer has been given these receipts alongside the exact four-file candidate, source identity, red/live controls and six earlier COM tests. Adding AccessKit to the product remains outside ADR-008's approved five-package set and awaits a concrete reviewed scope decision. No dependency, lockfile, vendor source, license exception or product policy was changed by this Root run.
