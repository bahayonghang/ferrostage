# FerroStage

PDF & Beamer presentation stage。基于 Rust 与 GPUI 的 PDF / Beamer 演示舞台。Windows 11 优先，离线，无遥测。

当前仓库是可编译骨架：状态机、设置、命令行和许可证门禁已经就位。演示窗口、PDFium 光栅和媒体播放尚未接入。

## 计划纳入 v1.0

- 打开 PDF（命令行、对话框、拖放、最近文件）；导入后处于就绪态，讲者显式开始。
- 讲者/观众双窗口分离；按显示器角色投影；一键交换；黑/白屏；全屏。
- Beamer：同页左右/上下拼合备注、`notes:` 独立备注页、`\pause` overlay 步进、PageLabel 逻辑页码。
- 讲者工作台：当前页、下一页、备注、计时器（正计/倒计/预警）、时钟、跳页预览、1–16 页网格缩略图。
- 归一化矢量激光笔与手绘批注（会话内存，不写文件）。
- 完整设置系统与设置界面；显示器识别与角色分配。
- PDF 文件被替换时的通用热重载（P2）。
- GIF 与离屏视频播放（P2）。

## 不纳入 v1.0

任何 LaTeX 源码或编译工具链集成；PDF 注释编辑或写回（只读媒体结构发现保留）；排练统计导出；Beamer `animate`；云协作、账号、遥测；PPTX。

## GitHub topics

建议的仓库 topics 保留 `beamer`，并移除 `latex`（若已设置）。`beamer` 指消费 Beamer 导出的 PDF，不是 LaTeX 工具链。

## 命令

```text
just ci
just build
just dev
just install
```

工具链：Rust 1.99.0，edition 2024。

GPUI 固定到 Zed 标签 `v1.23.3`（或 `gpui-kit` 0.7.3），不使用 crates.io 上的 `gpui` 0.2.x。

## 许可

MIT OR Apache-2.0。这是对演示行为的净室实现，不移植 Pympress 源码。`cargo deny` 拒绝 GPL / AGPL。

设计说明见 [docs/PROJECT_PLAN.md](docs/PROJECT_PLAN.md)。
