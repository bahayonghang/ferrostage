# V16 independent review and native handoff

Reviewed 2026-10-15 against HEAD `8ae980ced5ccb83b7804b698d3cceef474b80633` and the explicitly approved sixth maintained package. **No product source defect found; strict lint/typecheck/fmt pass. Current V16 is released to Root for native verification. Gate B remains INCOMPLETE.** No product source, existing receipt, freeze, manifest or task acceptance checkbox was changed by this review.

## Findings (fixed)

- Evidence interpretation corrected in this report and `product-independent-v16-readback.json`: the sealed `product-v16-baseline-and-lock.json` labels EXE `006904815a9a3b0af5367e234442bbd6b18f61c37ff8f0e22b48ca3d3c775dbe` as an `unchanged_v15_artifacts` binary. This is **not** the immutable V15 frozen EXE `d40bb0d113e363df6f2bae5d331fd13d8473a8fbcf5820e4438b3da5489e8fab`. Do not infer V15 executable identity or native acceptance from that field. Its provenance is unestablished by this review. Root confirms it did not independently verify the V15 EXE on this continuation and launched no V15 native app. Current V16 was freshly built and independently verified below, so this historical label does not prevent V16 native verification.

## Findings (not fixed)

- The original baseline receipt's misleading label is retained because it is sealed immutable evidence. Root agreed to this additive correction rather than rewriting that receipt or its manifest. No claim is made that its binary matched V15.
- Existing direct upstream HarfRust lint failures remain outside this approved four-file AccessKit scope. Its unused-function warning remains visible in consumer logs; no lint waiver was added. The required strict first-party consumer lint passes.
- Actual V16 hidden-window destruction, external live UIA, product teardown, physical/true single-zero and presenter notes display are not accepted by this review. Root exclusively owns native execution; actual notes UI remains task08 work. The user deferred further size work. These are evidence/scope limits, not additional code defects found here.

## Source and contract review

Independent reconstruction from the cached original archive and approved patch passes for all21 product files. Raw approved patch SHA is verified before CRLF transport normalization; only util/context/node/text differ from the published19-file package, and the two exact upstream license notices are added. Both original manifests, package lock, tests, example, copyright headers and other source bytes are preserved. Provenance, ADR-008, AGENTS, vendor README and maintained-dependencies describe the same bounded adoption and removal conditions.

`util::client_top_left` immediately reads the actual ClientToScreen BOOL result through `.ok()`. Only ERROR_INVALID_WINDOW_HANDLE becomes UIA_E_ELEMENTNOTAVAILABLE; other errors propagate unchanged. No preliminary IsWindow race, unconditional zero-coordinate fallback, accessibility disable or blanket suppression was introduced. Context propagates Result to all four actual COM entrypoints: BoundingRectangle, ElementProviderFromPoint, RangeFromPoint and GetBoundingRectangles. Missing node bounding boxes still return zero UiaRect without querying the HWND; empty text rectangles still return null first. Valid origin offsets, text transforms and SAFEARRAY population are unchanged. This does not establish handle-reuse safety or a provider drain protocol.

The separate harness matches every product runtime source byte except the explicit test-module redirection in lib.rs. Its exact manifest adaptation removes GUI development dependencies/example and adds the standalone workspace plus Root-only feature; these changes do not enter production. Source inspection and sealed8-test receipts cover invalid handle, all four real COM callers with retained Context, live coordinates/text/SAFEARRAY and live/invalid empty-geometry semantics. The Root-only compiled test retains providers on another thread, destroys16 owned hidden windows, and then checks64 COM calls. It remains NOT_RUN by this reviewer; its sequencing does not claim simultaneous destruction/conversion coverage.

Structured TOML comparison against committed Cargo.lock proves the only retained graph change is removal of the same-version AccessKit registry source/checksum. All744 package records otherwise match; no versions/edges, policy exceptions or GPUI families change. The initial resolver drift is preserved as intermediate evidence and was corrected before final-lock checks. All97 first-party files in the V15 freeze match V16. The core/snapshot/notes boundary is unchanged.

## Verification

Fresh independent checks:

- Four-crate app/UI/document/platform all-feature/all-target Clippy, locked/offline with `-D warnings`: PASS.
- Same four-crate typecheck: PASS.
- Workspace fmt: PASS; `git diff --check`: PASS.
- Exact package reconstruction,648 frozen source hashes and aggregate,62 implementation manifest records, harness equivalence, current EXE/PDFium/PDF/lock: PASS.

The first lint attempt could not open the existing external mbx cache and exited1 before compilation. It remains in `product-independent-v16-lint.log/.json`. The cache-authorized rerun passed. This environment failure is not relabeled as a product regression or deleted.

Verified reuse, **not independently rerun**: V16 AccessKit8 tests, UI82 tests (59 GPUI/23 pure), default just-ci151 tests, full all-feature deny and actual gpui,pdfium build all pass in sealed receipts. The hidden regression compiles. V15 PDFium/document2 evidence is reused with unchanged first-party/document/PDFium inputs, fixture and DLL; this is not presenter notes UI evidence. Broad suites were not repeated because source review found no change or unresolved concern requiring them.

## Immutable identity and release

- Freeze SHA: `0e2b1ed8ef9299676ba249e7600bd8ab6734f00c6ae17f677467405ad60402d3`.
- Source648 aggregate: `3e208cf586dcf37ab0420228e8eb3fba8ee07c0b73a8597c318a7f6a219c35b8`.
- EXE: `cf228c03655ddd45d25520bd88dab025c219af9bec9195314ce541dbb1cd89d1`.
- Implementation evidence62 manifest SHA: `1e004a195f04725d2fe511036c1013d6be4bd5346ffc01e93dae76b4bac71f63`.
- Notes PDF: `e939e35f931c184fd93b2e9b95e560db73ba6a6f6dfccbb0963262f0b359dfeb`.
- PDFium: `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`.

All reviewer commands returned. The10:08:32Z process readback is empty for product/compiler/test processes; no native HWND or app was launched by this reviewer. Root may take exclusive native execution, first verifying fresh window inventory and identity, then running its owned hidden-window regression and core-only plan. Use the current V16 freeze as authority; preserve all historical failures. No further product changes without coordinating release of the freeze. Gate B remains INCOMPLETE;08–11 remain dependency-gated/P2. No commit, push, archive, install or add_session performed.
