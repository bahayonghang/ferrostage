# V15 authorized product handoff

Read this report and the latest `implement.md` tail explicitly: earlier task/context injection is truncated beyond 32768 bytes. All changes remain uncommitted. Root owns desktop/native verification and the serial independent check. No process was launched as the FerroStage native application by the implementer.

## Implemented product scope

- `crates/ferrostage-platform/src/windows.rs`: acquire the DC using the real desktop HWND so pinned winsafe releases it. No unsafe/dependency/policy change. The real Windows regression in `windows/tests.rs` first measured GDI 2→66 for 64 old production enumerations; after repair it remains 1→1. All three MonitorInfo records match the old all-screen path. See `product-v15-getdc-diagnosis.md` and immutable red/green receipts.
- New `crates/ferrostage-document/tests/notes_native.rs`: actual PDFium test of Root's TeX-compiled notes fixture. It validates Right layout, labels 1/2/3/3/4, five physical/four logical pages, overlay stepping/logical skip, nonempty distinct public/notes crops on all pages, snapshot-derived audience pixels matching only the left full-page crop, and changing overlay content on both halves. Existing `tests/native.rs` is byte-identical to V14.
- Root owns the new fixture folder and the backend-quality/maintained-dependencies contract updates included in the freeze. The six top-level fixture files and five preview images are pinned. No UI rendering feature was added: real notes display in the task07 placeholders remains NOT_IMPLEMENTED/NOT_RUN until task08.

## Checks returned

| Check | Result |
| --- | --- |
| Real DC resource regression | red actual assertion FAIL; final green1 PASS |
| Final `just ci` | PASS, 151 tests; fmt, strict default Clippy, default tests and deny |
| Full UI/test-support suite | PASS,82 =59 actual GPUI+23 pure |
| Actual document/PDFium suites | PASS,2 tests in separate integration processes |
| Strict app/UI/document all-feature/all-target Clippy | PASS |
| App/UI/document all-feature/all-target typecheck | PASS |
| Full product `cargo deny --all-features --locked --offline check` | PASS, unchanged policy |
| Native app `gpui,pdfium` build | PASS |

The known upstream harfrust unused-function warning is preserved in native build/consumer receipts; no waiver was introduced. The first notes test attempt failed with LibraryUnavailable because the earlier test had shut down process-global PDFium; the failure receipt is preserved. Moving the new test to its own integration binary respects the existing runtime lifecycle. This harness failure is not recast as a fixture or product regression. Final `document-isolated` receipt pins the exact PDF/TeX/DLL hashes.

## F14 remains unresolved in the product

The direct COM-provider panic bypasses the GPUI callback/task cancellation boundary. `product-v15-a11y-proposal.md` compares early adapter retirement/drain against checking the actual ClientToScreen result. `product-v15-accesskit-proposal.patch` is a concrete four-file AccessKit candidate, in research only. No sixth maintained package has been adopted; Cargo manifests/lock and every product vendor file remain unchanged from V14.

Unmodified upstream native leaf red reproduces the exact invalid-HWND panic; live desktop positive passes. The initial BOOL/Result compile failure remains separate. Final isolated candidate6 tests PASS: all four actual COM coordinate methods with retained invalid-HWND Context and live value-preserving controls, plus the leaf cases. These tests create no windows. The Root-only 16-cycle hidden-window destruction harness is compiled but NOT_RUN by the implementer; exact command is in the proposal. A clean current product run would not prove this unadopted race fixed.

## Frozen identity

- `product-native-freeze-v15.json`: SHA256 `0d6a9c2a7bbb10fe6f6c7d24e7c4f7faf10340d456fd5185fdfe631acdc4370f`.
- 622 source/contract/fixture inputs; aggregate `fdc00fc36ad02e46234398899ef924892e690810caf24f290cc5b0ec070d835a`. Against V14:606 unchanged,3 changed (platform adapter/test + Root maintained contract),13 added (new integration test, Root backend contract,11 fixture inputs).
- EXE `d40bb0d113e363df6f2bae5d331fd13d8473a8fbcf5820e4438b3da5489e8fab`.
- PDFium DLL `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`.
- New native fixture PDF `e939e35f931c184fd93b2e9b95e560db73ba6a6f6dfccbb0963262f0b359dfeb`; TeX `9552dc97c6ac5216e488b448846773befc8cb7952fd0665197db25bec275bc34`.

`product-v15-evidence-manifest.json` seals new product receipts, candidate source/harness/patch and configuration-input copies. Existing V6–V14 freezes and native evidence were not overwritten. No Cargo/native process remains at the recorded process-closeout time; all command sessions returned before handoff.

Next: serial independent check, Root's bounded research native harness, then the actual V15 app using the notes PDF and full backtrace. F13 long-duration native resource behavior and F14 product shutdown remain open. User deferred additional width/size tests. Physical unplug/reconnect/port/real single-zero items remain NOT_RUN. Gate B INCOMPLETE,08–11 gated/P2 retained. No commit, push, archive or add_session.
