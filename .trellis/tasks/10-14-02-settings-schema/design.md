# Design — 设置 schema 与持久化

## 选择、依赖、输入输出
前置01。S-1…S-11/R-21/R-30…32由本任务实现；02唯一创建core MonitorRole，03才补MonitorId，04不参与创建。settings不依赖platform/GPUI；serde文本适配留settings。继续引用render已有cache常量，不创建新预算来源。
输入为配置路径/TOML/typed修改；输出LoadOutcome { settings, raw_table, warnings, write_policy, needs_writeback }与typed SaveError。pure parse与有副作用load/save分离，check-config只读。默认八节与父design §4一致；debug_overlay/grid_visible_pages/general.recent_files=[]（按recent_files_limit去重/截断）由此引入，media节由11追加。

## 文件范围
现有：crates/ferrostage-settings/src/lib.rs与Cargo.toml、crates/ferrostage-app/src/main.rs与Cargo.toml、crates/ferrostage-core/src/lib.rs、Cargo.toml（仅必要新依赖表）、docs/packaging.md。
实现模块：settings/src/{schema,keybindings,error,merge,parse,store}.rs；parse/{sections,controls,legacy,value}.rs分开节解析/迁移/叶校验，store/{files,paths}.rs分开文件替换/平台路径；core/src/display.rs（仅MonitorRole）；settings临时目录测试与app CLI集成测试。不提前建03的MonitorId或UI。

## 设计
以TOML value树逐已知叶解析，不先把整份文件serde成Settings而令坏单项吞掉全表。校验：target>0、yellow>red≥0；颜色4个u8；比例/宽度有限且合理；grid列1..4/页1..16；cache>0/debounce>0；非法monitor.role为None并告警，不能退为Audience。
保存递归merge已知叶，未知节/嵌套键/monitor元素字段保留；元素以id+重复稳定序号关联原树，保留失联binding。重复角色交03处理，02不在校验中偷偷改role权威。
missing schema默认为首分节v1，骨架扁平cache/debounce真实映射，索引monitor无法迁移则保留原键+warning。不创建空迁移框架。高版本WritePolicy::ReadOnly，不自动备份/写回/重置；unknown值允许按语义保留，注释布局不保证。
Store default_path遵循平台目录，--config覆盖；load坏语法须先成功broken备份，失败返回默认会话但禁止写原件。坏语法原件在显式save前保留；缺省节needs_writeback由app启动调用open_for_startup显式补全，check-config绝不触发。save同目录唯一tmp，sync/close后安全std::fs::rename；Windows已有目标/锁定验证已通过（research/windows-persistence.log），不删除旧目标来“保证替换”。save_session成功后同步原树及monitors来源序号，失败不推进来源序号，保证重复绑定删除/重排后多次保存保留对应未知字段。

## 错误、回滚和消费者
读/备份/写/replace具体错误分开，不日志dump配置。09保存失败保留有效会话值、dirty提示/重试；运行时应用失败先回滚该项，不能写失败值。未来版本控件只读。事务失败旧文件哈希不变，tmp清理；回滚02 API需同步app消费者。native原子替换研究可用父runtime证据，实机协议仍必须执行。
