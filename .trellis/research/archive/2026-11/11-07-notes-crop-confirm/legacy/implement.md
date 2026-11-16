# 同页备注裁切预览与放映前确认的实施与验收

- [x] 构造普通宽屏、四方向双色标记、独立／混合备注样本；先断言观众像素排除私有区域、输入 PDF 字节相同。
- [x] 实现候选判定、面板及中英文文案；分别从菜单与 Start 输入测试取消、确认、失败、同规则与批注保留。
- [x] 覆盖不同结构热替换在播放中的旧帧保留，运行相关 document 与 GPUI 回归；同步菜单现行规范。
- [x] 运行 `cargo test -p ferrostage-document --locked`。
- [x] 运行 `cargo test -p ferrostage-ui --features test-support --lib --locked`。记录原生场景、实际结果及未运行项；父任务收口运行 `just check-shell` 和 `just ci`。

[Windows 原生验收与回归结果](research/verification.md)
