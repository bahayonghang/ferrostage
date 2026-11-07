# 集成计划

- [x] 先独立审阅讲者外观任务；其余四项按用户指定的优化任务保存。功能实施顺序建议裁切确认 → 无备注布局，投影状态 → 返回原处。界面改造可先行。
- [x] 性能子任务已在固定外观下取得打开与翻步的原生 A/B 数据，[复测报告](../11-07-presenter-performance/research/native-results.md)记录了目标身份、窗口尺寸和绘制边界。
- [x] 各子项执行本项清单和正式入口测试；更新受影响的 Trellis 契约，保留只读和观众隔离边界。
- [x] 以普通 PDF、同页备注、混合独立备注和带 overlay 文稿连走父 PRD 场景，记录哪些 PDF 是真实导出、哪些为合成夹具。
- [x] 运行 `just check-shell` 和 `just ci`，复核讲者设置、一次原生放映及隐私像素。记录未执行检查，再决定能否归档。

最终集成结果与限制见 [integration-verification-20261107.md](research/integration-verification-20261107.md)。
