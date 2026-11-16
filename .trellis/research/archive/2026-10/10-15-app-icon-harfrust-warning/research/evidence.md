# Evidence

检查日期：2026-10-15。只记录当时读到的代码，未改产品文件。

## Window icon

- 截图标题栏是 Windows 默认图标。
- `assets/icons/README.md`：应用图标不在骨架里，打包时再放 Windows `.ico`。
- `vendor/gpui/src/platform.rs:2294`：`WindowOptions.icon` 注释为 x11 only。
- `vendor/gpui/build.rs` 的 `embed_resource()` 只在 feature `windows-manifest` 下嵌入 `resources/windows/gpui.rc`。该 rc 只有 `1 RT_MANIFEST`，没有 ICON。
- 工作区 GPUI 依赖是 `default-features = false`（根 `Cargo.toml`）。`ferrostage-ui` 的 `gpui` feature 没有打开 `windows-manifest`。
- `crates/ferrostage-app` 没有 `build.rs`。
- Zed checkout `96a1b1b`，`crates/gpui_windows/src/platform.rs:1470`：`load_icon()` 对当前模块 `LoadImageW`，资源 ID 1，类型 `IMAGE_ICON`。失败则空 `HICON`。
- 同文件 `window.rs:1419` 把该 handle 放进 `WNDCLASSW.hIcon`。
- `embed-resource` 3.0.12，registry checksum `6168487b5e7eb2673e6efe1200b1848016e436a6ca0b8031bf04bd9d493a249f`，`Cargo.toml` license `MIT`。`compile(...).manifest_optional()` 的文档示例就是嵌入图标。

## shape warning

用户命令：

```text
cargo run -p ferrostage-app --features gpui,pdfium
warning: function `shape` is never used
   --> vendor\harfrust\src\hb\face.rs:439:8
```

- `vendor/harfrust/src/lib.rs:13`：`mod hb` 私有。
- `lib.rs:27-28`：`pub use hb::face::shape` 包在 `experimental_font_api` 里。
- `Cargo.toml` default features 只有 `std`。`cosmic-text` 只启用 `harfrust/std`。全仓库 `.rs` 里对 `face::shape` 的引用只有这次 `pub use`。
- `face.rs:543` 是 `impl` 上的 `pub fn shape`，与自由函数不是同一个项。
- 因此产品构建里自由函数是私有模块中的未再导出项，`dead_code` 成立。
- `.trellis/spec/frontend/maintained-dependencies.md`：不允许新的 lint allow；既有六条 HarfRust strict lint 保持记录，不在本任务消除。
