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
