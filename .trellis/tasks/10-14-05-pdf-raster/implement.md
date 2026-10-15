# Implement — 05

## Current recovery status — independent check passed; technical delivery ready

I-1 through I-7 implementation and independent check are complete. The reviewer reproduced and fixed cross-worker CandidateReady reuse with a private shared worker identity, then expanded lifecycle/cache/native assertions. Final default `just ci`: 109 PASS plus fmt/clippy/deny; native suite: 42 PASS; native clippy and workspace all-features type-check: PASS. The normal native suite still fails with missing DLL (expected exit101). See `research/independent-check.md` and `research/independent-check-manifest.json` for current review evidence and hashes; `verification.md` / `verification-manifest.json` preserve the earlier implementation checkpoint. Final 4K P95 12.3451–13.9641ms and crop/FULL median ratios 0.465884/0.457172 were independently recomputed without rerunning the benchmark. No process remains running. No commit/push/archive/journal has occurred. Parent owns task progression and outstanding native UI/manual gates.

1. **I-1 native前置研究门**：核0.9.4 safe API/feature、binary7881来源/checksum/license、owned bytes寿命、matrix/clip限制；ADR-002/packaging纠正下载假设并评审分层CI。
2. **I-2 纯协议**：定义owned metadata、request/result完整键、active/candidate metadata-first Prepare→06定位→PreRender→ready token/ack、scheduler/cache预算；fake worker测试priority出队/不可抢占/epoch/重代/候选不被误丢弃。
3. **I-3 worker/raster**：在唯一线程创建PDFium及active/candidate，安全load/render/label/close；验证wrong bytes/空/缺库typed错误，接口不泄露nativehandle。
4. **I-4 金标**：独立制作红色/透明/字体/旋转/CropBox/32:9/上下夹具（不调用产品编译器）；核stride/channel/同密度crop对照/标签，有固定环境与容差，不能用两张全黑图通过。
5. **I-5 native CI**：定义pdfium feature转发、获取经checksum DLL、独立job必须跑fixture；缺库负测试必须失败。默认pure suite和native suite结果分开，no ignore。
6. **I-6 PoC2性能**：按design协议20warm/200samples、4K worker时间与crop比值，保存环境/原始数据和P95；未达目标记录阻塞/具体评审，不用明显更快结论。
7. **I-7 接口/质量评审**：07 probe、06labels、08frame、10Prepare/Commit/Retire用例，cache回收/错误/日志/许可证评审；run下列门，PoC2与GPU状态准确区分。

拟新增feature后命令：cargo test -p ferrostage-pdf -p ferrostage-render；cargo test -p ferrostage-pdf -p ferrostage-render --features pdfium；cargo clippy -p ferrostage-pdf -p ferrostage-render --all-targets --features pdfium -- -D warnings；just ci；cargo deny check。
PDFium配置按I-1确定显式runtime路径；feature名在本任务创建，当前尚不存在。启用native且缺库失败，不记跳过为成功。
回滚：I-3加载失败Abort保旧worker；I-5feature/CI/manifest/packaging同范围回滚；不写任何用户PDF。

## Recovery point — 2026-10-14 I-1 in progress

- Loaded task manifests, PRD/design, role context and backend guides; preserving all existing unrelated changes. Parent approved the concrete owned metadata/request/candidate protocol and synchronized consumers.
- Independently verified cached archive SHA256 `73cc0de638ac2095e7445bf56a38200a5b7c7ca0e9f4ba144598f2457377ac08` and DLL SHA256 `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`; VERSION 151.0.7881.0. No native load yet.
- Added optional exact `pdfium-render =0.9.4`, only `pdfium_7881` enabled, and pdf/render/app feature forwarding. Corrected the automatic-download comment. Cargo.lock updated by fetch.
- Sandboxed `cargo fetch` failed DNS (exit 1); approved network retry completed (exit 0). Exact crate source is now local. Investigating safe matrix/clip/raw/initialization APIs before raster implementation. Native fixtures/performance and all quality gates remain NOT_RUN.

## Recovery point — I-2/I-3 implemented, I-4 first native result

- Native adapter, owned protocol, cache/L1 ledger, scheduler and worker are now present. Core BgraFrame reused. Exact safe source API verified; `set_reverse_byte_order(false)` is mandatory. PDFium binding is process-global/sticky; mutex excludes concurrent sessions, configuration changes after initialization are rejected. Failed binding does not set the configuration.
- First compile hit sandbox build-lock access denied; approved retry reached source checks. Corrected native page index to i32 and an owned-message pattern guard. No unsafe/lint relaxation.
- `cargo test -p ferrostage-render --features pdfium --test native --offline -- --nocapture`: exit 0, 1 integration test PASS (0.17s), using the independently hashed cached DLL and self-authored deterministic 8-page fixture. Covered BGRA red, white alpha composition, labels, 90/180/270 rotations, offset CropBox, same-density left/right/top/bottom crop comparisons (2px excluded, per-channel <=1), candidate commit/retire and failed prepare isolation.
- This is functional evidence, not performance acceptance. Extending independent pixel expectations, controlled fake worker races and lifecycle tests next. Full pure/native/clippy/deny gates and 20+200 PoC2 remain NOT_RUN.

I-4 final native fixture rerun + missing-library process test starting; raw log research/native-tests.log. I-6 release PoC2 follows after successful native rerun; all outputs capture exit codes.

Native full suite exit0:13 tests (3pdf+8render+2 native), raw research/native-tests.log. Added 9th self-authored 16:9 art/text/formula page for nontrivial full4K timing; final native rerun after fixture update pending. I-6 PoC2 release build/run starting, raw research/poc2-run.log and poc-2-timing.json.

PoC2 first launch exit1: Pdf(LibraryUnavailable) with relative slash path; no timing samples emitted. Native tests used absolute DLL path. Added canonicalization of explicit DLL/font paths before binding to avoid Windows library-search reinterpretation. Retrying release benchmark, prior raw failure preserved poc2-run.log; retry output poc2-run-retry.log.

PoC2 first completed run exit0, formal performance FAIL: full4K P95 37.292ms; wide crop P95 34.631ms; tall crop P95 41.668ms (threshold25). All raw200 samples saved poc-2-before-optimization.json. Investigating safe PdfBitmap::from_bytes against owned mutable BGRA backing to remove redundant bitmap→Vec→Arc copies, no unsafe/threshold relaxation.

Owned-buffer golden rerun exit101: pixels passed but sticky-config assertion expected ConfigChanged for nonexistent path; new canonicalization correctly returns LibraryUnavailable before comparison. Test now changes to an existing font directory to isolate sticky-config behavior. Retrying native tests before performance.

Owned-buffer native golden retry exit0. Optimized release PoC2 beginning with unchanged 20warm/200sample protocol; report poc-2-timing.json and raw poc2-optimized.log.

Owned-buffer optimized run exit0 but performance still FAIL full4K P9527.216ms/tallcrop27.957ms, widecrop24.842ms PASS. Kept raw poc-2-owned-buffer.json. Added one bounded scratch BGRA allocation reused only at Arc strong_count1; retained consumer frames are never mutated. Next native safety regression + final timing.

Safe scratch run exit0: full4K P956.572ms, widecrop7.080ms, tallcrop6.802ms; crop/FULL median ratios below0.8. 1000 samples saved poc-2-timing.json. I-5 CI acquisition/native/negative jobs implemented; I-7 native clippy starting, log native-clippy.log.

Native clippy initial exit101: manual_inspect; changed side-effect-only map to inspect. Parent requested retained-previous-frame diagnostic to qualify warm unique-buffer results. Added separate same20+200 full4K case retaining previous frame throughout next render; added native cross-page red-frame immutability assertion. Release diagnostic starting; old warm report preserved poc-2-warm-scratch.json.

Retained-frame diagnostic exit0 but P95 25.602ms FAIL vs25, median23.807ms. Warm alone is not universal production acceptance. Parent reviewing minimal owned BgraFrame storage extension to remove mandatory Vec→Arc slice copy; no shared-core change before coordination. Source review/native suite negative and acquisition checks continue independently.

Parent approved shared-frame extension, synchronized04/05/08/11+parent. BgraFrame now owns private Arc<Vec<u8>>, constructor takes Vec, read-only getter slice; tests prove original allocation pointer/clone sharing and Debug omits pixels. Native directly moves completed borrowed-bitmap Vec into frame; removed scratch entirely. Current callsites updated. Core/native quality check and identical20+200/retained remeasurement next.

Arc<Vec> pure core/pdf/render32 tests PASS and native golden1PASS. Starting final samefixture20+200 full/crops+retained performance; no scratch path exists.

Final unified Vec-frame PoC2 exit0: full4K median12.479/P9513.717ms; retainedprevious median12.885/P9513.964ms; widecrop P9513.034ms; tallcrop P9512.345ms. All4Kcases <=25; same-densitywide/tall median crop/FULL ratios~0.466/~0.457<=0.8. Raw1200samples in poc-2-timing.json and poc2-vec-frame.log. No scratch dependence. Native full+clippy starting.

Final native clippy exit101: test slice chunk API lint and benchmark collapsible-if. Applying equivalent test/format changes; runtime raster untouched.

I-7 native clippy exit0, native full15tests exit0, separate normal suite missingDLL exit101 with LibraryUnavailable (expected negativePASS). Raw native-clippy-pass.log/native-final.log/native-suite-missing.log. Starting justci defaultworkspace fmt/clippy/test/deny, log just-ci.log.

just ci defaultworkspace exit0 (fmt/clippy/tests/deny), log just-ci.log; cargo-deny existing duplicate syn/windows-sys warnings only, advisories/bans/licenses/sources allok. Additional all-features deny + app feature compile starting to cover optional runtime graph/forwarding.

I-7 boundary regressions reproduced before repair: research/boundary-before-fix.log exit101, 10pass/2fail (spare Vec capacity undercount; narrow thumbnail height167 instead180). Parent approved read-only allocation_bytes and ceil thumbnail bounding width. Repairs applied and backend specs/design synchronized. all-features deny and app feature compile both exit0 (deny-all-features.log/app-feature-check.log). Starting final core/pdf/render native tests, clippy and final justci on repaired code; performance raster/constructor algorithm unchanged.

Final repaired native suite exit0: core21 + pdf3 + render12 + native2 = 38 tests, no ignored tests. Native-final-boundary.log captures output and exit. Starting final native clippy (native-clippy-final.log).

Final native clippy exit0 (native-clippy-final.log). Starting final default workspace just ci on repaired code (just-ci-final.log).

Final default just ci exit0, 105 tests; all native commands returned. Artifact regeneration byte-identical and git diff --check exit0 (artifact-verification.log; line-ending advisory warnings only). Modified files: core/frame.rs (parent-approved storage/getter/tests); pdf and render modules/manifests/tests/example; app feature forwarding; root Cargo/lock/just/CI; original fixtures; runtime acquisition/pin manifest; ADR002/packaging; backend directory/quality spec additions; task05 records/research. Unrelated pre-existing changes preserved. Final report and SHA manifest prepared for independent check. Native runtime/constructor/uncached Audience algorithm unchanged by final capacity/thumbnail boundary repair, so the first successful six-case Vec-frame timing is reused with explicit limitations, not rerun for sample selection.

## Independent check closeout — 2026-10-14

- Confirmed all 88 implementation source hashes, 34 evidence hashes and 3 local artifact hashes before edits, plus all five exact wrapper-source and 16 runtime-notice hashes. Recomputed all 1,200 samples, median/P95 and crop ratios successfully.
- New token regression failed first: `independent-token-before.log`, exit101, because the stopped worker's token committed another worker's same-numbered candidate. Parent approved minimal private identity repair. `independent-token-after.log` passed; final contract tests also cover epoch/reprepare/abort invalidation, candidate failure preserving active, and complete cache/request identity. New tests live in a separate module to keep files below the backend guideline.
- Final source checks after that test split: `independent-ci-closeout.log` exit0/109tests; `independent-native-closeout.log` exit0/42tests; `independent-native-clippy-closeout.log` and `independent-typecheck-closeout.log` exit0. `independent-native-missing.log` records required native exit101/LibraryUnavailable. Earlier sandbox build-lock denial is retained in `independent-ci-before.log`; approved use of the existing build cache resolved it.
- Technical delivery ready, no unresolved task05 code finding. Hosted CI, actual font fallback selection, non-Windows native runtime and GPU/manual acceptance remain NOT_RUN/UNVERIFIED. Task10 owns the main-thread publication/worker-ack transaction. No benchmark rerun or delivery side effect; all commands returned.
