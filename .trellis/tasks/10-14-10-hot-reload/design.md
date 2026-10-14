# Design — 外部 PDF 替换事务

## 选择、依赖、输入输出
完整前置01/04/05/06/07/08与Gate C；H-1…8/R-01/02/44。09不是hard依赖，设置可由配置指定；若UI存在由本任务启用09的对应控件。
输入target path/notify events/settings；输出attempt/candidate owned metadata/frame和主线程新generation事务。用户PDF只读，测试覆盖写入只在独立temp目录。

## 文件范围
现有platform/src/lib.rs/Cargo.toml、pdf/render/document/core/ui各src/lib.rs及05worker/06builder/07app协调器/08projection、core command/state、app/main.rs、09settings capability映射（若存在）、ADR-004。
拟建platform/src/reload.rs、app/ui reload coordinator及各crate就近integration tests。SHADOW_TMP_SUFFIX删掉及其误导注释/测试，不挪给settings制造第二tmp策略。

## 执行线程/数据流
notify监听父目录过滤文件（支持tmp rename），300ms静默事件合并；IO worker两次大小/mtime间隔采样（≥100ms）+尾1KiB EOF提示，read完整bytes关句柄后复核stat，30s未就绪debug放弃本轮。稳定+EOF不保证valid。
structure worker用lopdf解析媒体/结构；唯一render worker PrepareCandidate先PDFiumload/label/dimensions并返回owned metadata；06 builder合并metadata/structure建图/定位/crop，再worker PreRenderCandidate(P0/P1 requests)等待ready，所有PDFium对象仍在worker。active候选同时存在，由05两命名空间服务，候选结果不触发current-only丢弃。
AttemptToken随新事件/Close/新Load递增；ready结果先验证token/desired generation/仍同path。标签定位唯一组+ordinal，否则旧physical号钳到新物理范围，选≤该号最大content物理页，无则首content，不作content Vec下标，不掉notes；可映射页笔画保留、删页丢弃diagnostic，timer/stage不变。
提交主线程：prepare完整descriptor/document/visual/纹理→可失败步骤完成→同次发布新状态/owned document/snapshot→发worker CommitCandidate→旧UI/CPU缓存释放→worker Retire旧PDFium ack。Commit设计需worker保证已验证candidate不会丢失；commit/retire错误typed记录并重试有界退役，不能让主线程显示未加载document。提交前错误Abort/旧代继续；窗口/关闭导致token过期也Abort。
11尚未实现时generation失效事件无op，11接后先停旧媒体；不引入对11的循环依赖。计时只读04now无重启。watcher运行中开关重建并取消旧attempt，debounce动态更新。

## 错误与回滚
半写/坏PDF重试仅日志，不弹窗/不黑屏；旧服务不提前清缓存。并发候选只保最新，过时native文档在worker释放。资源回收计数/ack验证而非凭generation增加宣称完成。swap/focus/window/错误由07/08协调，重载不直接创建第二窗口。
回滚候选的bytes/metadata/frame/纹理在各线程释放；主线程发布后旧代只作退役不可重复提交。无任何以target path写方式打开的调用，fixture harness路径强制temp前缀。
