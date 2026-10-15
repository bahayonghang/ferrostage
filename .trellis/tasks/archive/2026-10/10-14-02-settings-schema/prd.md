# 设置系统：Schema、持久化、校验与迁移

父任务：`10-14-presenter-v0.3`（R-30 R-31 R-32）。复杂任务，设计和步骤见本目录design.md / implement.md。

## Goal

把 `ferrostage-settings` 从 6 个扁平键扩展为完整的分节设置系统，具备确定的文件位置、原子保存、损坏保护、未知键保留与版本迁移，使设置界面（`09`）、显示器角色（`03`）与 GPUI 外壳（`07`）都有稳定的数据源。

## Background

- 现状：`crates/ferrostage-settings/src/lib.rs:14-21` 只有 `presenter_monitor/audience_monitor: Option<u32>`、三个缓存预算、`hot_reload_debounce_ms`；只有 `from_toml`，没有路径、保存、迁移。
- `ferrostage-app` 的 `check-config <path>` 子命令调用 `from_toml`（`crates/ferrostage-app/src/main.rs:65-69`），需保持可用。
- 缓存默认值来自 `ferrostage_render::{L1_GPU_BUDGET_MIB, L2_CPU_BUDGET_MIB, L3_THUMB_BUDGET_MIB}`，保留该引用。
- 完整 schema 与默认值见 `docs/PROJECT_PLAN.md` §4.4；共享契约见父任务 `design.md` §4。
- 语言资源 `assets/locales/{zh-CN,en-US}/app.toml` 已存在，`general.language` 的合法值以此为准。
- `.trellis/spec/backend/database-guidelines.md` 规定"无 ORM；文档模型与 TOML"，本任务遵循。

## Requirements

- **S-1** schema_version与八节默认沿用PROJECT_PLAN。02同时引入general.debug_overlay=false、general.recent_files=[]（07消费）、presenter.grid_columns=4（1..4）与grid_visible_pages=16（1..16）；08消费、09接线，11缺省追加media节。
- **S-2** `display.monitors: Vec<MonitorBinding { id: String, friendly_name: String, role: MonitorRole }>`；`MonitorRole` 定义在 `ferrostage-core`（`Presenter | Audience | None`），settings 通过 `serde` 以小写字符串序列化。删除 `presenter_monitor` / `audience_monitor` 索引键。
- **S-3** `display.single_monitor_mode: PresenterOnly | WindowedAudience`，默认 `PresenterOnly`。
- **S-4** `SettingsStore::default_path()`：Windows `%APPDATA%\ferrostage\settings.toml`；Linux `$XDG_CONFIG_HOME/ferrostage/settings.toml`（默认 `~/.config`）；macOS `~/Library/Application Support/ferrostage/settings.toml`。`SettingsStore::at(path)` 支持 CLI 覆盖。
- **S-5** load返回typed Settings、原始TOML树、warnings、WritePolicy、needs_writeback。缺文件默认；坏语法先备份broken，备份失败原件不动且禁写；非法值及非法类型逐项回退。纯解析/check-config不改盘，app可写模式显式补缺省节。
- **S-6** save同目录唯一tmp，完整写入sync_all并关闭句柄后安全replace-existing，失败旧文件不变且清tmp。优先std安全API；Windows已有目标/锁定/中断实测，不假定rename不能覆盖或所有故障已经原子。
- **S-7** 递归merge已知叶键，不整节覆盖；保留未知顶层、已知节内部、monitors每项字段；按id+重复稳定序号对齐，删除项只删binding。保留语义值，不保证注释排版字节一致。
- **S-8** 首分节schema为v1；真实骨架扁平缓存/防抖键映射，旧索引角色保留未知并告警重绑；不建空迁移链。未来版本只读，不写回/改名/重置；缺省节仅可写当前版本补全。
- **S-9** keybindings为Vec<String>；去重/空串/跨动作冲突诊断。07统一语法解析和运行时冲突处理，冲突不能执行多个动作；02不引入GPUI。
- **S-10** app增加--config并保持只读check-config；语法/I/O失败exit1、单项回退exit0。SaveError供UI标未保存/重试、保留会话值，未来版本禁改。
- **S-11** 日志遵循 `.trellis/spec/backend/logging-guidelines.md`：告警用 `tracing::warn`，不打印完整文件内容。

## Acceptance Criteria

- [x] `from_toml("")` 得到的 `Settings` 与 `Settings::default()` 相等；`toml::to_string(&Settings::default())` 可被 `from_toml` 往返并相等。
- [x] 删除任意一节的 TOML 解析成功且该节为默认。
- [x] `timer.target_minutes = 0` 解析成功、值回退为 45、告警包含键名。
- [x] 顶层future、general未知键、monitors元素未知字段经改已知值/save均保留；未来版本拒save且哈希不变。
- [x] target_minutes非法类型仅单项回退，缺省节app补全，check-config不改盘；save失败旧文件完整。
- [x] 语法错误文件 load 后生成 `settings.toml.broken-*`，返回默认值。
- [x] `save` 在目标目录不存在时成功；崩溃模拟（写 tmp 后不 rename）不破坏既有 `settings.toml`。
- [x] `ferrostage check-config <path>` 行为符合 S-10。
- [x] `ferrostage-settings` 不依赖 `ferrostage-platform` 或 GPUI；`just ci` 全绿。

2026-10-14 实现者验证证据：`research/implementation-review-2026-10-14.md`、`research/ci.log`（修复前57个测试）、`research/windows-persistence.log`、`research/cli-probe.log`。独立trellis-check已完成：`research/check-review-2026-10-14.md`、`research/check-ci.log`（修复后59个测试）、`research/check-verified-source-sha256.json`。修复非法快捷键数组逐action回退，显式空数组仍为解绑；本勾选不代表提交/归档或父Gate A全项通过。

## Dependencies

完整前置：01（具体目录见父Task Map）。父子关联不表示依赖。

## Out of Scope

设置界面（`09`）；显示器枚举与角色解析（`03`）；快捷键字符串解析（`07`）。
