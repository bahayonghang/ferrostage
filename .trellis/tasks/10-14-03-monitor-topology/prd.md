# 显示器拓扑识别与角色解析（解决三屏问题）

父任务：`10-14-presenter-v0.3`（R-20 R-21 R-22 R-23）。复杂任务，设计和步骤见本目录design.md / implement.md。

## Goal

讲者在三块（或更多）显示器的环境里能明确指定：哪块屏投影给观众、哪块屏是讲者工作台、哪些屏闲置；这个指定跨重启、跨接口顺序变化稳定；屏幕热插拔时行为可预测且不泄露备注。

## Background

- 现状 `ferrostage-platform` 只有热重载常量（`crates/ferrostage-platform/src/lib.rs`）；没有显示器相关代码。
- 现状设置用 `Option<u32>` 索引（`crates/ferrostage-settings/src/lib.rs:15-16`），Windows 的显示器枚举顺序会随接口与驱动变化。
- 规则来源：`docs/PROJECT_PLAN.md` §4.3；共享类型见父任务 `design.md` §3。
- workspace unsafe_code=forbid；局部allow不能覆盖。先研究安全wrapper；不足时提出最小模块/lint继承/ADR政策提案评审，不能直接放宽。

## Requirements

- **M-1** enumerate_monitors返回Result<Vec<MonitorInfo>,PlatformError>，含身份来源、native_size、scale、物理position和primary。零屏正常空列表，OS错误不panic或伪造主屏。
- **M-2** 03在core display模块创建MonitorId，复用02 MonitorRole。设备路径/EDID优先、名称/分辨率退化标级别；不承诺跨接口必不变，同退化ID视歧义并禁止自动绑定Audience。
- **M-3** 纯resolve_roles遵循父design §3；presenter为Option、零屏None。无保存才启发式Audience；保存缺席/显式None不接管。重复角色按配置首次有效binding处理，余项None+warnings，面积同值按ID排序。先每ID唯一role，Audience候选排除resolved Presenter；保存Audience先定、Presenter回退排除该ID；单屏固定presenter本屏/audience=None。
- **M-4** 统一source与warnings：NoMonitors/SavedAudienceMissing/DuplicateAudience/DuplicatePresenter/DuplicateBinding/PresenterMissingFallbackPrimary/AmbiguousMonitorId；UI不另造解析策略。
- **M-5** `watch_topology(sink)`：Windows 监听 `WM_DISPLAYCHANGE`（独立安全OS事件源由03实现；安全事件API不可用时以200ms安全枚举轮询作为明确fallback，不等待07；07可补GPUI事件触发重新枚举）后重新枚举并发送 `TopologyEvent::Changed(Vec<MonitorInfo>)`；防抖 200 ms。
- **M-6** 提供 `diff(prev, next) -> TopologyDiff { added, removed }`，供 `07` 判断 Audience 屏是否消失（不变量 3）。
- **M-7** 不在本 crate 内创建窗口或调用 GPUI；`ferrostage-core` 不得依赖本 crate。

## Acceptance Criteria

- [ ] 纯函数单测覆盖：零屏/身份碰撞/重复Presenter；1 屏；2 屏无保存；3 屏无保存（Audience 选面积最大非主屏）；3 屏有保存且全部在场；保存的 Audience 缺席；两块保存为 Audience；保存的 Presenter 缺席（回退主屏并告警）。
- [ ] 三屏连续枚举/换接口/同型号/缺EDID记录ID等级与恢复；变化/碰撞告警且不接管Audience，实施时生成stability报告。
- [ ] 身份来源等级真实可见，无强身份API时标退化，不虚标edid。
- [ ] `watch_topology` 在拔/插屏后 ≤ 1 s 内发出事件（手工验证）。
- [ ] 安全wrapper exact版本/许可/API证据；确需项目unsafe先审最小政策提案，不以allow绕过forbid。
- [ ] `just ci` 全绿。

## Dependencies

完整前置：01/02（具体目录见父Task Map）。父子关联不表示依赖。

## Out of Scope

GPUI `DisplayId` 映射与窗口创建（`07`）；设置界面排布图与 Identify 覆盖层（`09`）；多 Audience 镜像。
