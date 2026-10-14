# Implement — 09

1. **I-1 消费者评审**：核02/03/07/08接口与上表，语言restart、theme/system事件明确；hot_reload/media capability未就绪禁用，不做循环依赖。
2. **I-2 界面骨架**：讲者内分类、WritePolicy只读、未保存/重试，先测试focus/disabled/SaveError，不接audience树。
3. **I-3 角色/Identify**：拓扑图/稳定等级/warnings、新Audience清旧Audience、新Presenter清旧Presenter（每ID一个role）、single mode、3sIdentify与Presenting禁用；调用07唯一协调器，迁移失败回滚。
4. **I-4 实际设置生效**：布局/grid两键/计时/指针/cache/debug/快捷键/recent；值改变→实际consumer呈现→save→restart一致，非法类型与冲突呈现准确。
5. **I-5 语言/主题/版本**：语言restart真正选择现有locales；dark/light/system配色和系统改变；future只读禁止save/reset，save失败dirty与retry可见。媒体未实现不渲染空control。
6. **I-6 三屏UI实机**：任意选Audience/Presenter/None/Identify→Start真实PDF；Presenting迁移≤500ms、拔线≤1s、保存重启/交换binding一致，焦点观众打开settings只在讲者；AC-1/2最终UI路径证据。
7. **I-7 评审/门**：mapping测试+实际效果人工/自动验证，保存失败/role失败回滚，feature-parity只标已验；run命令，Gate D settings部分记录。

命令：cargo test -p ferrostage-settings -p ferrostage-ui -p ferrostage-core；cargo build -p ferrostage-app --features gpui,pdfium；just ci。
回滚：I-3role事务失败原窗口/binding，I-4save失败旧文件保留；字段/资源接线回滚同步ui/app/settings。10/11后续补相应capability实际验收，本任务不称其功能已完成。
