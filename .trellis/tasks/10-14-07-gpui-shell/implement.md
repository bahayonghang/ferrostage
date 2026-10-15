# Implement — 07

## Resume 2026-10-14 America/New_York (2026-10-15 UTC)

Existing task07 reactivated; baseline `3ac1ec2`, clean. `research/resume-v6-identity-20261015.json` verifies94 crate files and V6 executable/DLL/PDF unchanged; only the recorded DESIGN.md status-line difference among602 inputs. Five retained manifests match. Reuse applicable V6 evidence. Root continues pending software-native matrix and controls all windows; implement agent owns new retirement diagnosis research only until source freeze is released. User skips physical hardware operations this turn; they remain NOT_RUN. Gate B remains incomplete and08–11 gated. New changes stay uncommitted.

1. **I-1 display研究门**：核pin Windows backend、DisplayId/uuid/bounds单位、fullscreen参数、DPI/事件与focus/keystroke；生成确有证据的gpui-display-api结论，择一允许路线。无法安全映射则保留具体阻塞。
2. **I-2 feature编译**：ui链接gpui、app转发gpui/pdfium，更新ADR-001/CI；WindowsMSVC native compile通过，core依赖树无GPUI。
3. **I-3 导入/讲者**：启动config/topology、CLI/dialog/drop/最近文件import调用05metadata；Load→Ready占位，无观众计时；坏文件在讲者提示、旧状态保留。
4. **I-4 窗口事务**：实现create-Curtain/Start-Live/Stop-close、失效/零屏/单屏显式模式；注入创建失败测试不留Live/计时窗口。
5. **I-5 动作/角色**：全局routing、Escape模态/preview优先消费、弹UI讲者activate、冲突按键禁用、Swap权威binding事务、save dirty提示、03事件拔屏停止；无独立swap flag。
6. **I-6 Gate B实机**：I-8后由最小分配UI选择B应用/A播放/C不使用，再以B应用/C播放复测；开始/结束/拔线、混DPI/旋转/单屏/零屏；占位画面即可，焦点/Identify在07验收。已有02配置驱动的历史结果保留，但不替代新UI验收。记录环境/操作/截图/代码版本。
7. **I-7 评审**：only-snapshot view、read-only导入、错误/焦点、featureCI与Gate B；不能把占位称真实PDF或设置UI完成。
8. **I-8 显示器分配修订（当前下一实施步骤，编号保留）**：先核03完整拓扑与07事务；替换工具栏Swap为显示器设置，构建数量/编号/两项选择/Apply/Cancel/Identify最小面板，保留旧快捷键兼容；用实际GPUI回归验证草稿、冲突、失效、焦点、运行时回滚与保存失败，随后独立check和真实三屏UI验收。I-8在Gate B之前完成，09复用，不以09为前置。

拟feature创建后命令：cargo build -p ferrostage-app --features gpui,pdfium；cargo clippy -p ferrostage-app -p ferrostage-ui --all-targets --features gpui -- -D warnings；cargo test -p ferrostage-core -p ferrostage-platform；just ci。
WindowsMSVC job编译门、纯suite、GateB实机分别记录。回滚Start失败清新窗口不计时；角色应用失败回滚窗口/bindings；feature路线回滚app/ui/CI/ADR同批。

## 主题执行补充（2026-10-14）

在I-2/I-3/I-7中先核DESIGN.md与07主题层所有权，实施对应token/布局/设置消费者后保存原生截图、对比度结果与键盘/状态检查。无运行窗口时记NOT_RUN，保持原有Gate依赖。

## 本轮执行边界（2026-10-14）

02/03/04/05技术交付及Gate A已就绪；按用户串行约束，在06独立check结束后才start07。用户明确本轮无法配合物理拔插/换接口，I-6相关项目必须保留NOT_RUN；可完成当前三屏上的真实窗口定位、配置角色切换、焦点/快捷键、开始/结束及视觉验证，不用模拟事件替代拔线时限验收。Gate B未完整通过仍阻塞08真实双窗集成。单屏/零屏等真实环境不可用时分别记实际自动测试与未执行实机项。

主会话统一使用已初始化的computer-use/sky控制原生应用，实施代理负责代码/编译/纯测试，避免两个控制器同时操作桌面。PDF和config测试使用仓库自建fixture及独立测试配置，保留用户已有配置。实机步骤及截图须在实际程序可运行后记录，不先勾选验收。

## Implementation checkpoint — 2026-10-14

- I-1: exact tagged Windows display source read. Candidate mapping uses the public UUID generated from the zero-filled 32-u16 GDI connection buffer (big-endian bytes, UUIDv5 DNS); require unique IDs, names and UUIDs plus complete set equality. No platform API expansion or geometry/DPI fallback is planned. Native verification is pending.
- I-2: isolated route spike at ignored `cache/gpui-route-spike`, exact Zed tag `v1.23.3`, using `gpui` plus `gpui_platform`. Product manifests remain unchanged pending route review.
- `cargo build --manifest-path cache/gpui-route-spike/Cargo.toml`, approved existing-cache/network retry, session `28920`, completed exit0 in 3m20s. Raw output: `research/route-spike-build.log`; status: `research/route-spike-status.txt`. Initial sandbox cache-write failure is retained in `research/route-spike-build-initial.log` (exit101). This is an isolated native compile PASS, with no window launch.
- Tagged route policy audit completed exit5 (`research/route-spike-deny-final.log`) after correcting the isolated spike's own license metadata. Unchanged policy rejects arrayref BSD-2-Clause, libbz2-rs-sys bzip2-1.0.6 and macOS build dependency cbindgen MPL-2.0; advisories flag paste, rustybuzz and ttf-parser as unmaintained. These are not exploit findings. No policy waiver, product dependency, pin or UI implementation change was made.
- Historical next step was evaluation of the second authorized route; it is now complete as recorded below. Both production I-2 integration and I-3 onward remain gated. Root owns native UI interaction and parent state updates. Physical unplug/port changes remain NOT_RUN.
- Alternative source confirmed: gpui-kit0.7.3 uses the exact gpui-pre0.3.9 family; gpui-component0.7.3 directly depends on paste1 and Windows resvg0.45.1. The pre crate records upstream revision `93d745668805029476e6d7fbb19652116fed51de`, distinct from tagged route `96a1b1b5`. Isolated alternative `cargo build --manifest-path cache/gpui-kit-route-spike/Cargo.toml` was RUNNING in session `94324` at the prior checkpoint and has now completed exit0 in 1m44s; raw log `research/kit-route-build.log`. No product manifests or policy changed.

## Final implementation handoff — blocked at I-2

- Route B policy audit completed exit5 (`research/kit-route-deny.log`, session94991). Route C removes all kit/base/component dependencies and audits only exact gpui-pre/gpui-pre-platform0.3.9 with defaults disabled; it also completed exit5 (`research/pre-minimal-deny.log`, session17355). No command remains running.
- The minimal family still requires paste, rustybuzz and ttf-parser (INFO Unmaintained; no patched versions), arrayref BSD-2-Clause, and full-graph Apple build dependency cbindgen MPL-2.0. Optional component removal does not resolve these. Exact required edges, per-route findings and Windows versus full-graph attribution are in [dependency-gate.md](research/dependency-gate.md).
- I-1 source audit is recorded in [gpui-display-api.md](research/gpui-display-api.md), including the strict UUID bridge, silent primary-display fallback requiring post-create verification, key syntax/focus and replacement-window migration. No mapping/native runtime test or policy-compliant route selection is claimed.
- Native compile PASS applies only to isolated A/B spikes, never to the product or Gate B. Their exact manifests/locks/main sources are retained under `research/route-fixtures`; original cache paths, hashes, source evidence, logs and binaries are in `research/evidence-manifest.json`. Product preservation is checked against06's final117-file manifest in `research/product-input-comparison.json`.
- Root requested an independent check of this blocker before a concrete Proposed ADR and user decision. No pin/deny change, policy waiver, maintenance fork, product dependency, feature/CI wiring, shell source or theme source was made. Root owns parent execution-state updates.
- Resume after the approved dependency decision: finish I-2 product integration/ADR/CI; implement I-3 safe worker import + presenter placeholder; I-4 Curtain lifecycle and replacement rollback; I-5 actions/focus/settings-authority/topology; shared DESIGN.md theme; then product default/native checks and I-6 real-window evidence. Preserve the existing07 scope: no08 workbench,09 settings UI,10 reload or11 playback implementation.
- Product native build/clippy, `just ci`, core dependency tree, shell transaction regressions and actual UI screenshots/actions are NOT_RUN for07. Prior06 tests were not rerun without product changes and are not relabeled as07 evidence. Physical unplug/port changes remain NOT_RUN per the user; current three-screen setup is not real single/zero-screen evidence. Gate B remains incomplete.

## Independent check — 2026-10-14

- [Independent review](research/independent-check.md) confirms the I-2 dependency blocker. Both isolated A/B build exits0 and all three original deny exits5 are supported by matching fixtures/logs/binaries; the minimal C full audit was independently rerun locked/offline and returned5. No product route, pin, policy exception or maintenance fork was approved.
- The initial99 task07 evidence records all matched, including52 source/license/advisory files. Task06's final117 inputs,43 evidence files and10 external inputs remain unchanged; all86 current crate files are covered, and all92 classified product inputs match. Prior128 default/63 native test evidence remains applicable and was not rerun or relabeled as07 delivery.
- Incremental unchanged-workspace fmt, warnings-denied clippy and all-targets/all-features type check returned0. Initial cache-lock failures and approved retries are preserved. These checks do not compile a GPUI product feature, which does not yet exist; `just ci`, task07 native feature checks and windows remain NOT_RUN.
- The source review corrected two timing omissions in `gpui-display-api.md`: activation is asynchronous while file-dialog ownership is captured synchronously; fullscreen is asynchronous and its flag is set before a native placement call whose failure is only logged. Safe public observation APIs are identified, but the actual focus/placement confirmation transaction, timeout/cancellation and native verification remain implementation prerequisites. Root synchronized the parent/07 planning boundaries and removed the obsolete geometry identity fallback.
- Original display-report/implement bytes and the original evidence manifest are preserved. Recompute with `python .trellis/tasks/10-14-07-gpui-shell/research/independent-hash-audit.py`; final hashes and expected document deltas are in `research/independent-hash-audit.json`. No product source, dependency, CI, theme, policy or task pointer changed. All reviewer commands completed; no windows or background review command remain. Gate B/C and physical NOT_RUN boundaries are unchanged.

## ADR-008 maintenance evaluation authorized — 2026-10-14

User approved the bounded maintenance evaluation while retaining gates. Resume I-2 through research/maintenance-plan.md: prepare an isolated exact-source dependency/API candidate, regressions, full unchanged audit, native compile and reproducible patch/lock evidence, then independent check. Production pin/policy/CI/UI remain unchanged pending the resulting candidate and any explicit license decision. Prior blocked reports remain historical evidence; do not repeat completed01–06. Physical display operations remain NOT_RUN. Root owns parent/ADR updates; implementer owns maintenance artifacts and isolated candidate only.

Maintenance step1: source boundary identified as exact-tag Apache GPUI SVG/font lookup plus local dependency-only patches to MIT pulp0.22.3 and MIT/Apache metal0.33.0 for the real maintained pastey package. The full graph includes Apple's metal→paste edge as well as Windows image→exr→pulp→paste. Preparing primary-source0.48 API inputs in ignored cache/gpui-maintenance-candidate; progress and actual exits are recorded separately in research/maintenance-status.txt. No original spike or shared Cargo source is edited.

Maintenance step2: first candidate native build exit0 (1m17s); real image regressions4/4 and GPUI SVG/font regressions14/14 pass. Stage1 complete deny exit5 confirms paste/rustybuzz resolved in the audited graph, but WGPU/Web→cosmic-text0.19→fontdb0.23 retains ttf-parser. Root approved a further isolated cosmic-text fontdb0.24 dependency-only migration; scope and the upstream vendored metadata subset are documented in maintenance-plan.md. Existing three license failures remain unchanged policy failures. No product integration or runtime acceptance is claimed.

## Maintenance evaluation handoff — technical candidate ready, policy FAIL

- Authorized bounded candidate is complete under ignored `cache/gpui-maintenance-candidate`; final findings, exact commands, source/API boundaries and limitations are in [maintenance-report.md](research/maintenance-report.md). Product pin, Cargo/lock/crates/CI/deny and UI remain unchanged. No window was launched.
- Final source Windows build, candidate warnings-denied clippy, direct GPUI warnings-denied clippy, modified-source rustfmt and actual WGPU direct-consumer host API check all exit0. Candidate all-targets typecheck also exited0 before the bit-identical constant correction; final build/clippy compiled that correction.43 distinct tests pass:14 GPUI SVG/font,7 image/cosmic/fontdb harness,17 upstream pulp,1 upstream fontdb and4 upstream cosmic. Native Apple/Linux/Web/GPU/window behavior is not inferred from these checks.
- Final unchanged full `cargo deny` exits4: advisories/bans/sources pass; arrayref0.3.9 BSD-2-Clause, libbz2-rs-sys0.2.5 bzip2-1.0.6 and Apple build-tool cbindgen0.28.0 MPL-2.0 still fail license policy. No exception or advisory ignore was added. Fontdb0.24 contains an upstream-maintained minimal ttf-parser0.25.1 metadata subset; package-node advisory absence does not mean all parser ancestry is removed.
- The last approved source addition is one original-tag Rust1.99 lint fix: `1.618_034` → `std::f32::consts::GOLDEN_RATIO`, proven identical f32 bits0x3fcf1bbd. Initial clippy failure, mistaken PHI-name proof failure, cache-read preparation failure/old-source lint retry and Cargo inactive-target resolver panic are preserved. The actual WGPU consumer separately compiles successfully. Full chronological status/exits/raw logs are retained; all command sessions have completed.
- Source verification: exact-tag diff exit0 and empty status for copied GPUI/workspace/path manifests/fonts;12 original crate archives match locked checksums and2,004 source files; five copied packages have only the documented patch/lock/notice differences. Original OFL texts, copyright and Plex RFN are retained. Derived TTC stays ignored-cache-only; no proprietary Windows font is redistributed.
- `maintenance-prepare.py`, source patch, exact retained manifests/locks, fixture/test sources, source validation and final evidence manifest make the candidate reviewable/reproducible. `maintenance-product-input-comparison.json` confirms all117 baseline inputs and92 classified product inputs unchanged, with no new crate files. Original99 route evidence remains historical, untouched.
- Next action belongs to Root: independent check, then explicit candidate/license decision. Product I-2 integration and I-3–I-7 remain gated. Existing physical NOT_RUN/Gate B boundaries and no08–11 scope remain unchanged; no commit/push/archive/journal/task switch occurred.

## Independent maintenance review recovery — 2026-10-14

- The independent review found and repaired two real cmap regressions: GPUI coverage semantics and HarfRust0.12 format2 consumption. The original red logs and insufficient pixel-only probe remain retained. The final strict old/new SVG comparisons match entire RGBA buffers for formats2/13; cached and alternate HarfRust constructors are tested. Only the approved GPUI helper and isolated HarfRust two-file scope was expanded; product/policy/UI remain unchanged.
- **Final authoritative recovery entry:** `research/maintenance-independent-prepare-final.py`. It reconstructs original fixed inputs, applies GPUI helper and HarfRust repairs, restores final locks, and verifies all526 final source files. Original `maintenance-prepare.py` / `maintenance-source.patch` alone restore the defective historical candidate and must not be used as the final adoption set. Final report/index: [maintenance-independent-report.md](research/maintenance-independent-report.md), `maintenance-independent-manifest.json` plus its SHA sidecar. Snapshot SHA `cc9a8ded6bfeeed1c0286ba5bf2e3ecf750b6ce34987f11d4b6e23d85ec8a960`; maintained-package combined diff SHA `003fd8d088ce9d5466c059608cf3990418cb2b4fdb3094798b1407b0f70b0a83`.
- Separate passing sets:16 GPUI,7 root,17 pulp,1 fontdb,4 cosmic and205 HarfRust tests;4 diagnostic tests include repeated comparisons and are reported separately. Final root/GPUI strict lint, all-targets typecheck, Windows WGPU consumer compilation, native candidate build and modified-source formatting return0. Direct HarfRust strict lint remains exit101 with exactly the same6 message/location pairs as the untouched upstream package; no unrelated source or lint allow was added. Do not summarize this as every lint passing.
- Final unchanged locked/offline full deny returns4, exactly arrayref0.3.9 BSD2, libbz2-rs-sys0.2.5 bzip2 and Apple-build cbindgen0.28 MPL2. Advisory/bans/source checks pass; exceptions and production adoption remain unapproved. The fifth maintained package is MIT HarfRust0.12.0. The three consumer locks only replace its verified registry source with the local copy; no package/version additions/removals occur.
- Final integrity review confirms12 archives/2,004 source bytes, exact-tag diff/status, untouched original handoff hashes, all117 baseline/92 classified product inputs and all86 crate files. Old product128 default/63 native results remain applicable and were not repeated. All review commands are complete; no reviewer Cargo/rustc/rustfmt process or window remains. Gate B/C, physical NOT_RUN, native platform/runtime and theme UI boundaries remain unchanged. Root owns candidate/license approval and parent/ADR recovery; task07 itself is not complete.

## Root review decision checkpoint

Root independently read back all753 final manifest records with zero differences. The concrete final-source/three-exact-license proposal is parent research/gpui-license-review.md; the user has been asked for this separate decision and no answer is assumed. Final report is in implement/check context; task.py validate returned0 with33/30 entries. Product pin/deny and I-2/I-3 are unchanged pending approval. Original maintenance-only approval is honored; do not request that scope again. Physical operations remain explicitly unavailable this turn.

## Product adoption approved — 2026-10-15T00:26:24.572570+00:00

User replied "批准采纳及这三个精确例外（推荐）" to the final candidate/exact-license proposal. Resume I-2 now: preserve the final reviewed five-package source, perform minimal product manifest/path adaptation, wire optional app/ui features/CI and only the three version-specific deny exceptions, then verify the actual product graph/build. Once I-2 gates pass continue I-3–I-5/I-7 automatically; do not ask again. Root owns ADR/parent planning and native UI actions; implementation agent owns07 product/source/tests and command evidence. No lint/advisory waiver, commit/push/archive or08–11 implementation. Physical I-6 portions remain NOT_RUN.

## Product I-2 checkpoint

The user-approved final five-package source is copied to vendor with original notices/fonts and a separately recorded test path/in-memory TTC adaptation. The reviewed526 inputs matched before copying. Only three exact license exceptions were added. Product native build exited0 (55.52s); full all-features locked/offline deny exited0 with advisories/bans/licenses/sources OK. Initial target/advisory-cache permission failures and approved retries are retained in research/product-status.txt and product-i2-*.log. Feature/Windows CI wiring is in place. I-3–I-5 implementation now proceeds; neither I-2 nor prior candidate evidence is UI/Gate B acceptance. Root owns isolated native configs and desktop control.

## Product I-3/I-4/I-5 checkpoint

Shell, snapshot-only audience, presenter/theme, single worker import and main-thread window/focus/role coordinator are implemented but not yet runtime-verified. Native product typecheck returned0 in product-shell-check-3.log after two retained API failures; no Cargo command remains running. Three discovered sequencing defects are being fixed with red regressions first: startup import rejected during presenter confirmation; Idle settings Escape incorrectly sends CancelJump; Stop cancellation briefly republishes Live. The shutdown regression consumes the same ordered publication plan as the native coordinator. Partial replacement cleanup, retired window IDs, focus confirmation and invalid jump text also receive pure tests. Next: run red suite, fix branches, strict product lint/build, current-source vendor regressions, then hand native binary to Root. I-6 physical operations remain NOT_RUN.

## Product automated verification / locale correction

The three sequencing regressions and reserved-shortcut conflict were recorded red, corrected and verified green. Actual shutdown/close/import/focus branches consume the tested decisions. Minimal locale integration now loads45 shared keys from existing en-US/zh-CN catalogs using startup general.language; no09 UI or schema change. The original pre-locale CI remains historical. Locale CI returned0 with145 tests; native feature-on app/UI tests returned0 with3 read-only CLI and18 UI tests. The skeleton-only startup test is cfg(not(gpui)), retaining4 default CLI tests without a native launch. Final native strict app/UI clippy returned0.

Actual product vendor tests returned0: GPUI16, HarfRust205, strict parser/full-RGBA/fallback diagnostic4. Product-only cfg(test) adaptations and all vendor hashes are in product-vendor-final-comparison.json / product-vendor-final-adaptation.patch; reviewed runtime bytes remain identical. Old pre-format adoption receipts are retained. Product comparison now shows107/117 baseline inputs unchanged and10 intended integration deltas; all prior core/platform/pdf/render/document inputs remain identical.

Final full deny offline attempt returned1 because cross-platform locked source cache entries were unavailable; complete-target normal fetch/audit retry is RUNNING in session54051, product-final-full-deny-retry.log. Prior feature test offline miss and approved retry are also retained. Do not infer final native build/freeze from these intermediate results: after audit returns, run core tree and final native build, capture product-native-freeze.json, then pause source changes for Root's native acceptance. No product window has been launched by this implementation agent.

## Product freeze / Root native handoff

The previous RUNNING checkpoint is complete: full deny retry session54051 returned0; final core tree0; final build0. A final required03 warning-log addition was followed by strict native clippy0 and native build0 (session6314). No active Cargo/rustc/rustfmt command remains. Product code/binary are frozen; Root has been notified and may start I-6. Only research/implement evidence is being finalized during native work.

Freeze: product-native-freeze.json,602 source files, snapshot `b59abb29460b9232398e7bc7a9013c19f11a0c40428b47c4eb4096182d49f4bb`; binary SHA256 `9452899cda26e8b1820c74c34b84cd531472a9ef431dd1598190e71ea3fff591`; lock `b22ab08e9e1615c721ffb02b953b0d65b37ad2552323582284d51d44847527f3`. Binary logical target/debug/ferrostage.exe resolves to the existing D:/mbx-cache target path recorded there. Pass --config <Root isolated profile> --pdfium-library <verified cache DLL> [PDF]; no environment variable is used for runtime library lookup.

I-3/I-4/I-5 and shared theme/locale shell implementation plus automated I-7 are ready for native/independent verification. Root owns native-acceptance-plan.md and all screenshots/actions; final task07 check remains required. No acceptance checkbox is inferred from compilation/pure tests. Physical unplug/port changes, unavailable single/zero-screen/hardware modes remain NOT_RUN and Gate B remains incomplete.

## Native startup red → v2 freeze

Root's first bounded native retry exited101 after successful UUID mapping: Shell was synchronously read during its update lease. Pinned open_window draws before returning. The actual GPUI TestPlatform/Shell::open regression reproduced the same panic (product-startup-red-retry.log); the preceding offline bit-set failure is separately classified. The approved fix gives Presenter15 drawing fields initialized from the current Shell borrow and refreshed by notifications; render no longer reads the controller. Audience is unchanged.

Nondefault test-support runs the real no-OS-window regression and is wired into Windows CI/just check-shell. Headless19 tests, full unchanged all-features deny, v2 just ci145 tests, all-feature/all-target strict app/UI lint and final native build all return0. No extra exception or vendor runtime change. Detailed causes/commands/source delta: product-startup-repair.md.

Root has v2 freeze and may continue native: product-native-freeze-v2.json,602-source hash `62595ac6b2fbaae3fe98053699a29cb33fabbfc44f45d86ef1dfbcce29f19f89`, exe `b96cb72ea54ddacacd9491090a5f92ef8950f7fdf720758c2d5f2dc6986bb396`, lock `2557e10b37fa5cfe63e78ce19965b8bbbd83438bdd540eed50405fbbc3dbc23e`. No active build command and no implementer-launched native window. Product code/binary are frozen pending Root feedback. v1 receipts and native panic are retained; they are not relabeled PASS.

## v2 native close-hook red / v3 repair in progress
Root native v2/full trace reproduced update re-entry during tick topology-event cancellation (not placement timeout): retire_pending -> close/remove_window -> synchronous on_window_closed -> Shell.update. Source and binary freeze v2 are retained. The next scoped repair covers the actual installed callback, topology cancellation, placement-timeout cleanup, replacement retirement and current-window close. Explicit application quit will preserve zero-window topology recovery. Placement rules remain unchanged; actual observations and cancellation causes will be logged. Headless callback red test is RUNNING; no implementer native launch.

## v3 close-callback repair / native freeze
The preceding RUNNING red checkpoint finished: initial sandbox lock denial101 is separate from actual headless red retry101 (all3 callback regressions reproduce native update re-entry). A tracing identifier compilation failure is retained separately; corrected headless suite22 PASS. Production close callbacks now defer original WindowId consumption until the Shell lease ends; Explicit quit permits temporary zero-window topology recovery, while current Presenter user-close still quits. Placement rules unchanged; cancellation/actual-state diagnostics added. Root corrected the v2 trace interpretation to topology-event cancellation, not placement timeout.
just ci145 tests, all-feature strict app/UI clippy, native build and diff-check all exit0. Only shell.rs changes fromv2 (601/602 inputs unchanged); lock/vendor/policy unchanged. See research/product-v3-close-repair.md and product-v3-source-delta.json. V3 freeze source7504bbec590d2c3df76a125350f8142838dc5f894ca8e76c1ee4802ee871c27a; exe fa8a85611c42a1039f172534c3b1200cfc525cb857459ef0d10fa1e3d359d74b. Root has exact resolved path/args and may run native. No active build commands at handoff, no implementer native launch. Source/binary remain frozen; native v3 and independent check pending.

## v3 native action red / v4 investigation
Root observed visible B Presenter, actual display65731 placement confirmation, Idle/no Audience and titlebar close exit0. Ctrl+O from focused Presenter produced window not found and no chooser (native/v3-open-dialog.*); source freezev3 retained. Root completed process/session cleanup and authorized minimal action/window-lease repair, actual input-route tests and localized names for the two unsupported default shortcuts. No native operation is performed by implementer.

## User stopped Computer Use — 2026-10-15T01:29:43.941377+00:00

- V4 source602/602 hashes matched. Freeze `product-native-freeze-v4.json`; exeSHA `0e435312372a914db92e7dcb1509c28ad6c739ab3db660514a205879bcb10615`, sourceSHA `208d6c5ec1069ab5e323cb0112c2191a902509f0f0ff47a534ef2ea87114c052`. Implementation reports26 headless tests,145 default CI tests, all-feature/all-target strictlint and nativebuild passed; independent final07 review has not run.
- Root launched isolated a-light Idle via `native/run-native.ps1`, PID167888, exec session30311. `native/v4-a-light.launch.json` binds config,argv,exe andsource. The process was still waiting when the launch tool returned; its exit is not verified.
- At the first V4 Computer Use selection/activation/capture call, the tool reported physical Escape by the user and explicitly required stopping Computer Use. No V4 screenshot/interaction result is accepted. No further UI calls, forced process cleanup or action retry was performed. The implementation agent was interrupted to preserve the frozen source.
- Resume: run the requested disk/task/process audit; inspect PID167888/exact executable and session30311 if available, along with v4-a-light stdout/stderr/exit files. Do not launch a duplicate or infer the process is closed. Reconfirm V4 hashes, then resume N01/N02 Ctrl+O/button import, remaining available native cases, and final independent07 check. Existing physical unplug/port-change limitation still applies.
- Parent remains incomplete; GateA technicalPASS, GateB incomplete, GateC/D NOT_RUN;08–11 planning and dependency-blocked. No commit/push/archive/add_session.

### Resume 2026-10-15T01:35:17.400830+00:00
- User explicitly resumed Computer Use. Current07 and v4 source602/exe/PDFium hashes match (`native/v4-resume-identity.json`).
- PID query is sandbox-limited (CIM access denied); do not infer exit from empty Get-Process. Sky found existing v4 Presenter61084300, active and interactive; session30311 remains pending. Reused this window, no duplicate launch.
- Ctrl+O now opens actual native chooser (`native/v4-open-dialog.json`); import and remaining native matrix IN_PROGRESS. Existing Idle error at resume retained as observation, not attributed without reproduction.
- Physical unplug/port tests remain NOT_RUN by user's prior response. No commit/archive.

### Native checkpoint 2026-10-15T01:38:49.876409+00:00
- v4-a-light normal titlebar close: exit0 receipt, PID167888/session30311 completed. Chooser opened and Cancel restored Presenter; dialog field automation returned stale/mismatched accessibility and did not enter a path, so chooser import remains UNVERIFIED (not product failure).
- v4-a-cli running PID66348/session54803, same freeze, own long-Chinese-name PDF. CLI import Ready/page1 of9/no audience PASS. Start confirmed actual A display131073/fullscreen; native audience page1, PageDown page2, B full black, Escape removed audience and returned Ready/page2 (original images `native/v4-a-*`, `v4-escape-ready-page2`).
- Curtain transition not directly captured; automated gate and placement log are separate evidence. Timer continuity not visually displayed by07. Remaining matrix IN_PROGRESS; physical cases unchanged NOT_RUN.

### Native checkpoint 2026-10-15T01:44:50.635312+00:00
- Swap PASS for actual PresenterA/AudienceB roles, new HWNDs43124864/51186720, placement logs confirmed; isolated a-light.toml roles read back. Page2 and blank state preserved; B toggled restored page2. Timer UI absent07, not visually claimed.
- Audience Ctrl+O opened chooser at PresenterA (evidence v4-audience-open-presenter-dialog); Presenter Open button simultaneously disabled is a policy inconsistency under independent review. Modal Cancel preserved Presenting. Text injection remains UNVERIFIED: native key d did enter filename, but type_text cleared selection without inserting path; helper focus/index metadata stale.
- Real captured sizes801x655,391x655,322x640: primary action visible, long title truncates, warnings wrap. Exact DESIGN size matrix not yet complete. Tab→Open, ShiftTab→Start visible rings PASS. FAIL: focused Start + Space advances page2→3; Return no action/Ready. Red native evidence v4-space-focused-start-advanced-page3/v4-enter-focused-start-no-action. Independent checker notified, product freeze still held.
- Root normal-close PID66348/session54803; see v4-a-cli.exit.json for returned code. Remaining native matrix and fixes IN_PROGRESS.

## v5 focused controls / import admission / Idle actions
Scope: real GPUI Tab-focused button Enter/Space priority before global actions, disabled/modal behavior, Open rejection before chooser with confirmed Presenter warning owner and focus-time recheck, and reproduced Idle navigation/blank no-op. Native v4 Start keyboard failure is retained; earlier resumedIdle error trigger remains UNKNOWN. Only07 UI routing/model-view availability and regressions will change. Checker is read-only. Source/exe v4 and all earlier receipts remain retained. Real input red tests are being prepared; no app windows or Cargo process active.

### v5 regression checkpoint
Actual GPUI red receipts: product-v5-red.log5 failures (focused controls/Open/Idle), topology-red3 recovery failures, jump-red and jump-numeric-red each a specific editing conflict, preview-red2 hidden preview failures. All now fixed in07 only and product-v5-green-final.log returns0 with41 tests. Existing core/vendor/manifest/lock remain unchanged. Keyboard uses local KeyContext, same deferred coordinator; popup admission rechecks after current Presenter activation; unsafe topology recovery reuses confirmed window replacement; Stop/Swap discard preview without changing core Stop. Physical/topology identity limits remain explicit. RUNNING next: required just ci, strict all-feature app/UI clippy, native build, then new immutable v5 freeze.

### v5 frozen handoff
All RUNNING quality commands returned: justci0/145 default tests, strict app/UI all-features/all-targets clippy0, native build0, diffcheck0. Product v5 frozen: source602 aggregate39907c8b4fdb0221ede20be1889b9ca20d6768a3794652479a7c451c4b1aa8f0; exe3516cca7cf432cf1e4dace74f1a065c19509d10be1c4bb54f93a7b9e0c382fa8; freeze SHA7cc8e5523c077c5d991aef6bbc4797a561022512b4fd67f66e8c24aeb813a442. product-v5-repair.md and focused patch/source delta contain full scope, red/green and limitations. Root has path/argv and owns native; checker is read-only reviewing this freeze. No active build at handoff; source/binary remain unchanged during native. Next step: Root native N11/Jump/Open refusal/preview continuity retest, then independent evidence closeout; unavailable physical cases remain NOT_RUN.

### v3–v5 evidence closeout
The version manifests are finalized without rewriting prior freezes or receipts. V3:20 records, SHA256 `7cb83dd335ec93150d5301ed769a2bc877c213460af1e6bae8f0863cca942e49`; V4:23 records, `b36ab615eb851a4c125120108eab834cc7c1f5b08fdc5504df4b8081b0c5562a`; V5:24 records, `de8e061e864aff6d45deeb179505bf60ce8eb432802be71824b315edfa3321aa`. Each records its freeze, focused patch, source comparison, red/green logs and extracted command/exit ledger; mutable Root native reports are separately owned. `research/product-version-manifests.py` writes only new evidence files, verifies every record and confirms current602/602 product inputs still match v5. No Cargo command or product launch occurred during closeout. V5 code/binary remain frozen pending Root native/check feedback; task07 and Gate B are not declared complete.


### 2026-10-15T02:06:59.315885+00:00 — v5 native input checkpoint (Root)
- Frozen product v5 exe `3516cca7cf432cf1e4dace74f1a065c19509d10be1c4bb54f93a7b9e0c382fa8`, 602 inputs verified; independent review manifest `04da229f448f7937b85c62ec3459e0ae619e0ad150e85bd27ded61e2fdf762b5`.
- `v5-input` PID107776/session93123 normal title-bar shutdown returned exit0 (`research/native/v5-input.exit.json`). No live process from this run remains.
- Actual native PASS: saved swapped roles relaunch PresenterA/AudienceB; focused Start Space starts without advancing1; audience Ctrl+O refuses at activated Presenter with localized warning/no chooser; Jump23→Backspace2 clears invalid error; preview2 keeps audience1; Return commits audience2; preview3 Escape retains Presenting2; PageDown then3; focused Stop Return ends Ready3. Screenshots/JSON prefix `v5-*` in child research/native.
- New FAIL F7: Settings click→Tab(close-modal)→Return removes focused control; accessibility focus becomes window region0; subsequent Tab does not focus a control or display a ring. Same region0 observed after Enter Stop. `v5-modal-close-tab-focus-lost` evidence. Needs real-GPUI red regression and focus restoration repair after native matrix, then targeted native recheck.
- Remaining native C vertical / dark / system / en-US / conflicting shortcut / error recents / actual config lock Retry in progress next. Physical unplug/ports and actual single/zero screens remain NOT_RUN by user availability. GateB incomplete;08–11 not started.


### 2026-10-15T02:14:50.376103+00:00 — v5 remaining native matrix checkpoint
- C vertical: native audience1 and White all-screen captures, audience Escape removes audience and returns Ready, PID128024 normal exit0. Dark actual paint captured.
- Actual read-lock/save Retry: PID166216 Swap moves Presenter B→A while locked config hash remains B96B8708FCB7207F7673C322BE62D1F2F68D1C9FF382FE3607FBF7D0D7249AB9; lock released, Retry saves role change SHA41798CB34079E64D46078604E4C382EC1698D1AD925977870CAF8C71E66FEFE3. Functional persistence PASS; F8 FAIL UI duplicates save error and retains red stale error after successful Retry. PID166216 exit0; helper44579 released/exit0.
- First v5-conflict fixture run invalid TOML due Root writer CRCRLF (not product regression). Product backed it up and fell back; excluded from shortcut/recents proof. Repair from original backup with UTF8 bytes, tomllib PASS, error-recents-preparation-v2.json. PID148112 exit0.
- Corrected v5-conflict-retry PID123468: conflict visible; malformed/missing recent imports keep raster.pdf/Ready; valid recent import clears import error; Start works and audience Space leaves page1. PID123468 normal exit0.
- Harness sequencing error: close click returned geometry unavailable, but next system-profile launch was issued before old exit. Both isolated processes temporarily overlapped; this system run excluded from native acceptance. Closed extra PID14224 exit0 then PID123468 exit0; next runs only after verified exit. No source/build mutation or user config touched.
- F7/F8 queued for V6 after system/en-US remaining checks; native-v5 screenshot/launch/exit/lock receipts retained. Gate B incomplete.


### 2026-10-15T02:18:22.696283+00:00 — v5 frozen native batch ended; V6 F7/F8 next
- English Ready actual labels PASS (PID174648 exit0); two resize attempts did not alter1102x791 outer capture and are NOT size acceptance despite provisional filenames.
- Isolated system-theme Idle dark initial appearance captured; PageDown Idle safe/no error (PID26988 exit0). Live OS theme event remains NOT_RUN.
- No-audience-role config Ready with disabled Start and visible reason; ShiftTab skips disabled Start to recent entry (PID174896 exit0). This is not physical single-screen acceptance.
- No live Root native runs or lock helpers. V5 partial receipts remain historical. Releasing source freeze for scoped V6 F7 focus removal and F8 stale/duplicate persistence feedback; implement then independent check then targeted native regression.08–11 remain blocked by GateB.

## v6 F7/F8 scoped repair
Root released v5 freeze after all native runs exited0. Scope is Presenter focus recovery when an actually focused subtree disappears, and a single SettingsSession persistence-feedback owner preserving unrelated Shell errors. Actual native F7/F8 red receipts remain in native-v5-checkpoint.md / native-v5-evidence-manifest.json. Add real GPUI key/focus and isolated actual SettingsStore red regressions, then repair and repeat the required quality gates. No vendor/core/manifest/policy/UI scope expansion or native launch. Prior v1-v5 files are retained.

### v6 regression checkpoint
Actual red: product-v6-red-3.log4 failures; product-v6-focus-red-final.log3 focus-removal failures with final valid-focus assertions. Earlier lock denial/two test-fixture compile failures are separate. Two green attempts retained a real modal focus failure; holding the return handle until deferred dispatch-tree validation fixes it. Final product-v6-headless-final.log exit0/49 tests, including actual key paths, newer focus/blur, audience ownership, import/replacement save failure and durable retry read-back. Product changes remain presenter.rs plus shell.rs; core/vendor/policy unchanged. Next required commands: just ci, strict all-feature consumer clippy, native build, then immutable v6 freeze.

### v6 frozen handoff / evidence closeout
All required commands returned: real GPUI headless49 PASS; just ci145 default tests plus fmt/lint/default deny exit0; strict all-feature/all-target consumer clippy0; native build0; diffcheck0. No active implementer command or native launch. Product is frozen for Root independent/native review. research/product-v6-repair.md distinguishes actual red from lock/fixture compilation failures and incomplete repair attempts, and records the return-handle lifetime correction, single persistence-feedback owner and test limits.
Freeze product-native-freeze-v6.json SHA b7df3a62ff2b1b7212924e47f79d72468ff86356265a1aaf650c0694adb6d7ea;602-source aggregate b71eb54d7da71f9b30ac5ab3bdb58fc1ada637bf2991adde10041f0a172170be; exe d4f2e653ee793176b384034797ad99e1ad665aa0da6535a454ba88a36b055074. Only presenter.rs/shell.rs differ from v5;600 inputs and lock/core/vendor unchanged. Focused patch SHA69bed99db29112e2a948fc76896fbc227b5d8776e034f1de17a41baca63d5300. New v6 evidence manifest26 records SHA dac5512c8887bbb3278e0b9c68fc8a622731fbc40eea187aa7dfef090a40912c, every receipt and602 source/exe hash read back. No previous evidence was overwritten.
Next: independent v6 review and Root native focus after modal/Start/Stop/Retry removal plus real read-lock persistence feedback. Headless Start lacks watcher and remains native-only; new V6 native outcomes pending. Physical unplug/ports and unavailable hardware modes remain NOT_RUN. Gate B/task07 not declared complete; no side-effect delivery.


### 2026-10-15T02:35:32.557578+00:00 — V6 independent check ready; native RUNNING
Independent reviewer allows targeted native with no further source edits: F7/F8 code and actual red/green/Store read-back checked;602source/exe/DLL,108v5native and26implementation receipts match. `product-independent-v6-review.md`; hash index finishing read-only. Start→Tab lacks a direct automatic case and must be observed natively, not claimed automated. Root verified identical V6 identities in native/v6-root-identity.json. Starting isolated a-dark targeted native; no outcome yet.49GPUI/145default/strictlint/build are PASS, task07/GateB still incomplete.


### 2026-10-15T02:40:49.898733+00:00 — V6 F7/F8 actual native PASS
- PID29092/session21407 still live, Ready2 on PresenterB; no audience. Settings-lock session59632 released and returned0.
- Settings Close Enter→Tab/Open PASS; focused Start Space→Presenting1→Tab/Settings PASS (fills explicit Start native case); focused Stop Enter→Ready1→Tab/Open PASS. Focused Jump Commit2→Tab/Open and G→Escape→Tab/Open PASS.
- Actual config lock Swap moves PresenterA→B preservingReady2; single unsaved warning/Retry only; repeated Retry while locked keeps dirty state. Unlock→focused Retry Enter clears warning/button, durable SHA B96B8708FCB7207F7673C322BE62D1F2F68D1C9FF382FE3607FBF7D0D7249AB9 and Aaudience/Bpresenter; next Tab visibly and accessibly focusesOpen. Before lockedSHA41798… unchanged. native/v6-settings-readback.json, v6-dark-lock receipts and v6 screenshot/JSON evidence.
- F7/F8 scope verified natively; full07/GateB remain incomplete (physical topology and unexecuted matrix). Remaining chooser/size probes and normal close next; do not duplicate app.


### 2026-10-15T02:50:54.909514+00:00 — V6 native closeout; external/manual gate remains
- Successful native chooser import used actual folder navigation to `fixtures/pdf/raster.pdf`; Ready2 reset to Ready1/9 with focus on Open. `native/v6-chooser-import-pass` screenshot/AX evidence; PDF SHA256 stayed `25a7932f46ee8fcd4866011921c574b4f5b343af05b793bc397b7e93ed0df013`.
- `v6-focus-save` PID29092/session21407 normal title-bar close returned0 at02:45:52Z. Settings lock helper59632 had already released/returned0. Only after that, `v6-relaunch` PID175992/session68722 launched with persisted config SHA `b96b8708fcb7207f7673c322be62d1f2f68d1c9ff382fe3607fbf7d0d7249ab9`; actual PresenterB display65731 confirmed, Idle/no audience and dark initial appearance. Settings remains the explicit07 placeholder. Normal title-bar close returned0 at02:46:51Z; final Sky inventory has no FerroStage window.
- F7/F8 targeted native checks PASS; full07/GateB NOT complete. No active Root app, lock helper, Cargo job or implementation agent. Current remains07;08–11 planning/blocked, no new task tree and no delivery side effects.
- V6 native ran before any frozen input changed:602/602 plus exe/DLL/fixture matched. After app exit, only DESIGN.md's stale Known Gaps paragraph was updated to current evidence; its frozen bytes are retained as `research/native-v6-design-before-closeout.md`. Compiled sources, dependencies and binary remain unchanged; `research/native-v6-closeout-identity.json` records this exact documentation-only delta. No old freeze, failed receipt or independent review is rewritten.
- 12/12 context validation passed (`../10-14-presenter-v0.3/research/context-validation-v6.json`); this is context validity only. Latest code just ci145, actual GPUI49, strict all-feature/all-target consumer clippy and native build remain applicable; no unnecessary duplicate Cargo run. Final diff/document/identity review follows.


### 2026-10-15T03:03:18.786383+00:00 — 独立原生证据复核结束 / 本轮外部阻塞保存
- 新独立报告：07 `research/native-v6-independent-closeout.md`。限定V6 F7/F8、选择器、保存角色重启及正常关闭证据成立；3项PRD勾选对应原V4 A/B、V5 C与当前only-snapshot边界，不冒充同次V6全矩阵。
- 独立清单 `native-v6-independent-manifest.json` 213/213匹配，SHA256 `0aec8d0e6f68591adcc695f3cd89fdfc29efb601f0bcda575b0e47c66910fb5e`；54/108/26/141既有清单也分别匹配。V6 native结束后只有DESIGN第151行状态更新，全部94crate及可执行文件/DLL/fixture未变，复用49GPUI/145default、lint/build/deny证据。
- 父PRD/design/traceability、feature-parity及PROJECT_PLAN已校正过期I-2状态和采用版本；13份当前文档9个本地链接检查通过，2个历史代理UUID伪链接已改为标识文本。初次失败与修正后报告均保留于父research/native-v6-document-closeout.json和-v2.json。12/12任务上下文validate仍适用（上下文清单无后续改动）；不等于产品验收。
- 所有受控V6 app/锁助手及实施/check代理均已返回；没有未完成命令或活跃native进程。本会话current仍07；父planning、01–07 in_progress、08–11 planning，未手改状态或启动受阻任务。
- Gate A技术PASS；Gate B INCOMPLETE；C/D NOT_RUN，父AC-1…10、M1/M2未完成。03物理项和07剩余原生矩阵/退役窗口诊断保留待验，08–11依赖受阻；用户本轮无法物理配合，不重复询问或绕过Gate。恢复从这些未完成验收继续，不重做已验证实现。
- 分支main、HEAD ab78a84d96c7cf4a0bbc8dfd4f8e45da180bafb3；全部代码、文档、task与证据保留未提交。无commit、push、archive或add_session。


### Resume native checkpoint 2026-10-15T03:26Z

Fresh V6 audience PageDown/black/Escape and dark Ready2 observed; `07/research/resume-v6-native-checkpoint.md` retains exact limits. Both owned runs exited0, first exit trigger unknown, second AX Close observed. Resize attempts did not establish logical size; cursor/Curtain remain partial. Generic retirement and invalid-handle errors reproduced. Frozen V6 source released only after exit for minimal source-location/client-geometry diagnostics; no vendor suppression or Gate B PASS.


### V7 diagnostic implementation checkpoint

Two first-party files add tracing source location and actual bounds/viewport/scale diagnostics only. Implementer reports fmt, just ci145, GPUI49, strict consumer clippy and native build exit0; first sandbox target-lock failure is retained separately. `product-native-freeze-v7.json` binds exe14dfd1f97b35df5611d94a034e7cd474a4d0556f9f5481ff2b76002f5839b309 and602inputs. Serial independent review precedes Root native attribution. No retirement repair is claimed. Supported SystemSettings launch returned no targetable window; live OS theme event stays NOT_RUN, no setting changed.


## Planning correction — display selection before further acceptance

The screenshot feedback replaces the primary Swap workflow with G-13. This is a planned requirement, not implemented UI. Review parent `research/display-assignment-plan.md`; resume with regression-backed retirement repair and I-8, then serial check. Preserve V6/V7 historic evidence without treating config-based role tests as new UI acceptance. Existing03/07 physical/native gates and downstream08–11 dependencies stay in force. No commits, pushes or archives.


### 2026-10-15T03:52:28.734297+00:00 — I-8规划审阅完成（实现未开始）

父research/display-assignment-review.md已闭合三项规划冲突；单屏“不播放/本屏窗口播放”和多屏物理目标通过同一草稿提交角色与既有single_monitor_mode；Presenting“不播放”明确安全停止。父+11子上下文12/12通过，非UI验收。下一实施批从退役回归/修复及I-8恢复，不重复既有V6/V7检查、不提前推进08；新改动继续未提交。

### 2026-10-15 — V8 implementation and automated gate handoff

Implemented the authorized regression-backed retirement repair and I-8 shared display assignment. `research/product-v8-implementation.md` records exact contracts, file ownership, real red/green receipts, tests and boundaries. `DisplayDraft`/`PdfTarget`/`numbered` plus the shared panel/Identify reuse Shell's existing role transaction; Swap remains a keyboard compatibility action. Runtime no-play commits Curtain→Stop→close only after successful replacement placement, retaining page and freezing time; valid migration preserves the running core state. Fresh topology invalidates stale drafts; no future-schema writes, index persistence or parallel role authority.

Actual GPUI lifecycle red reproduced the V7 window.rs1914 errors; owning Window token plus weak callback lifetime and deferred first-party should-close pass real regression. Native Windows HRESULT/dispatcher absence remains for Root, not inferred from these tests.

Product checks: just ci148; UI feature suite62 (41 real GPUI +21 pure); strict all-feature/all-target app/UI clippy; unchanged full all-feature locked/offline deny; native gpui,pdfium build all PASS. Initial command/harness/product failures remain under product-v8-*; final strict build follows the test-only import correction. Source/exe/DLL/PDF/config freeze and evidence manifest are prepared as product-native-freeze-v8.json and product-v8-evidence-manifest.json. No controlled Cargo/native process remains. Next is serial trellis-check and Root-only native UI/close acceptance. Gate B stays INCOMPLETE; physical tests and real single/zero displays NOT_RUN; 08–11 not implemented. All new changes remain uncommitted; no push/archive/add_session.

### 2026-10-15 — V10 mixed-DPI placement correction

Root's V9 native check found C client1280×912 wider than its1066.67 logical display; target monitor alone had incorrectly satisfied windowed confirmation. `research/product-v10-implementation.md` records the native geometry, pinned source cause, red2→green2 regressions and full repair boundaries. Only first-party `shell.rs`/`policy.rs` changed: use target visible work area with32px native-frame allowance, restore requested client size through existing GPUI resize after target DPI settles, require actual size and containment before transaction commit. Incorrect placement still times out and rolls back running state/windows/settings. No vendor/dependency change or retirement log suppression.

Final product checks PASS: just ci150; feature UI68 (45 actual GPUI+23 pure); strict all-feature/all-target consumer clippy; full unchanged all-feature deny; native gpui,pdfium build; diff check. New GPUI scroll/selection/Cancel tests cover Chinese/English800×600,390×844,320×640 clients, without claiming native evidence. Both product red failures and initial synthetic fixture failure are retained. FreezeV10 binds609 source/contract inputs aggregate2ee35522a9f885cf8c62ac5007e7feecc7d5c32f28949fc2f7b65e4052383a67 and exe783d5769785029f11fa73f631720b47f7280e5124b30801d1930ee213dee37af; source delta is2 UI files plus Root's updated state-management contract,606 unchanged. New evidence/config-input manifest follows this freeze. Root owns serial independent check and actual C titlebar/fit/native sizes acceptance. All V6–V9 evidence preserved; Gate B remains incomplete, physical and real single/zero tests NOT_RUN,08–11 unimplemented. No commit/push/archive/add_session/native launch.

### 2026-10-15 — V11 bounded native-frame correction

V10 native C startup FAIL remains preserved: requesting1002.6666×760 yielded1000×754; strict placement timed out. Root stopped the now-windowless owned process with exit-1, not normal-close PASS. `research/product-v11-implementation.md` records pinned frame-inset conversion, actual4×9 physical pixel evidence, and regression coverage missed by the exact headless resize model.

Only shell changes: retain API request and pre-request observation; after a changed native bounds callback permit at most one≤32px/axis client-delta compensation. Stale callbacks, large error or nonconvergence do not loop; original2px size and full usable-containment checks plus timeout rollback stay intact. New actual GPUI Windows-response model red1→green1; bounded/nonconvergent/stale cases preserve existing running owner/state/settings. Final UI70 (47 actual GPUI+23pure), just ci150, strict all-feature/all-target consumer Clippy, full deny and native gpui,pdfium build PASS. Initial new-test compile error is preserved separately from final70 pass.

FreezeV11 binds609 inputs with607 unchanged; shell and Root state-management contract are the sole delta. Aggregate fdec41e24793817d526127a3928c9e1f0bfafa021a293dce26cc9125f87c48f1; EXE027ab17fc23a92c00c480148b06c1ac115f81fcfe1a2145ac210dd6dd79a4d6d. Next is serial independent check then Root actual C startup/client/titlebar acceptance. V11 native still NOT_RUN; remaining native/physical gates unchanged. No app launch or delivery side effect by implementer; all work uncommitted.

### 2026-10-15T05:36Z — Root V11 native results and F11

Independent V11 check passed; Root completed four native runs, now all exited0 with empty final Sky inventories. `research/native-v11-checkpoint.md` records new-panel B/A→B/C Presenting migration preserving page/blank/running timer, no-play Ready2 elapsed138110ms freeze, real read-lock save failure/Retry and disk read-back, B/C persisted relaunch, schema999 readonly unchanged bytes, dark320x640 and389.6x844 actual clients. Initial C resize converged, but external moves during CU interruption preceded the screenshot, so initial full-titlebar visibility remains UNVERIFIED. Light800x600.8 is only an approximate-size observation; exact800x600/1440x900 and remaining matrix are pending.07 placeholders do not prove08 PDF rendering.

main/relaunch/future logs have no retirement errors; dark actual titlebar Close logs dispatcher.rs129 Invalid window handle at05:36:19.862676Z despite exit0. This is F11 FAIL. Root released source only after exit and dispatched the existing implementer for V12 minimal regression/repair, then serial independent check/new freeze/native retest. No log suppression or additional package-policy approval inferred. Gate B INCOMPLETE, physical/true single-zero NOT_RUN;08–11 gated and P2 retained. No commit/push/archive/add_session.

### 2026-10-15 — V12 first-party poll-task lifecycle handoff

Read `research/product-v12-implementation.md` and this tail explicitly because injected parent/child documents exceed32768 bytes. Pinned dispatcher129 posts a wake to the platform HWND; Shell's detached25ms loop had no quit cancellation, and its weak-entity check occurred after foreground dispatch. Real GPUI red2→green2 proves no timer-driven Shell work after current Presenter close or actual App.shutdown; positive coverage proves retired/pending/audience close preserves live polling. Shell now owns the Task and synchronously cancels before Presenter quit and on_app_quit. No vendor/dependency/package-policy change, log suppression or V8 owning-token rework.

Final product gates PASS: UI73 (50 actual GPUI+23pure), just ci150, strict all-feature/all-target Clippy, full deny, native gpui,pdfium build and diff/read-back. FreezeV12 has609 inputs with607 unchanged; only shell plus Root component contract differ. Aggregate fa6617505a045fe2aa4dce0a3185eff47c3ed70bb283d1e1c7030e4c04054e73; EXE c8291c651a485ddff5cc00354dede2c67c25ea1080a4205e4d12fa0796789fda. New V12 evidence/config-input manifest preserves all earlier identities.

This repairs the proven first-party late-wake source; native F11 HRESULT absence is NOT yet established. Root next performs serial independent check, cross-version evidence assessment, then new-identity dark resize/panel/Close retest and remaining matrix. V11 dark config is parsed-identical with CRLF→LF, not byte-identical; future/relaunch byte identity is preserved in Root's checkpoint. Initial C full-frame screenshot remains unverified. Gate B incomplete; physical/true single-zero NOT_RUN, no08–11 implementation. No native launch or delivery side effect by implementer; all work remains uncommitted.


## Root V13 native checkpoint — 2026-10-15 06:30Z

V13独立检查完成：UI77（54 GPUI+23 pure）、严格lint/typecheck/fmt/native build PASS，未变default150/deny核验复用。Root五轮原生中main/dark/en/open实际关闭exit0、空窗口、干净日志；第五light-small的320x640面板关闭虽exit0，却在poll取消后报F12 vendor/gpui/src/window.rs:1943 window not found（hover回调）。总体关闭验收FAIL。所有Root app已退出，609输入/exe在06:31:45复核后释放给既有implement代理，进行V14回归与同生命周期回调审计，随后必须串行独立check、新freeze、原生复验。

V13历史source609=77aa180c7f60bcc39de7315554d9781d5f2f39effd211078a1da43cf7ed40d4a；EXE=d1164919dcdc09053df4fc6caae09be249329cccd0ad8bd482a67347561a6435。当前字节不能直接当V13。详见07 research/native-v13-checkpoint.md；92项清单/26原图SHA e08f7bb83832a0e6ccbd23b0ea5abb6a489181baad8ef85f27bc224475d4aedf。

新实测包括C初始未移动前完整标题栏与严格放置、新UI C/B→B/C、精确light1440x900/800x600/390x844/320x640及dark320x640、英文新面板、Audience关闭→重新开始→Presenting主窗关闭、原生chooser取消与成功导入。light320几何截图不覆盖该run关闭FAIL。V9 Identify与V11新UI迁移/不播放计时/真实锁Retry/未来只读/重启按独立审阅适用性复用，不标成V13同次运行。

剩余可执行：dark1440x900/800x600/精确390x844及有限键盘/控件状态（含窄窗Presenting/Stop），修复后继续，不列为外部阻塞。物理拔插/重连/接口/延迟及真实单屏/零屏NOT_RUN；live OS主题、跨窗口拖放、瞬态Curtain/loading和完整光标矩阵仍有限制/UNVERIFIED。Gate B INCOMPLETE，C/D NOT_RUN，08–11依赖受阻、P2不变。所有新改动未提交，不push/archive/add_session。

### 2026-10-15 — V14 shared platform callback lifetime handoff

Read `research/product-v14-implementation.md` and this tail explicitly; long injected task documents can truncate. F12 hover is a different lifetime boundary from V12/V13 Shell task cancellation. The existing GPUI Window-owned token now covers all17 Window-access event registrations, including an execution-time check for deferred appearance. Weak callbacks cannot extend owner lifetime, token drops before native platform teardown, and every live update/log remains unchanged. Input/hit-test preserve existing default results. Existing on_close idempotent cleanup, first-party should-close, and poll/read/chooser ownership remain intact. A11y Send/channel receivers have no equivalent failing executable evidence here and are explicitly outside this repair, not claimed covered. No new package/policy/lint waiver.

Actual GPUI red-final:4 failures/1 positive control and19 removed-window errors; green:5/5. Tests retain real TestWindow callbacks after owning Window removal, and prove a live leased Window still reports its error. Full UI82 (59 actual GPUI+23pure), strict all-feature/all-target consumer Clippy/typecheck, workspace/vendor fmt, fresh full all-feature deny and actual gpui,pdfium build PASS. Default150 is V12 evidence reused after unchanged-default input checks, explicitly not a new just-ci run. Harness compile/default-assertion failures remain preserved separately from final product red/green.

FreezeV14 has609 inputs,602 unchanged againstV13. Only window.rs runtime plus GPUI test support/Shell tests and Root's3 spec/ADR files changed. Source aggregate9764bfa0d94764c314fd559d8df2b55e056fc8f6ef77882cdde89662fe5d4413; EXE4be903dc34b4bb47163816cc88a67caf8c9f9806772f931b209bf27a9d3dbcd1; freezeSHA3f00b403a9668d3a841e0ba15564d28ad804934f8b740c3d4f1803b82da41a1c. `research/product-v14-evidence-manifest.json` seals new receipts/config-inputs/process-closeout/readback. Five-dimensional break-loop and Root knowledge capture are in `research/break-loop-f12.md`; no commit performed.

Next: serial independent check, then Root-only native F12 narrow light titlebar-close/full-log retest and remaining feasible matrix. NativeV14 is NOT_RUN at handoff, historicalV13 F12 remains FAIL. GateB remains INCOMPLETE, physical/real single-zero NOT_RUN,08–11 gated/P2 retained. All new code/evidence uncommitted; no native launch/push/archive/add_session by implementer.
# Root native update — 2026-10-15 08:05Z

User deferred further width/size tests; prioritize core functionality and functional defects. V14 independent check passed. Root completed3 owned native runs, all exited. New Displays migration preserves page2/white/running timer; no-play stops projection, keeps page2, freezes122003ms across26.42s; saved no-play relaunch, conflict protection, application relocation, Start/Stop and Audience close/restart have direct receipts. Light-small F12 trigger and fresh Presenting owner Close pass scoped clean-log checks. Dark long run instead exposed F13 leaked desktop DC and F14 AccessKit Invalid HWND close panic despite exit0. See research/native-v14-core-results.md.125file/45original-image manifest477cfa6ecf65ba131eb60e705bbdbb955b77d7299b92990a16d203cf9b5c8590;609inputs/exe verified before releasing freeze. Existing implementer owns platform/GPUI regressions/minimal repair, Root owns specs/native/parent, then serial independent check. Gate B remains INCOMPLETE; physical NOT_RUN,08–11 gated/P2, no-commit boundaries unchanged.


### 2026-10-15 — V15 DC resource fix and real notes fixture handoff

Read research/product-v15-implementation.md and this tail explicitly; long injected task documents truncate. F13 is proven by real Windows GDI counts: old production enumeration leaks one DC per query (64 calls:2→66), safe real-desktop owner keeps counts flat (1→1) and all three MonitorInfo records equal. Only platform/windows.rs runtime changed; its real regression is new. No unsafe/dependency/package waiver.

Root's actual TeX Beamer notes PDF is now pinned for native work. New document/tests/notes_native.rs verifies real PDFium Right layout,5physical/4logical,labels1/2/3/3/4,overlay navigation and independent nonempty public/notes pixels; audience snapshot renders match only left full-page pixels. Existing native.rs remains byte-identical. Initial same-process PDFium reinitialization harness failure is preserved, corrected by a separate test binary. Task07 placeholder UI is still NOT_IMPLEMENTED/NOT_RUN for actual notes display until08.

Final checks PASS: just ci151,UI82(59GPUI+23pure),document native2,strict all-feature/all-target app/UI/document Clippy and typecheck,full all-feature deny,native build. F14 is NOT fixed in this product: research-only four-file AccessKit proposal plus red native panic/six green COM controls and compiled Root-only hidden-window harness await independent review and maintenance boundary decision. No sixth package adopted; no product vendor or dependency changes. See product-v15-a11y-proposal.md.

FreezeV15:622inputs aggregate fdc00fc36ad02e46234398899ef924892e690810caf24f290cc5b0ec070d835a;EXE d40bb0d113e363df6f2bae5d331fd13d8473a8fbcf5820e4438b3da5489e8fab;notesPDF e939e35f931c184fd93b2e9b95e560db73ba6a6f6dfccbb0963262f0b359dfeb. Fresh manifest preserves config inputs, all red/harness failures and candidate evidence. Serial independent check precedes Root native; F13 long-run native and F14 remain open. Additional width/size work deferred by user; physical/true single-zero NOT_RUN,GateB incomplete,08–11 gated/P2. All work uncommitted; no native app launch/commit/push/archive/add_session by implementer.

### Root V15 checkpoint — Computer Use stopped before product launch

Root executed the already compiled isolated F14 candidate test at08:41Z:16 owned hidden HWND destruction cycles,64 retained-context COM queries returned normally; see research/product-v15-root-a11y-results.md and immutable root-hidden receipts. This does not adopt or accept the product fix. Independent check completed with no source changes, strict four-crate lint/typecheck and all frozen identity checks PASS; prior affected product regressions reused with verified inputs. Final report/manifest are under product-independent-v15.

Sky list_windows then reported physical Escape user stop before any V15 app launch. Root stopped all Computer Use immediately; V15 native DPI/placement/lifecycle/notes UI are NOT_RUN/user-stopped. New real notes PDF and document/PDFium crop tests are complete, actual07 UI remains placeholders. Native continuation is recorded in research/native-v15-checkpoint.md; all prior evidence and user-deferred size/physical boundaries remain. GateB INCOMPLETE,08–11 gated,P2 unchanged; no commit/push/archive/add_session.
