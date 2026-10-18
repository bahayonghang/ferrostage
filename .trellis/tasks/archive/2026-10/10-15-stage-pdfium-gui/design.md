# 让 just dev 和 just build 直接打开可导入 PDF 的窗口

## Boundaries

加载契约不变。`RenderHandle::spawn` 仍接收一个 DLL 文件路径。未传 `--pdfium-library` 时，路径仍是 `current_exe()` 旁边的 `pdfium.dll`。相对路径仍先 `canonicalize`。应用进程不发起网络请求。

`just ci` 继续使用各 crate 的 default features。`ferrostage-app` 的 default 保持空。桌面功能只从 `just dev`、`just build`、`just install` 的显式 feature 列表进入。

媒体运行库保持 `docs/packaging.md` 里的独立准备步骤。这次三条例程都不传 `media`。

## Recipe order

`scripts/stage-pdfium.ps1 -Destination <dir> [-DllOnly]`：

1. 读取仓库根的 `docs/pdfium-runtime.json`。
2. 缓存目录固定为 `cache/pdfium-7881-win-x64`。`bin/pdfium.dll` 缺失或 SHA256 与 `dll_sha256` 不同时，调用现有 `scripts/fetch-pdfium.ps1 -Destination <缓存目录>`。哈希已匹配则不下载、不重新解压。
3. 把 `bin/pdfium.dll` 复制到 `<dir>\pdfium.dll`，再校验目标哈希。
4. 无 `-DllOnly` 时，另把缓存中的 `LICENSE` 和整个 `licenses/` 复制到 `<dir>`。不复制 `include/`、`lib/`、压缩包或 SDK 工具。
5. 目标哈希不符或任一 notice 哈希不符时失败，不启动应用。

`just dev *args` 先把运行库放到 `target/debug`，再执行：

```text
cargo run -p ferrostage-app --features gpui,pdfium --locked -- {{args}}
```

`just build` 的最后一步必须是带 feature 的 app 链接，避免 `cargo build --workspace --release` 把 exe 覆盖回骨架程序：

```text
cargo build --workspace --release --locked --exclude ferrostage-app
cargo build -p ferrostage-app --release --features gpui,pdfium --locked
scripts/stage-pdfium.ps1 -Destination target/release
```

`--exclude ferrostage-app` 仍构建其余 workspace 成员。单独再跑不带 feature 的 `cargo build --workspace --release` 会覆盖这个 exe；README 写明双击产物以 `just build` 的结果为准。

`just install`：

```text
cargo install --path crates/ferrostage-app --locked --features gpui,pdfium
scripts/stage-pdfium.ps1 -Destination <CARGO_HOME>\bin -DllOnly
```

`CARGO_HOME` 缺省时是 `%USERPROFILE%\.cargo`。只复制 DLL，因为 Cargo bin 目录不是通知文件的分发目录。完整通知文件留在 `target\release`。

## Windowed release binary

`crates/ferrostage-app/src/main.rs` 增加：

```rust
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
```

调试构建和 `cargo test` 默认编出来的 exe 仍是控制台子系统，`just dev` 的 tracing 留在当前终端。release exe 是 GUI 子系统，资源管理器不会为它分配控制台。不引入 `windows-sys`，不写 `unsafe`，因此不为 release 二进制附加父控制台。

由此带来的可见性：

| 启动方式 | 结果 |
| --- | --- |
| 资源管理器双击 release exe | 无控制台，窗口保持 |
| `just dev` | 当前终端可见日志，窗口打开 |
| 调试 `check-config` | stdout/stderr 可见，只读 |
| release `check-config`，stdout 被管道或重定向 | 文本仍写出，退出码不变 |
| release `check-config`，直接在交互终端里看屏幕 | 没有控制台输出 |

交互式 release 子命令不是这次的使用路径。需要看设置解析时用 `just dev -- check-config <path>`。

窗口出现前就退出的 release 失败（设置路径或 `open_for_startup` 返回错误）无法再靠闪一下的控制台看见。`start_app` 在这些 `ExitCode::from(1)` 路径上，于 `eprintln!` 之外把同一句追加到设置文件所在目录的 `startup-error.log`。设置路径还无法解析时，写到 exe 旁边。`check-config` 不进入这条路径。

## Docs

同步 `README.md`、`README.zh-CN.md`、`docs/packaging.md` 和 `justfile` 注释：

- `just dev` / `just build` / `just install` 启用 `gpui,pdfium`，并放置已校验 DLL。
- `just ci` 仍是默认 features。
- 应用进程不下载运行库。
- 视频仍要按打包说明单独准备 media runtime。
- 双击对象是 `just build` 之后的 `target\release\ferrostage.exe`。

## Compatibility

已有 `--pdfium-library <file>` 优先于旁边的 DLL。测试用的 `FERROSTAGE_TEST_PDFIUM` 仍只属于测试夹具，不成为产品配置。`PDFIUM_DYNAMIC_LIB_PATH` 继续不是 FerroStage 的运行时配置。

缺 DLL 时外壳行为保持现状：窗口能开，导入报 `LibraryUnavailable`。配方的职责是在正常 `just dev` / `just build` 之后不走到这条路径。

## Rollback

先还原 `justfile` 和 `main.rs` 的子系统属性，再删除 `scripts/stage-pdfium.ps1`。文档与配方一起还原。不删除 `cache/pdfium-7881-win-x64`，那是已有的本地缓存。`target\` 下复制出来的 DLL 和 licenses 是生成物，不必单独提交。
