# 让 just dev 和 just build 直接打开可导入 PDF 的窗口

按这个顺序做。先放运行库和配方，再改 release 子系统，最后改文档。子系统属性一旦加上，就要用下面的 release 探针确认没有把骨架程序留成“双击毫无反应”。

## Checklist

1. 新增 `scripts/stage-pdfium.ps1`。缓存命中 `docs/pdfium-runtime.json` 的 `dll_sha256` 时不得调用网络。`-DllOnly` 只复制 `pdfium.dll`。完整模式还复制 `LICENSE` 与 `licenses/`，并校验 notice 哈希。不复制 `include/` 和 `lib/`。
2. 改 `justfile` 的 `dev`、`build`、`install`，按 `design.md` 的命令顺序调用脚本。`build` 最后一步是 `cargo build -p ferrostage-app --release --features gpui,pdfium --locked`，然后 stage 到 `target/release`。`dev` 把额外参数原样传给 `cargo run`。
3. 在 `crates/ferrostage-app/src/main.rs` 给非调试 Windows 构建加上 `windows_subsystem = "windows"`。窗口前失败时追加 `startup-error.log`。不要改 `check-config` 的读写边界，不要加 `unsafe`。
4. 更新 `README.md`、`README.zh-CN.md`、`docs/packaging.md` 和 `justfile` 顶部注释，使命令说明和配方一致。
5. 把本次命令输出里的哈希、PE subsystem 和导入结果摘要写到 `research/`。不要把 DLL、许可证副本或用户 PDF 写进任务目录。

## Validation

缓存已在 `cache/pdfium-7881-win-x64` 时，先断网或挡住下载也能完成 stage。然后再跑：

```text
just ci
cargo test -p ferrostage-app --locked
cargo clippy -p ferrostage-app -p ferrostage-ui --all-targets --features gpui,pdfium --locked --offline -- -D warnings
just build
just dev
```

`just ci` 期间不设置 `FERROSTAGE_TEST_PDFIUM`。`just build` 会编译 GPUI release，耗时单独看，不用 `just ci` 代替。

release 产物核对：

- PE optional header subsystem 为 2。调试 `target\debug\ferrostage.exe` 仍为 3。
- release exe 不含 `presenter UI is not linked yet`。
- `target\debug\pdfium.dll` 与 `target\release\pdfium.dll` 的 SHA256 等于 `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`。
- `target\release\licenses\` 的文件哈希与 `docs/pdfium-runtime.json` 的 notices 一致。

用临时 settings 路径运行 release `ferrostage.exe check-config <fixture>`，stdout 重定向到文件。退出码与调试构建一致，临时目录里没有 `startup-error.log`。再把 settings 路径指到一个不可写位置，确认窗口出现前的失败会留下一行日志。

桌面验收用 `just dev`：窗口出现，打开一张本地可读 PDF，空闲页上的 PDFium 路径错误消失，文档进入就绪。再双击 `target\release\ferrostage.exe`，确认没有控制台，窗口保持，导入同样成功。

## Risky files

- `justfile`
- `crates/ferrostage-app/src/main.rs`
- `scripts/stage-pdfium.ps1`
- `README.md`
- `README.zh-CN.md`
- `docs/packaging.md`

回退点在清单 3 之前：此时还没有 GUI 子系统，双击行为仍是旧的控制台骨架。清单 3 之后若 release 探针失败，先去掉 `windows_subsystem` 属性，保留 DLL 放置，再看窗口是否恢复。

## Start gate

用户批准本目录最新规划摘要之前，不运行 `task.py start`，不改产品代码。`implement.jsonl` 和 `check.jsonl` 在 start 前必须有策划条目。
