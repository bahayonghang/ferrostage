# Design — 生命周期与投影纯类型

## 选择、依赖、输入输出
前置01，L-1…11/R-10…14/R-43预览部分。core创建stage、timer纯计算、BgraFrame/AudienceVisual骨架；不创建MonitorRole/Id。输入PresentationCommand与app传入单调now，输出Result和audience projection；core不自行访问clock/OS。document消费者同次改。
Start每次新演讲清零，Stop冻结elapsed，Ready沿同timer.mode显示停止值（正计elapsed、倒计target-elapsed），不跳变；Close/Load清零。new(page_count)为Ready，生产idle()+Load；Ready接受定位/画笔/blank但Curtain，不显示visual。

## 文件范围
现有core/src/{lib,state,command,snapshot,geometry,error}.rs与document/src/lib.rs；拟建core/src/{session,timer,frame}.rs（仅确需拆分时），就近状态/属性测试。不改settings或OS类型。01已建ADR-006/ADR-003由此确认具体契约。

## 设计
合法边完整定义：Idle Load(nonempty)→Ready；Ready Load→Ready/Start(valid)→Presenting；Presenting Stop→Ready；Close任意→Idle。非法Start/Stop/Load/计时命令返回typed CoreError且状态不变。Idle导航等拒绝。
TimerState仅记起点/accumulated/pause，apply_at与elapsed_at对注入Instant纯计算；Reset清零保留pause，重复Pause/Resume定为幂等（仅Presenting有效）。Load失败保留旧状态。Start窗口成功与否由07协调，不由core枚举。
JumpPreview与已提交页分离；preview中的Step只移preview，Commit才修改current，Cancel清预览；snapshot恒已提交页。先写能抓住现state.rs泄漏/隐式提交的回归。
投影包含stage/gen/page/crop/blank/laser/strokes及visual可选；core纯state产出visual=None，08才组装。Curtain不带视觉。BgraFrame构造检查nonzero/checked geometry/stride/len，AudienceVisual媒体元素仅slot/rect/pixels；没有notes/path/clock/Presenter cache。
DocumentDescriptor先只gen/page_count，06扩SlideMap，10替换旧bytes提交载荷；Strict newer验收保留，core不留bytes。SwapScreens移除screens_swapped独立权威，作为app意图通过受控dispatch处理，直接core apply不再翻flag；页/crop不变。消费者/既有测试必须明确新语义，不暗保旧演讲态构造。

## 错误、回滚
空文档/非法stage/无Audience/陈旧generation/非法frame均typed error，不半更新状态。Bgra含opaque像素而不增IO依赖。回滚共享snapshot/command API同时回滚document；不能让两种stage行为并存。
