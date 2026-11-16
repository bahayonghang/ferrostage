# 应用图标与 harfrust shape 警告

## Boundaries

图标只进入 `ferrostage.exe` 的 Windows 资源。GPUI 窗口选项和 `gpui_windows` 保持不动。`load_icon()` 已经按资源 ID 1 读取 exe 图标；本任务补上这枚资源。

`shape` 警告只改 `vendor/harfrust/src/hb/face.rs` 一处属性。`Shaper::shape`、format2 修补和产品 feature 集合不动。

## Icon

生成一张 1:1 Q 版小熊猫，然后提交两份资产：

| 文件 | 作用 |
| --- | --- |
| `assets/icons/ferrostage.png` | 源图 |
| `assets/icons/ferrostage.ico` | 16 / 32 / 48 / 256，供资源编译器使用 |

画面：正面、大头、小身体、圆耳、深色眼罩。毛色 `#A34F35`，吻部 `#FAF9F5`，眼罩与瞳孔 `#141413`，底用 `#181715` 或透明。不写文字，不用螃蟹造型。

`crates/ferrostage-app/resources/windows/ferrostage.rc`：

```rc
1 ICON "ferrostage.ico"
```

rc 旁放 `ferrostage.ico` 的副本，或让 rc 用相对路径指向 `assets/icons/ferrostage.ico`。资源编译器的工作目录不稳定，实施时用 `ParamsIncludeDirs` 指向 `assets/icons`，rc 里只写文件名。源文件仍以 `assets/icons/` 为准，避免两份图标内容分叉；若编译器必须看到 rc 同目录文件，构建脚本复制过去，并把复制结果视为生成物，不另提交。

`crates/ferrostage-app/build.rs` 仅在 `CARGO_CFG_TARGET_OS=windows` 时调用：

```rust
embed_resource::compile(
    "resources/windows/ferrostage.rc",
    embed_resource::ParamsIncludeDirs(["../../assets/icons"]),
)
.manifest_optional()
.unwrap();
```

`../../assets/icons` 相对 `crates/ferrostage-app`（build script 的工作目录）。rc 里只写 `ferrostage.ico`。`embed-resource` 3.0.12 文档把 `manifest_optional()` 用于图标这类非必须 manifest。`ferrostage-app` 是 bin，rustc 1.50 之后资源链进二进制。`GetModuleHandleW(None)` 因此能读到 ID 1。

`crates/ferrostage-app/Cargo.toml` 增加：

```toml
[build-dependencies]
embed-resource = "=3.0.12"
```

版本与锁文件里的现有条目相同。MIT 已在 `deny.toml` allow 列表。不启用 GPUI 的 `windows-manifest`，避免把另一份 manifest 策略卷进来。

## Warning

`lib.rs` 已有：

```rust
#[cfg(feature = "experimental_font_api")]
pub use hb::face::shape;
```

自由函数改为同一条件：

```rust
#[cfg(feature = "experimental_font_api")]
pub fn shape(
```

属性放在文档注释之后、`pub fn` 之前。feature 关闭时整个项不参与编译，`dead_code` 消失。feature 打开时再导出仍然指向这个函数。

`hb` 是私有模块，外部不能绕过 `harfrust::shape` 调用它。仓库内唯一引用是上述 `pub use`。

## Docs

ADR-008 追加一段当天边界：批准的只是这一处 `cfg`，不扩大 lint 例外，不改排版。`vendor/README.md` 的 harfrust 句子补上第三处源码改动的范围。`maintained-dependencies.md` 的 “两处 format2 文件” 合同保留，并加上这条独立的条件编译边界。

## Rollback

删掉 `build.rs`、rc、`[build-dependencies]` 和 `assets/icons` 下两个新文件，标题栏回到系统默认图标。去掉 `face.rs` 的 `cfg` 后，警告回到当前行为。文档三处同步回退。

## Trade-off

把 `experimental_font_api` 开到产品依赖上也能让 `pub use` 把函数标成已使用。那会把实验 API 带进产品图，只为消掉一条警告。条件编译与现有再导出对齐，产品图不变。
