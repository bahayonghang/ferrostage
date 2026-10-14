# Implement — 07

1. **I-1 display研究门**：核pin Windows backend、DisplayId/uuid/bounds单位、fullscreen参数、DPI/事件与focus/keystroke；生成确有证据的gpui-display-api结论，择一允许路线。无法安全映射则保留具体阻塞。
2. **I-2 feature编译**：ui链接gpui、app转发gpui/pdfium，更新ADR-001/CI；WindowsMSVC native compile通过，core依赖树无GPUI。
3. **I-3 导入/讲者**：启动config/topology、CLI/dialog/drop/最近文件import调用05metadata；Load→Ready占位，无观众计时；坏文件在讲者提示、旧状态保留。
4. **I-4 窗口事务**：实现create-Curtain/Start-Live/Stop-close、失效/零屏/单屏显式模式；注入创建失败测试不留Live/计时窗口。
5. **I-5 动作/角色**：全局routing、Escape模态/preview优先消费、弹UI讲者activate、冲突按键禁用、Swap权威binding事务、save dirty提示、03事件拔屏停止；无独立swap flag。
6. **I-6 Gate B实机**：02配置A Audience/B Presenter/C None→开始/结束/拔线/换C复测；混DPI/旋转/单屏/零屏；占位画面即可，焦点/Identify UI留09。记录环境/操作/截图/代码版本，报告实施后创建。
7. **I-7 评审**：only-snapshot view、read-only导入、错误/焦点、featureCI与Gate B；不能把占位称真实PDF或设置UI完成。

拟feature创建后命令：cargo build -p ferrostage-app --features gpui,pdfium；cargo clippy -p ferrostage-app -p ferrostage-ui --all-targets --features gpui -- -D warnings；cargo test -p ferrostage-core -p ferrostage-platform；just ci。
WindowsMSVC job编译门、纯suite、GateB实机分别记录。回滚Start失败清新窗口不计时；角色应用失败回滚窗口/bindings；feature路线回滚app/ui/CI/ADR同批。
