# 无备注文稿的稳定双预览布局的实施与验收

- [x] 对全无、混合、待确认、加载、失败分别断言空间分配；增加封面→有备注页和最后一步回归。
- [x] 复用分隔线与现有宽窄断点，接入恢复当前布局比例和下一步标签，验证键盘操作。
- [x] 从无备注文稿切回混合文稿验证比例恢复；以一个宽窗和断点两侧的布局断言检查工作区。
- [x] 核对涉及的 shell／workbench 测试通过。
- [x] 运行 `cargo test -p ferrostage-ui --features test-support --lib --locked`。记录原生场景、实际结果及未运行项；父任务收口运行 `just check-shell` 和 `just ci`。

[Verification](research/verification.md)
