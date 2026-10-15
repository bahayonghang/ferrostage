# Implement — 03

1. **I-1 安全接口研究门**：基于父runtime证据核安全enum/device identity/event wrapper、exact版本/许可；若缺接口提出最小ADR/lint方案等待政策决定，不靠allow绕过forbid。
2. **I-2 类型**：core补MonitorId，platform Info/source/warnings/Optional topology/diff；复用02 MonitorRole。编译与core依赖树确认无OS。
3. **I-3 纯解析**：实现零/一/二/三屏，无保存/已有记录/显式None，缺Audience/Presenter，同ID重复binding/重复roles/无primary角色互斥/ID碰撞、面积tie稳定排序；property只断言真实不变量（最多1 Audience、来源合法）。
4. **I-4 平台枚举**：安全适配输出标准单位与身份级别，测试错误/无EDID/重复名；不输出伪强身份。
5. **I-5 事件/diff**：独立安全OS事件源（不足则200ms安全枚举轮询，不等待07），watch重新枚举、合并、防抖、取消订阅，fake输入测试快速插拔；不在此建GPUI窗口。
6. **I-6 实机门**：同型号/不同DPI三屏，连续枚举/换接口/旋转/缺席/重复几何，记录机器/identity等级与恢复，不匹配时确认无Audience接管；事件≤1s。报告只实施后创建。
7. **I-7 评审**：zero/warning/唯一role来源、safe策略、07 mapping输入完整，运行Gate A topology部分与命令。

命令：cargo test -p ferrostage-platform -p ferrostage-core；cargo tree -p ferrostage-core；cargo clippy -p ferrostage-platform --all-targets -- -D warnings；just ci。
回滚：新OS适配失败保留纯类型/解析，不能回退不稳定索引；共享类型回滚需同步消费者。无实机仅可通过纯契约部分，不标03完成。

## Execution — 2026-10-14T14:57:25.009267+00:00

- I-1 IN_PROGRESS: reviewed injected task/spec context and parent shared §3. Cargo info fetched exact winsafe 0.0.29 (MIT, MSRV 1.87) and display-info 0.5.9 (Apache-2.0). Initial sandbox registry lookup failed; escalated read-only fetch succeeded. No product edits yet.
- Selected strategy sent to coordinator: Windows-only safe device-interface enumeration plus cross-checked display-info geometry/scale; explicit 200 ms polling, typed unavailable event; no policy relaxation or windows. Native scale is process-dependent and must be revalidated by 07.
- Next: save API evidence/design, implement pure IDs/resolution, adapter and watcher; run portable contracts and native static inventory. Physical replug/rotation/same-model tests remain NOT_RUN.

- I-1 complete: coordinator accepted exact safe-wrapper combination and synchronized parent/07/09 contracts. I-2 through I-5 implementation drafted, including regression/property tests and fail-closed adapter validation.
- Verification IN_PROGRESS: formatting, targeted core/platform contract suite and native Windows compile; dependency resolution/build results pending. No PASS recorded before command completion.

- Targeted Windows MSVC compile and tests PASS: core 21 + platform 21 (including adapter validation, pure property tests and worker cancellation), exit 0. Evidence: research/targeted-tests.log. Initial sandbox registry fetch failed before tests (research/targeted-sandbox.log), escalated retry succeeded.
- Native static inventory IN_PROGRESS: new read-only monitor_probe example will enumerate ten times, print owned identity/physical geometry/source, verify initial watcher event and joined cancellation. No display changes or windows.

- Native static inventory PASS (10 stable three-screen samples, initial watcher snapshot and explicit cancellation, exit 0); research/native-static-probe.log. First probe example compilation caught a pattern-guard move, fixed to compare references; retained native-probe-compile-first.log. Physical replug/port changes/same-model identity/rotation mutation and ≤1 s event timing remain NOT_RUN; process DPI context unmeasured.
- Added Gate A integration tests through actual settings TOML parsing into MonitorId/MonitorRole tuples (settings is a dev-dependency only). Gate A suite and just ci IN_PROGRESS; final results pending.

- Gate A contract suite PASS: 82 tests (core21 + platform21 + settings-topology2 + settings38), exit0; production core tree only thiserror/tracing, no IO/native dependency. Platform clippy all-targets -D warnings PASS. Added optional bounded `monitor_probe --watch` as a developer-only observation harness for remaining manual acceptance. Full just ci IN_PROGRESS at 2026-10-14T15:08:36.080120+00:00.

- Full CI attempt 1 FAILED at cargo deny: RUSTSEC-2025-0057 (`display-info -> fxhash 0.2.1`, no safe upgrade); fmt/clippy/92 tests PASS, license/ban/source PASS. Kept full log as research/just-ci-first-deny-failure.log. No advisory ignore or policy change.
- I-1 revisited: winsafe `gdi` includes safe desktop DC RAII, per-monitor callback DC, EnumDisplaySettings and GetDeviceCaps. Replacing display-info entirely; scale source becomes explicit GdiDeviceCapsRatio (current-context capability ratio, not effective DPI). Coordinator informed before adapter changes. New native probe and full CI required; previous static evidence describes the superseded adapter only.

- Final winsafe-only native static probe PASS: 10 stable three-screen samples, every cap/DEVMODE-width consistency check accepted, initial watcher snapshot and stop/join PASS; exit0, research/native-static-probe.log. Initial wrapper compilation found a missing Handle import and enum-vs-bitflag comparison; fixed, failure retained as native-winsafe-first-compile.log. Final source is GdiDeviceCapsRatio and all display-info/fxhash dependency entries were removed.
- Gate A / full CI rerun IN_PROGRESS for the final adapter; prior failed CI and superseded adapter probe are retained as history, not final evidence. User cannot operate physical displays this turn; manual acceptance remains NOT_RUN.

## Current recovery point — 2026-10-14T15:18:41.324493+00:00

- I-1/I-2/I-3/I-4/I-5 implemented and verified for the final single winsafe adapter. No unsafe/ignore/policy relaxation; removed failed dependency candidate.
- I-6 PARTIAL: native-static-probe.log PASS (10 three-screen samples + initial event + cancellation). Physical replug/port-change/same-model/identity absence/DPI or rotation mutation/≤1s timing NOT_RUN; user unavailable for these operations. Exact future commands in research/stability.md.
- I-7 automated PASS: final just ci exit0,92tests + fmt/clippy/deny; Gate A core/settings/platform integration subset82 passed within that final run. Source/manifest SHA in verified-code-sha256.json; API/license evidence in api-evidence.md. Previous failed/superseded logs retained distinctly.
- No commands remain running. Next: independent Trellis check, then coordinator can advance according to parent dependencies while retaining manual gates. Files remain uncommitted; no archive or journal action.
- Files: core display.rs/lib.rs; platform Cargo.toml/lib.rs plus monitor/topology/watch/windows modules, module tests, settings_topology integration test, monitor_probe example; root Cargo.toml/lock; ADR007, feature-parity monitor row, backend directory spec, this task planning/evidence. Existing settings/lifecycle/Claude edits preserved.

- Final documentation check: task.py validate exit0 (implement17/check13); git diff --check exit0 (line-ending notices only). All62 source/manifest hashes still match final CI evidence; no product edits after CI.

## Independent check recovery point — 2026-10-14

- Independent review found one local native validation defect: winsafe 0.0.29 `has(mask)` means any overlapping bit, so the combined required DEVMODE fields were not all enforced. Extracted the existing mode conversion into a private testable helper; regression first FAILED on missing POSITION (exit101), then PASS after replacing the predicate with full mask equality. Each of POSITION, DISPLAYORIENTATION, PELSWIDTH and PELSHEIGHT is independently required. Other adapter `has` calls use single bits and are correct. Backend spec records this wrapper contract.
- First regression attempt could not open the existing Cargo build lock in the sandbox; retained independent-regression-sandbox.log. The approved cache-access retry executed the failing regression, retained as independent-regression-before.log; independent-regression-after.log records the corrected PASS. No policy, dependency or public interface changed.
- Fresh final checks completed: independent-just-ci.log exit0 (fmt, workspace clippy, 93 tests, deny); Gate A subset is now core21 + settings38 + platform22 + integration2 = 83. independent-typecheck.log exit0. independent-native-static-probe.log exit0 (ten stable three-screen samples, all four required native fields checked, first Changed and stop/join).
- These checks supersede the earlier 92-test/82-subset evidence for current code. Pre-review hashes matched all62 source/manifest files and all8 dependency source files; new current fingerprints and detailed review are saved in research/independent-code-sha256.json and independent-check.md. Only windows.rs and windows/tests.rs differ from the earlier code set. No commands remain running.
- I-7 independent review is technically PASS after the fix; I-6 remains PARTIAL. Physical unplug/replug latency, same-model/port-change/missing-identity hardware, DPI/rotation mutation and actual GPUI placement remain NOT_RUN. Task stays in_progress; no full03, AC-2/3 or Gate B completion claim, commit, archive or journal action.
- Final task-context validation PASS (implement17/check14), git diff --check exit0 (line-ending notices only), and all62 current source/manifest hashes plus the rebuilt probe hash matched. Independent report and current evidence are saved; no commands running.
