# Design — GPUI 外壳与拓扑 PoC

## 选择、依赖、输入输出
前置01/02/03/04/05，Gate A纯契约；G-1…11/R-13/21/23/42。05仅供只读import metadata，Gate B允许占位，无06/08真实画面要求。02配置角色，不宣称09 UI可用。app组合settings/topology/import/core，ui封装所有GPUI调用；输入owned metadata/配置/事件与命令，输出窗口事务/快照/讲者告警。

## 文件范围
现有ui/src/lib.rs与Cargo.toml、app/src/main.rs与Cargo.toml、platform/src/lib.rs及03拟建topology模块（事件桥所需）、Cargo.toml、.github/workflows/ci.yml、ADR-001。
拟建ui/src/{shell,presenter,audience,actions}.rs、app运行协调模块（main过大才拆）。不直接读lopdf，05 probe供导入，不实作工作台/设置页。

## 研究与设计
优先Zed tag v1.23.3已允许路线，先spike Windows build/fullscreen/事件/keystroke；gpui-kit0.7.3为已允许替代但必须记录具体路线，不能混引两套GPUI。native DisplayId↔MonitorId桥优先，GPUI bounds为Pixels而OS为物理桌面，混DPI/旋转转换需验证；多/零匹配拒绝Start，不凭index或第一矩形。
启动load config→enumerate/resolve→讲者window（无屏等待，不unwrap）→CLI/dialog/drop经可替换DocumentImporter导入read-only probe→纯物理页descriptor→Load→Ready，占位显示页号；最近文件只属讲者设置控制侧。07仅Shell metadata，06 builder由08接到初次真实Load，不能把07称完整LoadedDocument导入。启动没有观众/计时。
WindowCoordinator处理Start：校验Ready/目标/单屏显式模式→创建初始Curtain窗口→成功Start并发布Live占位；失败清窗口保持Ready/计时不增长。Stop先Curtain→Stop→关闭；Close/拔Audience同路径，讲者窗口化，新屏不自动接管。
Action dispatcher是所有入口：Escape优先关闭当前模态/设置或CancelJump并消费，只有无模态/preview才匹配Stop绑定；导航等送core apply_at，Swap/设置/打开先由app协调；对话框/输入弹出前activate讲者。非法/冲突绑定禁用冲突项并显示警告，不能多个动作同时执行；录制和规范语法给09复用。
Swap通过修改display.monitors→resolve→窗口迁移→save同一协调器；单屏/缺Audience拒绝，core无独立flag。运行时迁移失败回滚role/window，save失败保留实际会话role并标未保存。热插拔由03 owned事件，零屏/歧义及时Curtain安全停止。
AudienceView只snapshot Entity，Live占位也不接LoadedDocument；Curtain全黑无focus/UI。gpui feature app转发，nativebuild/clippy与pure CI分开，不在headless suite启动窗口。

## 错误和回滚
import/窗口创建/映射/事件失败typed报告仅讲者；原doc在坏Load时不变。新观众创建失败不能残留Presenting timer。路线/library政策研究未通过不凭猜API写调用；07回滚UI/app/platform桥与feature/CI一起，core纯状态不引GPUI。
