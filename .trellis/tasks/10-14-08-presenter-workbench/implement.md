# Implement — 08

1. **I-1 门/范围审阅**：Gate B与05实际native/06models通过；审父design §6、P0/P1责任，核GPUI上传/drop/Present测量API，确立受控snapshot入口。
2. **I-2 当前/next/notes**：替换07 Shell importer：05 metadata/结构→06 builder→core map/crop→Ready，完整LoadedDocument首Load由08 app接线；实现workbench/P0布局、06裁剪/多notes、05P0/P1请求；notescache与audience结果权限隔离，测试key/generation/page/crop/DPI。
3. **I-3 投影/纹理**：组装完整snapshot/只current BGRA，新页未就绪保旧完整投影；Curtain/blank覆盖，texture RAII替换/淘汰/关闭/代际/DPI计数；错键/notes不能发布。
4. **I-4 时间/导航**：elapsed显示/阈值/clock/progress，Ready冻结/restart；文档菜单layout/notes识别/physical_pages/诊断→06重建→08 core ApplyDocumentView同代更新保留timer/stage；content导航/preview10页/Enter/Esc、黑白屏/交换/focus，用回归验证观众直到Commit不变。
5. **I-5 最小指针/基本画笔与M1汇报**：normalized矩形两窗变换、core Begin/Append/End基本会话画笔（M1汇报含画笔）、P0功能全路径；≥80页/≥30min真实PDF含notes/overlay、开始/停止/拔线，记录Gate C，配置角色路径不是09UI路径。
6. **I-6 P1增强**：Erase/highlighter与基本会话笔画增强、拖分割/1–16网格/虚拟滚动/debug；非法点/边框忽略/跨页笔画、保存dirty均测。M2前完整，不宣称M1全部P1已完成。
7. **I-7 性能/隔离评审**：按design200翻页/60s laser/200VRAM/阻塞2s、30次warm启动、50次Start、30s静止CPU及1000页grid滚动协议和截图偏差；actual Present measurement先核可用；只标已验证feature-parity条目，跑命令。

命令：cargo test -p ferrostage-core -p ferrostage-ui -p ferrostage-render；cargo build -p ferrostage-app --features gpui,pdfium；just ci（并保留05native job）。
pure测试/GPUI编译不能替代Windows真实画面/GPU报告。回滚组装器/annotation类型需同批UI/core；raster/upload失败保旧完整投影不泄备注。本次不执行。
