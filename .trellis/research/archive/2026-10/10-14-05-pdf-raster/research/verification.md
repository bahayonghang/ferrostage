# Task05 implementation verification

Implementation is ready for the parent's independent check. All commands have returned; no worker, benchmark or Cargo session remains active. No commit, push, archival or journal operation was performed.

## Final checks

| Check | Actual result | Raw evidence |
| --- | --- | --- |
| Final `just ci` | Exit 0: fmt, default workspace clippy, **105 tests**, Rust dependency deny | `just-ci-final.log` |
| Final core/pdf/render native all-target tests, locked/offline | Exit 0: **38 tests** (21 core, 3 PDF, 12 render, 2 native integration), none ignored | `native-final-boundary.log` |
| Final core/pdf/render native all-target clippy, warnings denied | Exit 0 | `native-clippy-final.log` |
| `cargo deny --all-features check` | Exit 0, advisories/bans/licenses/sources all OK; existing duplicate syn/windows-sys warnings | `deny-all-features.log` |
| App `--features pdfium --locked --offline` check | Exit 0 | `app-feature-check.log` |
| Normal native fixture test with missing DLL | Expected exit **101**, `LibraryUnavailable`; no ignore or early success | `native-suite-missing.log` |
| Failed initialization then valid runtime, separate integration process | PASS in final native suite; nonexistent library and invalid font directory do not poison valid retry | `native-final-boundary.log` |
| Pinned runtime acquisition/verification script | Exit 0, cached artifact and every locked notice verified | `runtime-acquisition.log` |
| Fixture regeneration and final whitespace check | Results in `artifact-verification.log` | `artifact-verification.log` |

The final native and default suites cover the approved shared core frame change; task04 is not restarted. The feature graph did not change after the successful all-features deny/app feature checks. The missing-DLL normal-suite check is reused because initialization and that test are unchanged. The final native suite additionally exercises the fresh-process negative/recovery path.

The two final boundary regressions were demonstrated failing first (`boundary-before-fix.log`, exit101, 10 pass/2 fail), then fixed: L2/L3 account for Vec capacity via read-only `allocation_bytes()`, and narrow-page thumbnail bounding width rounds up to preserve 180px height. Final tests include both regressions.

## PoC2 measured result

`poc-2-timing.json` contains all **1,200 samples**: 20 warmups followed by 200 real renders for each of six cases. `submit_uncached` bypasses lookup and insertion, and every result checks `cache_hit == false` and matching request identity. The worker's elapsed interval includes native render, fresh BGRA allocation/white initialization, and moving validated pixels into the owned frame. Queue time, caller comparisons and GPU upload are outside it. Median is the mean of sorted samples 100/101; P95 is nearest-rank sample 190. No sample trimming or best-of-run selection was used.

| Case | Pixel dimensions | Median ms | P95 ms | Criterion |
| --- | --- | ---: | ---: | --- |
| Full page (page index8) | 3840×2160 | 12.47900 | 13.7167 | PASS ≤25ms |
| Full page with previous output retained | 3840×2160 | 12.88515 | 13.9641 | PASS ≤25ms |
| Wide full, same density (page index1) | 7680×2160 | 24.99215 | 26.9730 | Double-area comparison baseline |
| Wide left crop | 3840×2160 | 11.64345 | 13.0335 | PASS ≤25ms; median ratio **0.465884** ≤0.8 |
| Tall full, same density (page index2) | 3840×4320 | 24.37430 | 25.3200 | Double-area comparison baseline |
| Tall top crop | 3840×2160 | 11.14325 | 12.3451 | PASS ≤25ms; median ratio **0.457172** ≤0.8 |

The retained case holds the previous frame during the next render and checks unchanged deterministic pixels. The native suite separately keeps a red frame while rendering different pages and verifies the original stays red. Both cases now use identical fresh-Vec allocation; there is no scratch reuse that depends on UI/L2 releasing old frames. Full same-density baselines render twice as many pixels and are not 4K threshold cases.

This was a CPU AGG raster test on Windows 10.0.26300.0, Intel Core Ultra9 275HX (24 physical/logical cores). `poc-2-environment.json` preserves the parent's measured CPU/OS/GPU driver inventory with its timestamp. GPU upload/presentation is NOT_RUN and no GPU throughput claim follows from these results. WMI AdapterRAM is not a reliable dedicated VRAM measurement. There is no window or DPI scaling in these explicit pixel targets.

The benchmark and normal goldens use `font_directories=[]`. The PDF embeds original Type3 geometric E=mc² glyphs; its font source hash is `64377d9dffb51e281c02053c5e4c65c3c70c242478b939e3271a4e17ec536430`. No host-font substitution is required for this fixture. A separate recovery test supplies the existing `fixtures/pdf` directory to exercise nonempty font-path configuration; it does not prove any particular fallback font was selected. Real missing-font substitution quality remains UNVERIFIED.

PDF SHA256: `25a7932f46ee8fcd4866011921c574b4f5b343af05b793bc397b7e93ed0df013`. DLL SHA256: `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`. Timing JSON SHA256: `e77a54a7325db0e91e7a07a9ad41feeaa6a206c15fcac86579c97170dc1bb9f2`. The measured release executable remains local at `target/release/examples/poc2.exe`, SHA256 `b50efbe7a82e04f3df47be39df4d4eca453001b507018682adb45a6f6679e464`.

Timing reuse boundary: the final native raster file is unchanged since before that executable's build (native.rs mtime15:57:23Z; executable15:58:23Z, 2026-10-14). Later changes are the read-only capacity getter, cache accounting, thumbnail constructor rounding, regression tests, an equivalent collapsed benchmark assertion and formatting/docs. The core frame constructor and uncached Audience render path are unchanged. Final source hashes are captured at closeout; an original whole-tree hash snapshot was not taken at benchmark start, so this is an explicit reviewed reuse decision, not a claim that the entire current tree was the measured build.

Earlier failures remain available: initial export/Arc-slice copy full P95 37.292ms; borrowed bitmap but Vec→Arc-slice copy 27.216ms; unique-scratch warm-only result 6.572ms; scratch with previous frame retained **25.602ms FAIL**. Their raw JSONs and logs are retained. The final safe Arc<Vec> change removed the full-frame slice copy and scratch dependency. It passed in the first full six-case run for that implementation.

## Functional coverage and implementation boundary

Native goldens validate owned metadata/page labels; invalid bytes; solid red channel order and packed stride; transparent red over white; independent quadrant expectations for intrinsic rotations and offset CropBox; nonblack embedded glyph pixels; left/right/top/bottom crops against FULL at identical pixel density (exclude two pixels at edges, per-channel tolerance1); 180px thumbnails; and failed prepare/commit/retire isolation. Fixture data is self-authored and deterministic. It is a fixed correctness/performance corpus, not a guarantee for arbitrary PDFs.

Controlled fake tests verify all request identity, next-dispatch priority, ten queued P4 cancellations, in-flight stale suppression, old generations, active work during candidate batches, per-request candidate priority/epoch, ready-token invalidation and batch nonce isolation. L2/L3 share each tier's budget across generations; cache hits have no fake render duration. Cache retention is bounded by allocated pixel capacity; consumer-held clones and queued owned requests are outside cache accounting. L1 returns eviction keys for the future UI owner to release GPU resources.

Runtime API/notice review is in `native-api-audit.md` and `docs/pdfium-runtime.json`. Precompiled third-party DLL license review is separate from Rust `cargo deny`. Runtime paths are explicit; successful configuration is process-sticky, failed pre-initialization is retryable, and concurrent sessions are rejected. No project unsafe or RGBA export was introduced. Native form overlays remain unsupported by the matrix/clip path.

Files owned by task05: PDF metadata/raster/native modules and feature manifest; render protocol/cache/scheduler/worker/fake/native tests/PoC2; approved core frame storage and tests; minimal app feature forwarding, root Cargo/lock/just/CI; fixture generator/PDF/manifest/README; runtime acquisition script/pin manifest; ADR002/packaging; task05 records; relevant backend spec additions. Other dirty settings/platform/lifecycle/task files belong to parallel parent work and were preserved.

Hosted Ubuntu/Windows workflow execution is **NOT_RUN** locally; the required separate native job has been configured and its relevant commands passed here. Non-Windows native runtime, real font fallback quality, interactive forms, GPU upload/presentation and physical monitor actions are not validated by this task. No remaining task05 code or local check is pending; independent review is pending. Raw `.log` evidence and the executable/runtime caches are local ignored files; `verification-manifest.json` records their hashes for this checkout and does not imply they were committed.
