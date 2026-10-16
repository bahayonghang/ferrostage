# 应用图标与 harfrust shape 警告

实施顺序固定。图标生成和警告修补互不依赖，文档在两处代码都落地后改。

## Checklist

1. 用图像生成得到 1:1 小熊猫源图，写入 `assets/icons/ferrostage.png`。
2. 把源图做成含 16、32、48、256 的 `assets/icons/ferrostage.ico`。优先用本机已有的 ImageMagick 或 Pillow。都没有时，用一次性本地命令调用已锁定的 `image` 0.25.10 写出 ico，不把 `image` 加进 `ferrostage-app` 的 build-dependencies。
3. 加 `crates/ferrostage-app/resources/windows/ferrostage.rc`、`build.rs`，以及 `embed-resource = "=3.0.12"` build-dependency。运行 `cargo update -p embed-resource --precise 3.0.12` 只在锁文件缺直接边时更新；不得升级版本。
4. 给 `vendor/harfrust/src/hb/face.rs:439` 的自由函数加上 `#[cfg(feature = "experimental_font_api")]`。
5. 更新 `assets/icons/README.md`、`vendor/README.md`、ADR-008、`.trellis/spec/frontend/maintained-dependencies.md`。
6. 在 `research/` 记录实际生成命令、ico 档位、`cargo metadata` 里的 `embed-resource` 版本，以及警告前后的编译输出片段。

## Validation

```text
cargo run -p ferrostage-app --features gpui,pdfium
cargo check -p harfrust --features experimental_font_api --locked --offline
cargo clippy -p ferrostage-app -p ferrostage-ui --all-targets --features gpui,pdfium --locked --offline -- -D warnings
cargo deny --all-features --locked --offline check
```

`cargo run` 看警告是否消失，并看讲者窗口标题栏与任务栏。feature 检查只确认再导出仍能编译。clippy 与 deny 是产品门禁。不把 HarfRust 六条既有 strict lint 当作本任务失败。

默认 `just ci` 不链 GPUI。本任务的图标只在 `--features gpui,pdfium` 的 Windows exe 上可见，所以窗口验收用上面的 `cargo run`，不用默认 `just ci` 代替。

## Risky files

- `vendor/harfrust/src/hb/face.rs`
- `crates/ferrostage-app/Cargo.toml`
- `Cargo.lock`
- `crates/ferrostage-app/build.rs`

回退点是实施清单 4 之前：此时还没有 vendor 改动。清单 4 之后若检查失败，先还原 `face.rs` 的属性，再处理图标。

## Start gate

`task.py start` 之前，`implement.jsonl` 和 `check.jsonl` 必须已经有策划条目。用户批准本目录的规划摘要后才能 start。批准前不改产品代码，不生成图标。
