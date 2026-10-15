# Independent V15 review — product repair, notes fixture, AccessKit candidate

Active task: `.trellis/tasks/10-14-07-gpui-shell`.

**Conclusion: V15 F13 product source and lower-layer notes evidence PASS independent review. No product source fix is required. The four-file F14 candidate supports its explicitly limited invalid-HWND error repair, but is research-only and requires separate maintenance authorization before adoption. Product F14 remains unresolved FAIL.**

Root reported a user Escape stop during Computer Use while this review was active. All reviewer commands have returned. No new native test or application was started by this reviewer. V15 product-native acceptance is **NOT_RUN / user-stopped**; this handoff is readiness for a later authorized native run, not an instruction to resume the stopped desktop session. Width/size work is user-deferred. Gate B remains INCOMPLETE, physical/true single-zero items NOT_RUN,08–11 gated and P2 unchanged.

## Findings (fixed)

No product issue required an independent fix. Only new review scripts/receipts/report were written. An audit helper initially assumed all manifests used a dictionary; native V14 uses a list of path records. The read-only helper failed before writing its receipt, was corrected to accept both schemas, and then verified every entry. No product or historic evidence was changed by this correction.

## Findings (not fixed)

- **F14 is not repaired in FerroStage.** Product Cargo manifests, lockfile and all vendor sources remain byte-identical to V14. A clean future V15 close alone cannot establish that the unadopted AccessKit defect is gone.
- **F13 native application verification remains pending.** The resource regression is strong evidence for the specific leak, but long-running polling and complete topology/scale observations in the actual GPUI DPI context still need Root's native run.
- **Notes UI remains unavailable in07.** The actual compiled PDF and PDFium model/crop results do not provide native presenter notes or audience raster acceptance.07 still renders placeholders;08 remains gated.
- No simultaneous conversion/destruction stress, HWND reuse, external UIA client, original upstream GUI suite or complete product shutdown is established by the isolated candidate. Other native errors are preserved by code inspection, not an injected non-invalid-HWND failure test. Empty rectangle/range behavior is also source-verified rather than separately exercised by the six candidate tests.

## F13 source and regression

Read `product-v15-getdc-diagnosis.md`, production `windows.rs`, the complete new resource test, the red/green records, and pinned winsafe0.0.29 source. `ReleaseDCGuard::drop` calls ReleaseDC only when `hwnd.as_opt()` succeeds; Handle::as_opt rejects NULL. The old `HWND::NULL.GetDC()` therefore acquired a valid common screen DC that its guard did not release. This is an actual ownership defect, not a guess based only on the wrapper's type name or the native failure's zero LastError.

The production change uses `HWND::GetDesktopWindow().GetDC()` and retains the existing local guard. It introduces no unsafe block, dependency, shared DC cache or new error policy. The same thread owns acquisition, enumeration and guard drop. Early Result returns also drop the guard. Callback handles do not escape. Device identity matching, native geometry/scale provenance, before/after topology validation and unavailable-topology safety remain intact; stale topology is not promoted to valid after failure.

The actual red resource test compared all three complete MonitorInfo records and found candidate GDI2→2 versus old production2→66 over64 enumerations. Green found1→1 for both candidate and repaired production. The sentinel DC is intentional because winsafe represents a zero GetGuiResources count as an error. The test does not create/move windows. On the red code, the comparison genuinely used the old all-screen acquisition; on green, the duplicated candidate is no longer an independent old-path oracle, so the preserved red equality evidence matters. No fixed GDI quota or hardware independence is inferred. A non-NULL DC's visible-region semantics make the real application topology/DPI retest necessary even though this desktop's complete comparison passed.

Only windows.rs is a runtime product delta. windows/tests.rs adds the regression. The unused Handle import was removed before final strict gates; the earlier green receipt's unused-import warning is preserved and is not presented as final lint evidence.

## Real notes input and document/PDFium test

The fixture is original TeX using documented Beamer/pgfpages syntax, with public/private canaries and distinct notes on each page. Pympress is a documented workflow reference, not copied implementation code or a linked dependency. The TeX declares physical page labels1/2/3/3/4 because this installed XeLaTeX/pgfpages combination otherwise labels virtual pages incorrectly. The PDF is generated directly from TeX; it is not post-processed. This deterministic fixture does not prove arbitrary third-party PDF labels are correct.

Read all new TeX, verification script/record, README and `notes_native.rs`; independently viewed the two overlay preview pages. They show readable changing public content and private03A/03B notes on distinct halves. The frozen verification records bind all five previews and side-specific text canaries. The product test uses actual PDFium metadata and render worker, verifies Right layout, five physical/four logical pages, labels and groups, sequential overlay stepping and NextLogicalSlide skipping the second overlay. Snapshot-derived audience crop is LEFT_HALF. Every page's audience and notes frames are independently compared pixel-by-pixel against the corresponding full-page half, with a small edge exclusion and <=1 channel rounding tolerance; both halves contain nonempty distinct content and both overlay halves change.

The real final PDFium receipt contains two separate integration processes: unchanged `native.rs` and new `notes_native.rs`, one passing test each. The initial LibraryUnavailable failure from process-global PDFium reinitialization is preserved as a test-harness lifecycle failure. Isolating the new test in its own binary follows the existing runtime contract. This test proves document/model/crop behavior, not a GPUI upload/render path, reverse-navigation UI behavior, native notes availability or a performance target.

Frozen notes PDF SHA256: `e939e35f931c184fd93b2e9b95e560db73ba6a6f6dfccbb0963262f0b359dfeb`.

## F14 candidate: limited approval recommendation

Read the proposal, exact four-file patch, standalone source/harness, upstream provenance and all final candidate receipts, including Root's `product-v15-root-a11y-results.md`. Independently reconstructed the unified diff from the cached registry originals and candidate bytes: it exactly matches the submitted patch. The harness bytes in the candidate exactly match the attached research harness.

The captured native error is a direct AccessKit COM-provider ClientToScreen unwrap panic. Weak Context lookup does not prevent a provider that already holds a strong Arc from querying an HWND after destruction. GPUI callback tokens and channel cancellation do not intercept this direct COM route. Taking/dropping the adapter earlier alone cannot revoke such an Arc; a full drain protocol would add synchronization and SendMessage/COM reentrancy obligations across another package. The actual conversion-result check is the smallest supported repair for the recorded failure.

| Candidate change | Independent assessment |
| --- | --- |
| util.rs | Returns Result<Point>; tests the actual ClientToScreen BOOL with `.ok()` immediately. Only HRESULT_FROM_WIN32(ERROR_INVALID_WINDOW_HANDLE) becomes UIA_E_ELEMENTNOTAVAILABLE. Other error objects are returned unchanged; success coordinates are unchanged. No IsWindow check/use gap, panic catcher, fallback coordinates or logger suppression enters product code. |
| context.rs | Propagates the internal Result; no ownership/public interface redesign. |
| node.rs | BoundingRectangle, ElementProviderFromPoint and RangeFromPoint propagate the failure through their existing COM Result boundary. Missing bounding_box still returns the original zero rectangle without conversion. |
| text.rs | GetBoundingRectangles propagates conversion failure after the existing empty-range early return, preserving null/empty semantics. |

The final six isolated tests exercise native invalid-HWND and live controls, real retained Context/providers, all four actual in-process COM coordinate methods, text `AB`, exact live rectangle/SAFEARRAY values and explicit SAFEARRAY cleanup. Root's separate16-cycle hidden-window test creates only owned invisible HWNDs, builds providers while live, retains Context on a worker, destroys each HWND on its owning thread, and then receives64 normal UIA unavailable errors. Channel synchronization deliberately places queries after destruction; this is deterministic post-destruction evidence, not random overlap, COM marshaling from an external client, or handle reuse coverage. The code and receipts support the limited invalid-HWND conversion repair without claiming complete lifetime safety.

**Recommendation:** the four-file candidate is concrete and suitable for a narrowly bounded adoption decision. It has no public API or dependency-version change and introduces no new license exception; upstream0.34.0 is MIT OR Apache-2.0. However, adopting it adds a sixth maintained package beyond ADR-008's five-package authorization. Root must obtain the user's additional authorization using this exact reviewable scope. This reviewer did not adopt it or infer permission from F13 implementation authorization.

Adoption would need exact upstream/checksum/reconstruction provenance, original license files and manifest, an upstream tracking/removal condition, rebase checks for all four callers, maintained COM regressions, full product deny/lint/typecheck/build and Root native close/query verification. The research manifest's removed GUI dev dependencies and redirected test module are isolation details, not an approved replacement production package manifest. Existing valid accessibility and empty semantics must remain covered; tests for broader retirement/handle reuse require a separately justified design rather than being asserted by this patch.

Candidate patch SHA256: `93563d304909e2bca657dc0136382ba9179e638b71799d9c6823918ad2a9f2f0`.

## Verification and evidence reuse

| Check | Result and applicability |
| --- | --- |
| Independent strict Clippy | app/UI/document/platform, all features/all targets, locked/offline, -D warnings: exit0 |
| Independent explicit typecheck | Same four packages/features/targets: exit0 |
| Current product CI | Final just-ci receipt/log verified:151 tests, format/default lint/test/deny PASS |
| Current full UI |82 =59 real GPUI+23 pure, PASS; UI runtime/vendor sources unchanged from reviewed V14 |
| Current F13/PDFium | Final real DC1 and document/PDFium2 receipts verified and reused; no additional native tests after user stop |
| Current dependency/build gates | Full all-feature locked/offline deny and gpui,pdfium native build receipts verified PASS |
| Candidate | Six final isolated tests plus Root's one16-cycle/64-COM-query hidden-window test verified; no product acceptance inferred |

The known dependency HarfRust shape warning remains visible. No new suppression or policy waiver exists. Already applicable full checks were not repeated. The reviewer started no native app, hidden HWND test or desktop operation. Current product and candidate receipts are distinct from this reviewer's independently executed lint/typecheck.

V9/V11/V13/V14 functional UI observations retain their previous version-scoped applicability for unchanged settings/roles/Identify/timer/save paths. F13's new acquisition needs actual complete topology confirmation, so prior native topology cannot prove the replacement path. V14's scoped clean F12 trigger/owner closures remain historical successes; its later F13/F14 failures remain failures. No whole Gate B, full design or native notes PASS follows from reuse. Additional width/size tests are deferred by the user, not reclassified as external blockers or silently passed.

## Identity and handoff

Read-back at2026-10-15T08:50:33Z matched all622 frozen inputs and recursively bound executable/DLL/fixture/config/check records with no mismatch. Against V14:606 unchanged,3 changed and13 added, matching the recorded source delta. Product manifests/lock and every vendor source are identical to V14.

- Source aggregate: `fdc00fc36ad02e46234398899ef924892e690810caf24f290cc5b0ec070d835a`.
- EXE SHA256: `d40bb0d113e363df6f2bae5d331fd13d8473a8fbcf5820e4438b3da5489e8fab`.
- Freeze SHA256: `0d6a9c2a7bbb10fe6f6c7d24e7c4f7faf10340d456fd5185fdfe631acdc4370f`.
- Product manifest64 SHA256: `458707b87d7b4d3a929412d24d2bfd3433226d041bbcfb5d8f429fb335179324`.
- Native V14 manifest125 SHA256: `477cfa6ecf65ba131eb60e705bbdbb955b77d7299b92990a16d203cf9b5c8590`.
- Prior independent V14 manifest15 SHA256: `418c5f3ed6abfd7f89a94b22cce2e22c21f69abbef6941b0991653ea92261b46`.

All referenced manifest entries were read back. Root's later hidden-window receipt/log are additionally hash-bound in the independent identity record; the log contains all16 cycle completions. Every reviewer tool session returned, including the read-only identity helper. No host-wide process-empty claim is inferred from sandbox visibility; Root owns host inventory and native launch. Source review is complete, no product mutation is pending, and the frozen build is ready for later authorized core-native verification. The current user stop remains respected: V15 product-native NOT_RUN / user-stopped.

No commit, push, archive or add_session was performed. All existing uncommitted work and historical failures remain preserved.
