# 讲者界面轻盈毛玻璃与外观设置的实施与验收

- [x] 运行 design-atlas apply 并审查备份，将非视觉放映契约移回设计源，记录深色各角色及实色回退策略。
- [x] 在既有 settings schema／parse／merge 和通用外观 UI 中接入设置、双语和旧配置测试。验证重启与 system 跟随，实色回退不改写用户偏好。
- [x] 从 theme.rs 推进首页、讲者壳、按钮、计时、弹层、设置和识别提示；排查散落的旧色、旧主按钮常量及固定深色。避免 PDF 绘制路径套用主题。
- [x] 测试亮暗对比、禁用、焦点和选中状态；普通原生窗口记录浅／深与开／关，覆盖设置及放映切换。运行 design-atlas slop_check，注明 Rust 扫描覆盖，独立复核截图并报告实际模糊支持。
- [x] 运行 `cargo test -p ferrostage-settings --locked` 及讲者主题对比测试。
- [x] 运行 `cargo test -p ferrostage-ui --features test-support --lib --locked`。记录原生场景、实际结果及未运行项；父任务收口运行 `just check-shell` 和 `just ci`。

[验收记录与原生截图](research/verification.md)
