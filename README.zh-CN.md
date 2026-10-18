# FerroStage

PDF & Beamer presentation stage。基于 Rust 与 GPUI 的 PDF / Beamer 演示舞台。Windows 11 优先，离线，无遥测。

可选桌面构建已接入讲者工作台、单线程 PDFium 光栅、完整设置、外部 PDF 热重载、GIF/WebP 和隔离进程中的原生媒体播放。默认工作区检查不依赖原生 SDK。当前实施与分项验证见[功能对照](docs/feature-parity.md)；完整硬件、视觉矩阵和性能验收单独记录。

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
just ci       # 默认 features：fmt、clippy、test、cargo deny
just dev      # 把已校验的 PDFium 放到 target/debug，再以 gpui,pdfium 运行
just build    # 发布 gpui,pdfium；双击 target\release\ferrostage.exe
just install  # 安装 gpui,pdfium，并只把 pdfium.dll 放到 Cargo 的 bin
```

工具链：Rust 1.99.0，edition 2024。

`just ci` 保持默认 features。`just dev`、`just build`、`just install` 启用 `gpui` 与 `pdfium`，并放置哈希校验过的 `pdfium.dll`。来源、通知文件和准备命令见[打包说明](docs/packaging.md)。`just dev` 与 `just build` 同时放置 `LICENSE` 和 `licenses/`。`just install` 只放置 DLL，完整通知文件留在 `target\release`。应用进程不下载 PDFium，也不写用户 PDF。

双击产物是 `just build` 之后的 `target\release\ferrostage.exe`。该配方最后才链接带 feature 的程序。视频仍要单独准备媒体助手和运行库，构建命令是 `cargo build -p ferrostage-app --features gpui,pdfium,media --locked`。

`just dev` 保留控制台。发布版可执行文件使用 Windows GUI 子系统，资源管理器打开 `target\release\ferrostage.exe` 时不分配控制台。`just dev -- check-config <path>` 在终端打印只读的设置解析结果。交互式的 release `check-config` 没有控制台。把 release `check-config` 的 stdout 重定向后，解析结果仍会写出。

GPUI 使用基于 Zed 标签 `v1.23.3` 的已审维护源（ADR-008），不使用 crates.io 上的 `gpui` 0.2.x。唯一原生媒体后端为独立助手中的 GStreamer（ADR-009）。

## 许可

MIT OR Apache-2.0。这是对演示行为的净室实现，不移植 Pympress 源码。`cargo deny` 拒绝 GPL / AGPL。

设计说明见 [docs/PROJECT_PLAN.md](docs/PROJECT_PLAN.md)。
