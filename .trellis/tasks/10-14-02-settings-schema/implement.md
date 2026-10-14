# Implement — 02

1. **I-1 研究/评审**：读父design §3–4、runtime安全rename证据及backend specs；列八节叶键/default/range/consumer，审MonitorRole所有权和Windows目标已存在策略；形成具体设计后才实施。
2. **I-2 类型/schema**：建立core MonitorRole及settings serde适配，八节/debug/grid默认；验证空配置往返、每节缺省、非法值/类型单项回退且其他键不变。
3. **I-3 版本/merge**：实现原树保留、已知叶merge、monitors匹配、真实扁平迁移/未来只读；测试未知顶层/已知节/数组元素，经改role、删除binding仍符合约定。
4. **I-4 Store**：实现路径、broken备份、needs_writeback、same-dir tmp/flush/replace/cleanup；全部fixture创建在独立temp目录，故障注入写/备份/replace失败，旧文件哈希不变。
5. **I-5 app接线**：--config、只读check-config退出码/warnings；启动可写模式补缺省节，未来/备份失败不写；返回save error/dirty状态供09使用，不加入UI。
6. **I-6 Windows门**：NTFS目标不存在/已有、只读/锁定、写tmp后中断、replace失败/重试，保存完整新文件或保留完整旧文件；记录OS/文件系统/共享模式与结果（实施时创建报告，不提前注入路径）。
7. **I-7 评审**：未知键/只读用户PDF/core依赖/错误日志/范围复查，再过命令和父Gate A的settings部分。

命令（实施时）：cargo test -p ferrostage-settings -p ferrostage-core；cargo run -p ferrostage-app -- check-config <临时配置>；cargo tree -p ferrostage-core；just ci。
通过证据：纯Rust测试、实际Windows替换与CLI结果；缺实机标待验，不能把POSIX测试代替Windows。
回滚点：I-2 API/schema变更同步app，I-4任何失败保留旧settings，I-5失败不覆写用户文件。只提交/归档完整验收并获后续授权。
