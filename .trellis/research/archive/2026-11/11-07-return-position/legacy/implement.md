# 答疑跳转后返回准确讲述步骤的实施与验收

- [x] 用 A 步骤 2→B→C、返回失败／重试、乱序回执及黑屏构造回归；确认现有数字与缩略图入口。
- [x] 添加会话锚点、返回及清除动作、可配置绑定和双语文案，涵盖 Stop 与成功重载的不同生命周期。
- [x] 依赖真实投影回执契约后验收 Presenting；检查返回完成后的焦点和翻页笔方向键回归。
- [x] 核对涉及的 shell／workbench 测试通过。
- [x] 运行 `cargo test -p ferrostage-ui --features test-support --lib --locked`。记录原生场景、实际结果及未运行项；父任务收口运行 `just check-shell` 和 `just ci`。

验收结果与范围见 [verification.md](research/verification.md)。
