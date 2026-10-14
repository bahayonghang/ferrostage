# Implement — 04

1. **I-1 契约评审**：读父design §2/6与behavior证据；列合法/非法边、timer展示、preview和swap所有权；核01已同步旧spec，不引入OS。
2. **I-2 回归**：先以受控now测试Load→Ready/Curtain、Start、Stop/Close、无Audience拒绝；添加当前preview泄漏能失败的测试，避免镜像实现断言。
3. **I-3 状态/时间**：idle/newReady/apply_at/elapsed_at、Pause/Resume/Reset；Start5s→Pause5s→Resume2s→Stop7s、Ready冻结、再次Start0，非法命令状态不变。
4. **I-4 投影/frame**：新增stage、BgraFrame/AudienceVisual受控纯类型；0/溢出/错stride拒绝，Curtain无visual；document投影测试用secret notes确认不复制，review签名不靠Debug文本单独证明。
5. **I-5 预览/swap/descriptor**：分离preview/committed页、首尾钳制、Commit/Cancel；移除独立swap flag，声明app intent；纯DocumentDescriptor与strict generation，不存PDFbytes。
6. **I-6 全消费者/属性**：更新core name/error及document；随机命令序列stage不为Presenting⇒Curtain/无visual，preview不改audience，Start失败原状态不变；保留现有generation/crop性质测试。
7. **I-7 Gate A**：运行命令并检查core依赖/no clock read/no notes；记录API变更给05/06/07/08/10/11。真实窗口/像素隔离留07/08。

命令：cargo test -p ferrostage-core -p ferrostage-document；cargo tree -p ferrostage-core；just ci。
回滚点：I-3状态API、I-4snapshot与document同批回滚，I-5旧bytes桥接由10最终收敛，不能留下两套提交权威。本次不执行命令或启动。
