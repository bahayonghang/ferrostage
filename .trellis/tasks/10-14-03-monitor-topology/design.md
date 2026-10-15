# Design — 显示器拓扑

## 选择、依赖、输入输出
前置01/02，M-1…7/R-20…23；复用02 core MonitorRole，本任务在core新增MonitorId。OS查询输出Result<Vec<MonitorInfo>,PlatformError>；纯resolve_roles输入saved bindings和Info数组，输出父design §3的Optional presenter/audience、source/warnings/ignored；diff输出added/removed与身份歧义。

## 文件范围
现有platform/src/lib.rs及Cargo.toml、core/src/lib.rs、02拟建core/src/display.rs；Cargo.toml仅必要安全wrapper依赖，docs/adr/ADR-007-monitor-roles.md（01交付后更新）。
拟建platform/src/{monitor,topology}.rs与受限平台适配模块；纯函数tests就近，实际Windows测试报告后建。settings只消费binding，不重复造类型或转换规则。

## 设计
Win11首先评估安全wrapper：display-info可证明几何接口，尚不能证明完整EDID/device-path/events。必须核对exact版本/许可/字段；不足时形成最小FFI隔离与lint/ADR政策提案，不写局部allow。非Windows接口无实现时typed Unsupported/可枚举最小信息，不伪造硬件。
ID将stable confidence与current connection分开；path/EDID优先，name|WxH/GDI退化显式标等级，完全相同退化键为AmbiguousMonitorId。不能把device-interface path当换接口永不变承诺。normalize重复ID不准靠添加枚举序号恢复Audience。
纯解析规则唯一来自父design：零屏presenter/audience=None；单屏默认只讲者；无保存启发式最大非主屏；已有保存Audience缺席/None禁止启发式。同ID重复binding先按配置首项规范化唯一role并DuplicateBinding；重复roles再依配置首有效，其余ignored+warnings；无保存Audience候选排除已解析Presenter，有保存Presenter回退排除Audience。单屏输出presenter本屏/audience=None。Presenter缺席主屏回退只对讲者，Audience不回退。
watch_topology输出Changed owned Info；200ms防抖为事件合并目标，03负责独立安全OS事件源，若安全事件API不足则200ms安全枚举轮询作为明确fallback；07仅补GPUI事件触发，禁止另造隐藏子窗口监听策略。≤1s拔线观众关闭由07验收，本任务只负责事件/diff。伪时钟事件源可测但不替代实机。

## 错误、回滚
OS失败保留最后可信拓扑但Audience availability置不可用并警告，不从旧索引继续投影；空数组不是OS错误。role warnings是数据，不在platform弹UI。identity歧义/无法唯一匹配进入讲者only。回滚平台适配不影响core纯测试；类型变更须同步07/09使用计划。政策门未通过不修改workspace lint。

## Accepted implementation API (2026-10-14)

- Windows-only exact dependency: winsafe `=0.0.29` (`gdi`, includes user/kernel, MIT, MSRV 1.87). Coordinator reviewed the final single-wrapper implementation. The initially tried display-info combination failed cargo deny (fxhash advisory) and was removed without a policy exception. No workspace lint change, raw project FFI or window.
- `MonitorId(pub String)` is in core; `resolve_roles(&[MonitorInfo], &[(MonitorId, MonitorRole)]) -> ResolvedTopology`. App explicitly maps settings string IDs; platform does not depend on settings.
- `MonitorInfo` owns id/name/connection_name/native_size/position/scale_factor/scale_source/rotation_degrees/is_primary/identity_source/identity_confidence. Physical pixels are DEVMODE units. Device interface path = ConnectionScoped, name+resolution = Degraded. GDI connection_name is current mapping data, never persistent identity.
- winsafe copies every EnumDisplayDevices result before advancing its borrowed-buffer iterator. Complete attached non-mirroring adapters and active monitor devices must correspond one-to-one with EnumDisplayMonitors/EnumDisplaySettings geometry; missing/duplicate GDI names, invalid geometry, inconsistent primary, or an identity change across the two device queries return typed errors. No partial success.
- ScaleSource::GdiDeviceCapsRatio records per-monitor DESKTOPHORZRES/HORZRES in the current DPI context. HWND::NULL.GetDC gives a RAII desktop DC; EnumDisplayMonitors receives it and supplies per-monitor DCs. Physical cap width must equal DEVMODE width; dimensions/required fields/rotation and complete GDI sets are checked. This ratio is not measured effective DPI, and 1.0 in an aware context does not imply 100% desktop scaling. Enumeration does not change process awareness. 07 validates in the actual GPUI DPI context.
- `watch_topology(Sender<TopologyEvent>) -> Result<TopologyWatcher, PlatformError>` uses a named worker, 200 ms polling and 200 ms stable-change coalescing. Initial topology is immediate. Changed owns a complete list; Unavailable owns typed error and diagnostic last_known. Errors are immediate, recovery publishes Changed even for the same geometry, and consumers must disable Audience/stop safely on Unavailable. Explicit `stop()` and Drop cancel the wait and join; an ongoing synchronous OS call cannot be interrupted. No native event window is created. Non-Windows returns UnsupportedPlatform.
- `diff` reports sorted added/removed IDs plus ambiguous next IDs; geometry-only changes still produce Changed. Any identity collision suppresses Audience, including heuristic selection.


## Screen-number presentation boundary

07 consumes the existing complete topology to show detected count, readable numbered entries and matching Identify overlays. UI numbers are never MonitorId, persisted role keys, or a claim about OS display numbering.03 continues to own identity, confidence, geometry and event truth; it must distinguish an empty successful enumeration from Unavailable. No new wrapper, identity scheme, index binding or changed role-resolution policy is required by this planning correction.09 reuses07's presentation mapping.
