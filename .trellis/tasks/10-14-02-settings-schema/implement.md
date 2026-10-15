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

## 2026-10-14 恢复实施记录

- I-1：已审阅既有未提交实现、父共享契约与01报告；不重建现有模块。
- I-2…5：实现已验证。连续保存回归已修复，save_session成功后更新来源序号；失败保留内存值/原树/dirty/错误，重试成功清理状态。
- 基线检查已完成：app 4/core 10/settings既有34通过，新增重复绑定连续保存回归失败（未知字段丢失）。原始输出：`research/baseline-tests.log`。
- I-6：PASS，Windows 10.0.26300.0 / NTFS：缺失/已有目标、只读属性、无DELETE共享锁、释放后重试、写tmp后中断模拟、replace失败/清理；原始输出 `research/windows-persistence.log`。不是断电持久性证明。
- I-7实现者审查：PASS。模块拆分后`just ci`退出0，57测试全部通过（settings36/core10/app4/其他7），fmt/clippy/deny通过；deny有syn/windows-sys重复版本非失败告警。`research/ci.log`。
- CLI probe：PASS，`cargo run -p ferrostage-app -- check-config <临时配置>`退出0，target=0回退45，SHA256前后一致；语法/I/O退出1与无备份由4个CLI测试覆盖。`research/cli-probe.log`。
- core/settings依赖树、toolchain与SHA证据：`research/{core-tree,settings-tree,toolchain}.log`、`research/verified-source-sha256.json`。Git diff --check通过。
- 实现者交接时恢复点：等待主会话调度独立trellis-check；无运行中的检查、无task02外部阻塞。未提交/推送/归档；父Gate A的04/03部分由父任务协调。

## 2026-10-14 独立检查（PASS）

- 已核对实现者SHA256清单，47个源码/manifest/构建及packaging文件均匹配。
- I-7发现并修复：`keybindings.next = [123]` 被过滤为空列表，违反S-5非法类型逐键默认回退。先补错误元素/显式空数组回归，再最小修复。
- 回归已复现：`cargo test -p ferrostage-settings invalid_keybinding_elements_fall_back_only_that_action`退出101，实际`[]`对比默认`[PageDown, Space, Right]`；输出`research/check-keybinding-regression.log`。修复为错误元素令该action回退默认，不影响其他action；显式`[]`仍为解绑。
- 定向检查PASS：`cargo test -p ferrostage-settings keybinding`，3测试；`research/check-keybinding-tests.log`。
- 最终`just ci` PASS/exit0：fmt、Clippy/type-check、59测试（settings38/core10/app4/其他7）、deny通过；`research/check-ci.log`。无新增失败；syn/windows-sys仍为既有非失败重复版本告警。
- 独立全范围审查PASS：`research/check-review-2026-10-14.md`；当前源码证据`research/check-verified-source-sha256.json`。Windows原协议仍适用，此次CI也重跑其测试通过；未将受控中断模拟称为断电保证。
- `git diff --check`与本任务上下文validate通过。无运行中检查、无02未修问题或外部阻塞；02实现与独立检查完成，待审阅/提交/归档。保持task.json in_progress；后续04/03与父Gate A交由主会话推进。
