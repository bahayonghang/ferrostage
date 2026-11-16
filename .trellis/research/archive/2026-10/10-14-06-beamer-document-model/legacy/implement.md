# Implement — 06

1. **I-1 研究门**：读behavior证据/PGF尺寸推导，核05metadata/rotation；主源PDF规范核媒体支持字段路径，用自建夹具确定子集，不能从Pympress README反推实现。
2. **I-2 纯content map**：core新增SlideMap/TOP/BOTTOM，document构建器；空/缺失/非数字/连续与非连续重号，首尾及有邻居逆向测试，logical逆向仅比组。
3. **I-3 layout/notes**：实现比例候选/来源warnings/五种手动覆盖、独立识别开关、physical_pages开关、按物理步多notes配对；宽图/竖版误判和混合尺寸反例必须测试。
4. **I-4 只读结构**：Movie/Screen/RichMedia支持子集/Rect/Embedded/source/time字段，构建字段路径明示的自制PDF，missing/unsupported可诊断，不制造默认“已还原”。
5. **I-5 core/document接线**：从04descriptor升级map元数据，导航/preview用content，project仍无notes；更新所有现有core/document测试，不将整个LoadedDocument交观众。
6. **I-6 PoC3**：A常规/overlay，B左右与上下，C独立多notes/orphan，D媒体参数，E普通宽图/重复标签；记录期望map/crops/warnings与actual，真实PDF另作限制验证。
7. **I-7 评审**：core纯依赖/notes与paths隔离、范围与手动覆盖、结构只读/无compiler，跑命令；05实际native提供labels/尺寸的组合测试通过。

命令：cargo test -p ferrostage-core -p ferrostage-document -p ferrostage-pdf；cargo test -p ferrostage-pdf --features pdfium；cargo tree -p ferrostage-core；just ci。
回滚：map/descriptor跨core/document同次；candidate结构失败保旧文档。夹具写入独立temp目录或本项目自制fixture，用户PDF哈希不变。

## Execution
- I-1 primary PDF1.7 tables 9.9/9.13-16/9.20-24/9.30-31 and Extension3 9.50a/9.51a-c read; typed subset under implementation.
- I-2/I-5 first core compatibility test: RUNNING (core-initial.log).
- core-initial.log: exit101, sandbox build-lock AccessDenied. Approved retry RUNNING (core-retry.log).
- core-retry.log: exit101 compile error (PhysicalPage intentionally lacks Ord); compare contained physical numbers. Core compatibility rerun RUNNING (core-map.log).
- core-map.log: PASS exit0, 26 tests plus doc-tests (0); I-2/I-5 compatibility intact. Development-only Beamer generation RUNNING (beamer-compile.log).
- beamer-compile.log: exit0 for both actual Beamer sources; compile only, model acceptance PENDING. I-3/I-4 builder and extraction implementation underway.
- I-3/I-4 targeted core/document/pdf suite RUNNING (model-initial.log).
- model-initial.log: exit101, lopdf Dictionary method is has; corrected. Targeted rerun RUNNING (model-tests.log).
- model-tests.log: PASS exit0, core26/document3/pdf6 (35); native cfg disabled by default as intended. Actual native combination RUNNING (native-initial.log).
- native-initial.log: PASS exit0, 1 native combination test over 12 PDFs. Actual A labels 1/1/2 -> 2 groups; actual B labels 1/1 -> heuristic 1 group despite 2 source frames (producer limitation, no label correction). B geometry/layout correct.
- beamer-movie-compile.log exit0; actual producer media inspection RUNNING (native-movie.log).
- native-movie.log: PASS exit0, 13 PDFs, actual Beamer movie scaled Start2/Duration8/Repeat extracted; automatic activation unresolved. Producer action inspection RUNNING (producer-inspect.log).
- producer-inspect.log: PASS exit0, actual Beamer uses Widget/AA/PO -> Movie action/T; unique same-page title targeting added from PDF1.7 tables8.44/8.59. Ambiguous title and action override regressions added. Targeted suite RUNNING (model-expanded.log).
- model-expanded.log: PASS exit0, 37 tests. Final targeted edge/crop tests added. Formatting and initial native clippy RUNNING (clippy-initial.log).
- clippy-initial.log: exit101, new code style diagnostics (let-chains, as_chunks, test iterators); corrected. Native clippy rerun RUNNING (clippy-final.log).
- clippy-final.log: PASS exit0, core/document/pdf/render pdfium all-targets warnings denied. Full native regression RUNNING (native-final.log).
- native-final.log: PASS exit0, 59 tests (core26/document5+native1/pdf9/render16+native1+missing1), zero ignored. Actual B left/right crops match equal-density full-page pixels within one BGRA byte; independent content present in both. Full workspace gate RUNNING (ci-final.log).
- ci-final.log: PASS exit0, fmt/workspace warnings-denied clippy/124 default tests/deny; existing duplicate-dependency warnings remain nonfatal. All-feature typecheck RUNNING (typecheck.log).
- typecheck.log: PASS exit0; core dependency audit RUNNING (core-tree.log).
- core-tree.log: PASS exit0; only thiserror/tracing production dependencies, proptest dev. Native missing-library negative verification RUNNING (native-missing.log).
- native-missing.log: expected exit101 with LibraryUnavailable; negative harness PASS. No tests skipped.
- Missing-runtime log verification correction: initial Select-String positional arguments emitted a nonterminating error; explicit -LiteralPath/-Pattern/-ErrorAction Stop audit now confirms LibraryUnavailable. Cargo native test actual exit101 remains preserved; corrected audit exit0.
- Final diff audit: disabling core.autocrlf caused CRLF lines across existing workspace to appear as trailing whitespace (exit2, diff-check-line-ending-override.log); normal project-configured git diff --check PASS exit0 (diff-check.log). No global line-ending rewrite was performed.
- I-1 through I-7 implementation complete; ready for independent check. ci124/native59/clippy/typecheck/core-tree/diff-check PASS; missing-native expected101 verified. research/verification.md, media-field-contract.md, fixture-provenance.json and artifact-audit.log are final evidence. Own TeX auxiliaries moved to ignored cache/beamer-build. All commands returned; no active command, commit/push/archive/journal/task switch.

## Independent check
- Applicability: 117 source and 24 evidence hashes plus 19 fixture/10 external hashes match the implementation checkpoint. Independent media regressions RUNNING (independent-media-before.log).
- independent-media-before.log: exit101 before compilation, sandbox build-lock AccessDenied. Existing-cache authorized retry RUNNING (independent-media-red.log).
- independent-media-red.log: expected exit101, both new media regressions failed (external source incorrectly recovered; unsupported action parameters undiagnosed). Local fixes applied; affected structure suite RUNNING (independent-media-green.log).
- independent-media-green.log: PASS exit0, 8 structure tests. Added a direct regression for preview clearing and atomic reconfiguration with a paused/Ready timer. Full workspace gate RUNNING (independent-ci.log).
- independent-ci.log: PASS exit0, fmt/clippy/tests/deny. Final native suite RUNNING (independent-native.log).
- independent-native.log: PASS exit0; all real fixtures and new regressions pass. Native clippy RUNNING (independent-native-clippy.log).
- independent-native-clippy.log: PASS exit0. All-feature typecheck RUNNING (independent-typecheck.log).
- independent-typecheck.log: PASS exit0. Fresh-process normal document native suite with nonexistent DLL RUNNING (independent-native-missing.log).
- independent-native-missing.log: expected cargo exit101/LibraryUnavailable; negative harness PASS exit0. All verification commands returned.

- Independent check complete: just ci127/native62/native clippy/all-feature typecheck/core-tree/diff-check PASS. Both media failures reproduced before local fixes. Review report records one pre-existing public API follow-up; task06 criteria remain satisfied. Native UI/playback/physical-monitor and GateB remain NOT_RUN. All commands returned; no commit/push/archive/journal/task switch.
- Root approved coordinated validation of the pre-existing crop setter, reusing CoreError::InvalidDocumentCrop and the existing rectangle predicate. Additional regression RUNNING (independent-crop-red.log); earlier final checkpoint superseded pending this repair.
- independent-crop-red.log: expected exit101, invalid NaN crop mutated state. Existing predicate/error reused in Result-returning setter; unique existing caller updated. Affected core regression RUNNING (independent-crop-green.log).
- independent-crop-green.log: PASS exit0, core28. Final post-setter workspace gate RUNNING (independent-ci-closeout.log).
- independent-ci-closeout.log: PASS exit0. Final post-setter native suite RUNNING (independent-native-closeout.log).
- independent-native-closeout.log: PASS exit0. Final native clippy RUNNING (independent-native-clippy-closeout.log).
- independent-native-clippy-closeout.log: PASS exit0. Final all-feature typecheck RUNNING (independent-typecheck-closeout.log).

- Final post-setter closeout: independent-typecheck-closeout.log PASS exit0; just ci128/native63/native clippy/typecheck all PASS. The previously deferred crop API issue is now fixed under root approval; no unresolved task06 finding. Original checkpoints remain historical; independent-check.md/manifest are final. All commands returned, no task switch or delivery side effect.
