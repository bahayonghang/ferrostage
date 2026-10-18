# V16 recovery evidence audit — 2026-10-15

**Current V16 product and bounded native evidence remain applicable. No new product defect or identity mismatch found. Gate B remains INCOMPLETE.** This is a read-only product/evidence audit; only this new report, its script and JSON receipt are added. No sealed record, product file or acceptance checkbox was changed; no app, hidden HWND, compiler or test suite was launched.

## Findings (fixed)

None. The existing additive correction in `product-independent-v16-review.md` remains necessary: `product-v16-baseline-and-lock.json`'s historical `unchanged_v15_artifacts` EXE label does not prove identity with the frozen V15 EXE. That sealed receipt is preserved. Current V16 identity is separately verified.

## Findings (not fixed)

- Physical unplug/reconnect/port changes and measured disappearance latency, true single-display modes and true zero-display recovery remain NOT_RUN. They require the corresponding real environment; disabling roles on this three-display desktop does not substitute. These block complete Gate B and therefore task08–11 integration under the current plan.
- Additional size-matrix work remains user-deferred. Do not perform it implicitly or convert that deferral into a PASS.
- The PRD's combined Audience-focused PageDown/B/Escape and no-dialog/focus/cursor-change criterion is still not fully accepted. V16 establishes navigation and Presenter-only Jump routing; prior receipts establish portions of B/Escape behavior. The remaining complete cursor/focus observation is a potentially feasible native check, subject to whether the active capture API actually records cursor appearance. Review available capability, then record only observed results. Repeating all V16 core flows is unnecessary.
- Cross-window drag/drop, live system-theme changes and transient loading/Curtain visual capture retain UNVERIFIED boundaries. Their current feasibility needs explicit capability inspection; historical tool limitations do not establish that the current native tool cannot perform them. Root reports the current native Computer Use inventory is available and contains no FerroStage window. This reviewer performed no desktop interaction.
- Exact AccessKit overlap races, HWND reuse and complete provider-drain coverage are not established by the bounded repair. These are explicit scope limits, not newly found defects or a request to expand the approved four-file patch.
- Task07 Audience remains a snapshot-only numeral placeholder (`crates/ferrostage-ui/src/audience.rs`), not real PDF/notes rendering. Task08 remains responsible for those pixels; the real notes fixture and document/PDFium tests do not prove presenter notes UI.
- Known upstream HarfRust direct lint failures remain outside this change; no waiver was added. First-party strict consumer lint is PASS.

Root owns corrections to stale current-status descriptions in parent planning/traceability documents. `docs/PROJECT_PLAN.md` is included in the frozen648 inputs: changing that file requires an additive applicability delta, rather than continuing to claim all648 old hashes match after the edit. This report describes the snapshot captured in the JSON receipt.

## Verification

Executed `python .trellis/tasks/10-14-07-gpui-shell/research/resume-2026-10-15-v16-audit.py`; result PASS, zero mismatches. The exclusive-create script cannot overwrite its existing receipt or any V16 artifact.

| Check | Result |
| --- | --- |
| Branch / HEAD | `main` / `8ae980ced5ccb83b7804b698d3cceef474b80633`; existing V16 work remains uncommitted |
| Frozen source/contract inputs |648/648 matching; aggregate `3e208cf586dcf37ab0420228e8eb3fba8ee07c0b73a8597c318a7f6a219c35b8` |
| Implementation manifest |62/62 sizes and hashes match; SHA `1e004a195f04725d2fe511036c1013d6be4bd5346ffc01e93dae76b4bac71f63` |
| Independent product manifest |15/15 match; SHA `1677b1b77b5c4b12e775de472d395019a52334b18ae7fa927210be89c00baafd` |
| Native manifest |138/138 match, including47 original JPGs; SHA `cad5c4ee34a7573c774efed77edb10cb53895201736fed2afd390501b13f91dc` |
| Independent native manifest |6/6 match; SHA `10f114c1b9d3c3e69764eebc0d64e6cbd6842e4970bf73f628ee431083ae56b8` |
| Lock and harness |All frozen harness file hashes match. Structured744-package lock comparison permits only same-version AccessKit registry source/checksum removal |
| Lint |PASS, verified reuse of returned strict four-crate all-feature/all-target Clippy receipt/log with `-D warnings`; not rerun |
| TypeCheck |PASS, verified reuse of matching four-crate check receipt/log; not rerun |
| Tests |PASS, verified returned receipts and log totals: default151, UI82, COM8; not rerun |
| Policy / native build / fmt |PASS, verified returned full-feature deny, gpui/pdfium build and independent fmt receipts/logs; not rerun |
| Hidden HWND regression |Verified returned exit0,16 distinct destruction cycles/four COM queries each; no new HWND execution |
| Native run closeout |PIDs42672/58568,126/48 stdout lines, empty stderr, no scanned panic/error/failed/invalid-window diagnostics; exit0 precedes each recorded fresh empty inventory |
| Fresh process observation |`Get-Process` matched no FerroStage/Cargo/rustc/Clippy/rustfmt/just/AccessKit process. Win32_Process CIM access was denied, so no fresh command-line/parent inventory is claimed |
| Whitespace |Fresh `git diff --check` PASS |

Successful receipts and complete relevant log tails, test counts, argv, timestamps, hashes, raw native launch/exit/post-exit records and process-query limit are retained in `resume-2026-10-15-v16-audit.json`. Existing cache-permission failures remain preserved alongside subsequent passed receipts; they are not relabeled as passes. A PowerShell brace-expansion parser mistake during this review ran no command and changed no file; the corrected read succeeded.

Runtime identities recomputed from disk:

- Freeze: `0e2b1ed8ef9299676ba249e7600bd8ab6734f00c6ae17f677467405ad60402d3`.
- EXE: `cf228c03655ddd45d25520bd88dab025c219af9bec9195314ce541dbb1cd89d1`.
- PDFium DLL: `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`.
- Real notes PDF: `e939e35f931c184fd93b2e9b95e560db73ba6a6f6dfccbb0963262f0b359dfeb`.
- Cargo.lock hash and resolved runtime paths: exact records in the JSON receipt.

Source inspection confirms the same bounded AccessKit flow: actual ClientToScreen result checked immediately, only invalid-HWND mapped to UIA_E_ELEMENTNOTAVAILABLE, Result propagated through all four callers, missing bounding boxes/empty text ranges handled before conversion. No first-party runtime diff exists against HEAD; current changes adopt the already reviewed dependency patch. Existing source-bound checks therefore suffice without another broad build/test pass.

Next: Root may complete genuinely feasible unfinished native observations and documentation applicability updates, preserving physical/deferred limits. No task08 start or Gate B PASS follows from this evidence audit. No commit, push, archive or add_session occurred.
