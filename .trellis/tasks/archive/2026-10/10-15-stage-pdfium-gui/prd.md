# 让 just dev 和 just build 直接打开可导入 PDF 的窗口

## Goal

在 Windows 上，`just dev` 打开的讲者窗口可以导入 PDF。`just build` 生成的 `target\release\ferrostage.exe` 双击后停留在讲者窗口，不弹出随即消失的控制台。默认 `just ci` 仍然不链接 GPUI，也不需要本机 PDFium。

## Background

这是两件独立的事。截图里的红字只解释 `just dev`。release 目录里双击闪退是另一条构建路径。

`just dev` 已链接 PDFium 的 Rust 绑定。`assets/locales/zh-CN/app.toml:33` 的 `import-unavailable` 加上 `crates/ferrostage-pdf/src/raster.rs:27-28` 的 `LibraryUnavailable`，拼出截图原文「PDF 导入不可用：PDFium library could not be loaded from the configured path」。这条错误只在 `pdfium` feature 打开、并且 `crates/ferrostage-pdf/src/native.rs:27-30` 的 `canonicalize` 或 `:56-57` 的 `bind_to_library` 失败时出现。`crates/ferrostage-ui/src/shell.rs:645-647` 在渲染器是 `Err` 时把它显示出来。绑定没有缺。

缺的是运行时 DLL。`crates/ferrostage-app/src/main.rs:116-125` 在未传 `--pdfium-library` 时使用 exe 旁边的 `pdfium.dll`。`justfile:47-48` 的 `just dev` 没有传这个参数，也没有把 DLL 拷到 `target\debug`。2026-10-15 实测：`target\debug\ferrostage.exe` 42,918,912 字节，PE subsystem 3（控制台），二进制里没有 “presenter UI is not linked yet”，有上述 PDFium 错误文案。`target\debug\pdfium.dll` 不存在。`cache\pdfium-7881-win-x64\bin\pdfium.dll` 存在，SHA256 `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`，与 `docs/pdfium-runtime.json` 的 `dll_sha256` 一致。

`just build` 没有打开桌面功能。`justfile:42-44` 是 `cargo build --workspace --release`，不带 `gpui,pdfium`。`crates/ferrostage-app/src/main.rs:92-99` 因此打印骨架句并返回成功。实测 `target\release\ferrostage.exe` 2,009,088 字节，修改时间 2026-10-15 22:16:59，PE subsystem 3。隔离配置启动后退出码 0，标准输出含 `ferrostage 0.1.0 skeleton — presenter UI is not linked yet (GPUI pin v1.23.3)`。资源管理器双击控制台程序时会分配控制台；进程马上退出，窗口就只闪一下。`target\release\pdfium.dll` 同样不存在。release 二进制里没有 PDFium 加载失败文案。

`README.md:45-54` 和 `README.zh-CN.md:27-37` 仍写着这几条 just 命令使用默认 features。`just dev` 实际上已经传了 `gpui,pdfium`，文档和配方不一致。`docs/packaging.md:24-29` 规定应用进程永不下载 PDFium；`scripts/fetch-pdfium.ps1` 只给开发者准备已锁定的 151.0.7881.0 产物。`docs/packaging.md:59-64` 要求完整桌面构建显式传入 DLL，缺库时外壳仍要能开。

`crates/ferrostage-app/src/main.rs` 没有 `windows_subsystem`。两个现成 exe 都是 `WINDOWS_CUI`。工作区 `unsafe_code = "forbid"`，不能在应用里调用 `AttachConsole`。

`justfile:51-52` 的 `just install` 同样不带 `gpui,pdfium`，装出来的仍是骨架程序。本次报告没有走这条命令，但它和 `just build` 是同一类缺口。

## Requirements

- **R1** `just dev` 启动后进入讲者窗口。使用已校验的 `pdfium.dll` 时，点击「打开 PDF」不再出现 `LibraryUnavailable`。未改 `--pdfium-library` 时，查找路径仍是 exe 旁边的 `pdfium.dll`。
- **R2** `just build` 留下的 `target\release\ferrostage.exe` 链接 `gpui` 与 `pdfium`。双击后讲者窗口保持打开，不分配控制台。exe 旁有校验通过的 `pdfium.dll`，以及打包说明要求随应用分发的 `LICENSE` 和完整 `licenses/`。
- **R3** 应用进程仍然不下载 PDFium，不写用户 PDF。下载只发生在现有 `scripts/fetch-pdfium.ps1`，并且只接受 `docs/pdfium-runtime.json` 里的 URL 与哈希。缓存已匹配时不再下载。
- **R4** `just ci`、默认 `cargo test` 和默认 `cargo clippy` 不启用 `gpui` 或 `pdfium`，也不要求本机存在 DLL。
- **R5** `check-config` 保持只读：不写设置、不写备份、不写启动错误日志。调试构建从终端运行时仍把结果打到 stdout/stderr。现有 `crates/ferrostage-app/tests/check_config.rs` 继续通过。
- **R6** release 在打开窗口前失败时，把同一条错误写进设置目录下的 `startup-error.log`。日志不含 PDF 字节，也不转储设置全文。成功启动不写这个文件。
- **R7** `just install` 安装带 `gpui,pdfium` 的程序，并把校验过的 `pdfium.dll` 放到该 exe 旁边。不把 `licenses/` 树放进 Cargo 的 bin 目录。

## Acceptance Criteria

- [ ] `just dev` 打开讲者窗口。用现有缓存 DLL 时，导入一张可读 PDF 能离开空闲页，界面上没有 “PDFium library could not be loaded from the configured path”。
- [ ] `target\release\ferrostage.exe` 的 PE subsystem 为 2（`WINDOWS_GUI`）。双击后窗口保持，不出现控制台。二进制中找不到 `presenter UI is not linked yet`。
- [ ] `target\debug\pdfium.dll` 与 `target\release\pdfium.dll` 的 SHA256 都是 `docs/pdfium-runtime.json` 的 `dll_sha256`。release 目录含未改动的 PDFium `LICENSE` 与 `licenses/`。
- [ ] 隔离配置下，release `check-config` 在 stdout 被重定向时仍打印解析结果，退出码与调试构建一致，且不创建 `startup-error.log`。
- [ ] 窗口打开前的失败路径会追加 `startup-error.log`。`cargo test -p ferrostage-app --locked` 通过。
- [ ] `just ci` 通过，且不依赖 `FERROSTAGE_TEST_PDFIUM` 或 `target\*\pdfium.dll`。
- [ ] `cargo clippy -p ferrostage-app -p ferrostage-ui --all-targets --features gpui,pdfium --locked --offline -- -D warnings` 通过。
- [ ] 产品源码中没有新增 `unsafe`，也没有 PDFium 下载逻辑。

## Out of Scope

`media` feature、GStreamer 运行库和 `ferrostage-media-helper.exe`。安装包、自动更新、把 PDFium 静态链进 exe。改 `LibraryUnavailable` 的错误类型或导入事务。Linux / macOS 的库查找。把 `gpui,pdfium` 设成 crate 的 default features。
