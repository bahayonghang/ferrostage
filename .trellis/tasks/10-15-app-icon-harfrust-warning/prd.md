# 应用图标与 harfrust shape 警告

## Goal

Windows 上的 FerroStage 使用一枚与现有界面一致的 Q 版小熊猫作为窗口图标。`just dev` 不再打印 `harfrust` 里未使用的自由函数 `shape` 的 `dead_code` 警告。已启用的排版行为保持不变。

## Background

用户在功能基本完成后要求继续做这两项。截图里的标题栏图标是 Windows 默认图标。`assets/icons/README.md` 预留了 Windows `.ico` 的位置，目录里目前只有这篇说明。

`vendor/gpui/src/platform.rs:2294` 的 `WindowOptions.icon` 只作用于 X11。Windows 图标由 Zed tag `v1.23.3` 的 `gpui_windows` 加载：`crates/gpui_windows/src/platform.rs:1470` 的 `load_icon()` 用 `LoadImageW` 读取当前 exe 的 `IMAGE_ICON` 资源 ID 1；失败则 `unwrap_or_default()`，窗口类得到空图标。`crates/ferrostage-app` 没有 `build.rs`，exe 里没有这枚资源。

`vendor/harfrust/src/hb/face.rs:439` 的 `pub fn shape` 只在 `vendor/harfrust/src/lib.rs:27-28` 于 `feature = "experimental_font_api"` 时再导出。`mod hb` 是私有模块。仓库内没有其他调用。产品构建未打开该 feature，所以 `cargo run -p ferrostage-app --features gpui,pdfium` 打出这条警告。同文件 `face.rs:543` 的 `Shaper::shape` 是另一个方法，产品排版走这条路径。

ADR-008 与 `.trellis/spec/frontend/maintained-dependencies.md` 不允许新增 lint allow。HarfRust 直接 strict lint 的既有六条诊断不在本任务里消除。`embed-resource` 3.0.12 已在 `Cargo.lock`，许可证是 MIT，属于 `deny.toml` 现有允许列表。

## Requirements

- **I-1** 动物定为 Q 版小熊猫。毛色用 `DESIGN.md` 的 primary `#A34F35`，吻部用 canvas `#FAF9F5`，眼罩用 ink `#141413`。正方形，无文字。16px 下仍能看出圆头、耳和大眼罩。
- **I-2** 源图放在 `assets/icons/ferrostage.png`。Windows 图标放在 `assets/icons/ferrostage.ico`，至少包含 16、32、48、256 四档。
- **I-3** `ferrostage.exe` 嵌入 `ICON` 资源 ID 1。讲者窗口标题栏和任务栏显示这枚图标。不修改 `gpui_windows`，不改 `WindowOptions.icon`。
- **I-4** 更新 `assets/icons/README.md`，写明文件名、资源 ID 和颜色来源。
- **W-1** `face.rs:439` 的自由函数 `shape` 与 `lib.rs:27-28` 使用同一 `experimental_font_api` 条件编译。feature 关闭时该函数不参与编译；feature 打开时 `harfrust::shape` 仍可调用。
- **W-2** 不添加 `allow(dead_code)` 或其他 lint allow。不删除该函数。不打开产品的 `experimental_font_api`。不改 `face.rs:543` 的 `Shaper::shape`，不改 `tables.rs` / `charmap.rs` 的 format2 修补。
- **W-3** 在 ADR-008、`vendor/README.md` 和 `.trellis/spec/frontend/maintained-dependencies.md` 各记一笔：这是第三处 HarfRust 源码改动，只做条件编译，不改变排版。

## Acceptance Criteria

- [ ] 重新 `cargo run -p ferrostage-app --features gpui,pdfium` 的输出里没有 `harfrust` `function shape is never used`。
- [ ] `cargo check -p harfrust --features experimental_font_api --locked --offline` 通过，且 `harfrust::shape` 仍由 `lib.rs` 再导出。
- [ ] 本机启动后的讲者窗口标题栏显示小熊猫图标，任务栏图标与之相同。观众窗口无标题栏，不另做品牌。
- [ ] `cargo clippy -p ferrostage-app -p ferrostage-ui --all-targets --features gpui,pdfium --locked --offline -- -D warnings` 通过。
- [ ] `cargo deny --all-features --locked --offline check` 通过。依赖图不新增 crate 版本；`embed-resource` 仍是 3.0.12。
- [ ] `git diff` 中没有针对这条警告的 `allow(dead_code)` / `#[allow(unused)]`。

## Out of Scope

讲者工作台里的 “FerroStage” 字标、设置页和观众画面。macOS / Linux 图标。安装包。HarfRust 既有六条 strict lint。format2、cmap、SVG 渲染和 GPUI 维护包的其他文件。
