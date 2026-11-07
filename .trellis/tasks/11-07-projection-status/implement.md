# 真实投影状态与绘制回执的实施与验收

- [ ] 盘点普通页、转场及遮罩的回执条件；先用乱序、同页新一代、断屏和失败构造状态转换用例。
- [ ] 接入讲者顶部徽标、详情与恢复；重用现有显示器设置入口、遮罩命令和中英文文案。
- [ ] 运行 playback、timed_playback 和 display 相关回归；记录单屏和可用窗口上的观察，缺少设备不得补写实测。
- [ ] 核对涉及的 shell／workbench 测试通过。
- [ ] 运行 `cargo test -p ferrostage-ui --features test-support --lib --locked`。记录原生场景、实际结果及未运行项；父任务收口运行 `just check-shell` 和 `just ci`。
