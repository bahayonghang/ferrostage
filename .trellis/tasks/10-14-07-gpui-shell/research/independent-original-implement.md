# Implement — 07

1. **I-1 display研究门**：核pin Windows backend、DisplayId/uuid/bounds单位、fullscreen参数、DPI/事件与focus/keystroke；生成确有证据的gpui-display-api结论，择一允许路线。无法安全映射则保留具体阻塞。
2. **I-2 feature编译**：ui链接gpui、app转发gpui/pdfium，更新ADR-001/CI；WindowsMSVC native compile通过，core依赖树无GPUI。
3. **I-3 导入/讲者**：启动config/topology、CLI/dialog/drop/最近文件import调用05metadata；Load→Ready占位，无观众计时；坏文件在讲者提示、旧状态保留。
4. **I-4 窗口事务**：实现create-Curtain/Start-Live/Stop-close、失效/零屏/单屏显式模式；注入创建失败测试不留Live/计时窗口。
5. **I-5 动作/角色**：全局routing、Escape模态/preview优先消费、弹UI讲者activate、冲突按键禁用、Swap权威binding事务、save dirty提示、03事件拔屏停止；无独立swap flag。
6. **I-6 Gate B实机**：02配置A Audience/B Presenter/C None→开始/结束/拔线/换C复测；混DPI/旋转/单屏/零屏；占位画面即可，焦点/Identify UI留09。记录环境/操作/截图/代码版本，报告实施后创建。
7. **I-7 评审**：only-snapshot view、read-only导入、错误/焦点、featureCI与Gate B；不能把占位称真实PDF或设置UI完成。

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
