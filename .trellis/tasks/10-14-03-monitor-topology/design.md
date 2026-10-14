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
