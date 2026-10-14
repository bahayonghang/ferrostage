# FerroStage

基于 Rust 与 GPUI 的 PDF / Beamer 演示舞台。Windows 11 优先，离线，无遥测。

当前仓库是可编译骨架：状态机、设置、命令行和许可证门禁已经就位。演示窗口、PDFium 光栅和媒体播放尚未接入。

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
