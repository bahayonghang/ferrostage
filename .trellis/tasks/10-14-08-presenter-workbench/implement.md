# Implement — 08

> Current acceptance policy (2026-10-15 user revision): `.trellis/tasks/10-14-presenter-v0.3/acceptance-policy.md` supersedes older hardware/matrix blockers below. Implement all features first; retain automatic/feature/native-functional checks. Physical, exhaustive native/size and performance/soak matrices are deferred, not PASS. Gate B implementation is satisfied by V16;08 may start. Gate C/D are functional integration gates. Historical results and product contracts remain unchanged.

1. **I-1 门/范围审阅**：Gate B与05实际native/06models通过；审父design §6、P0/P1责任，核GPUI上传/drop/Present测量API，确立受控snapshot入口。
2. **I-2 当前/next/notes**：替换07 Shell importer：05 metadata/结构→06 builder→core map/crop→Ready，完整LoadedDocument首Load由08 app接线；实现workbench/P0布局、06裁剪/多notes、05P0/P1请求；notescache与audience结果权限隔离，测试key/generation/page/crop/DPI。
3. **I-3 投影/纹理**：组装完整snapshot/只current BGRA，新页未就绪保旧完整投影；Curtain/blank覆盖，texture RAII替换/淘汰/关闭/代际/DPI计数；错键/notes不能发布。
4. **I-4 时间/导航**：elapsed显示/阈值/clock/progress，Ready冻结/restart；文档菜单layout/notes识别/physical_pages/诊断→06重建→core ReconfigureDocument同generation/同physicalcount原子更新并保留timer/stage、清preview；content导航/preview10页/Enter/Esc、黑白屏/07显示器设置入口/旧交换快捷键兼容/focus，用回归验证观众直到Commit不变。
5. **I-5 最小指针/基本画笔与M1汇报**：normalized矩形两窗变换、core Begin/Append/End基本会话画笔（M1汇报含画笔）、P0功能全路径；≥80页/≥30min真实PDF含notes/overlay、开始/停止/拔线，记录Gate C，07最小分配面板路径不是09完整设置路径。
6. **I-6 P1增强**：Erase/highlighter与基本会话笔画增强、拖分割/1–16网格/虚拟滚动/debug；非法点/边框忽略/跨页笔画、保存dirty均测。M2前完整，不宣称M1全部P1已完成。
7. **I-7 性能/隔离评审**：按design200翻页/60s laser/200VRAM/阻塞2s、30次warm启动、50次Start、30s静止CPU及1000页grid滚动协议和截图偏差；actual Present measurement先核可用；只标已验证feature-parity条目，跑命令。

命令：cargo test -p ferrostage-core -p ferrostage-ui -p ferrostage-render；cargo build -p ferrostage-app --features gpui,pdfium；just ci（并保留05native job）。
pure测试/GPUI编译不能替代Windows真实画面/GPU报告。回滚组装器/annotation类型需同批UI/core；raster/upload失败保旧完整投影不泄备注。初次规划未执行；当前按父execution-state.md和用户连续实施授权恢复。

## 主题执行补充（2026-10-14）

在I-2/I-6/I-7中先核DESIGN.md与07主题层所有权，实施对应token/布局/设置消费者后保存原生截图、对比度结果与键盘/状态检查。无运行窗口时记NOT_RUN，保持原有Gate依赖。

## 2026-10-14 依赖审计 checkpoint

07产品外壳未交付且Gate B未通过；不能进入真实双窗集成。 本任务保持planning，未start、未写本任务产品代码、未运行本任务验收。现有规划保留；恢复前先核父execution-state.md与07依赖门报告。03物理拔插/换接口本轮不可用，不能用模拟输入替代Gate B。无提交或归档。


## Resume audit - 2026-10-15

Task07 shell/display assignment is implemented and V16 scoped native checks are verified; the old statement that07 product source was absent is historical. Gate B remains INCOMPLETE, so I-1/real dual-window integration is blocked. Current/next/notes pixels and workbench remain unimplemented. See parent `research/resume-2026-10-15.md` and07 `research/resume-2026-10-15-v16-review.md` for current-code evidence and exact remaining gates. Task status and acceptance checkboxes are unchanged. No commit, push or archive.
