# Design — 有实际效果的设置界面

> Current acceptance policy (2026-10-15 user revision): `.trellis/tasks/10-14-presenter-v0.3/acceptance-policy.md` supersedes older hardware/matrix blockers below. Implement all features first; retain automatic/feature/native-functional checks. Physical, exhaustive native/size and performance/soak matrices are deferred, not PASS. Gate B implementation is satisfied by V16;08 may start. Gate C/D are functional integration gates. Historical results and product contracts remain unchanged.

## 选择、依赖、输入输出
前置01/02/03/07/08；U-1…10/R-24/33。M2交付完整设置集成，M1已使用07最小分配UI，但不宣称完整设置页已完成。输入Store WritePolicy/Settings/Topology与app应用协调器，输出校验修改→真实runtime effect→save status；只在讲者view树。
语言确定重启生效，theme dark/light/system基础配色真实应用；不提供空控件。media项由11新增，hot_reload由10接线后启用。

## 文件范围
现有ui/lib.rs和07actions/presenter、08workbench、settings/lib.rs/02store、app/main.rs/运行协调器、assets/locales/{zh-CN,en-US}/app.toml（必要UI标签）、ui/settings/app Cargo.toml（仅接线必要）、docs/feature-parity.md。
拟建ui/src/settings_view.rs/字段映射模型；嵌入07共享display_settings，不重建其编号/Identify/角色事务。不得将SettingsStore/path/notes传入audience Entity。

## 键→消费者→生效方式

Task08 concrete pointer contract: laser color/radius reach Audience only through the pure snapshot `LaserStyle`, updated by checked `SetLaserStyle`;09 must use that command and must not pass settings to the audience view. Failed validation leaves runtime and persisted settings unchanged.

Task08 GPU failure refinement: authorized visual metadata may carry a scalar `AudienceVisual.texture_budget_bytes` for the audience's local texture transaction; this is not a settings/controller capability. GPU upload success, not CPU raster availability, commits the new image; failed upload preserves the previous complete image and emits a narrow rendering-error event for Presenter feedback. Each window owns half of total L1, and reserves half its own allocation for one pending upload; committed images use the other half. A shrink that cannot fit retained committed images in the new committed half is rejected, preserving the prior effective budget.09 must preflight Presenter and Audience together before applying/persisting a new L1 setting, so a failed change cannot leave divergent effective budgets. An upload failure is not retried every paint for the same pixel identity; a new identity or document reopen permits another attempt. Consume the final08 implementation/review for the concrete APIs and regression evidence.

Consumer audit2026-10-15: the current RenderHandle accepts CPU/thumbnail budgets only at spawn;09 must add the smallest acknowledged runtime budget-update message to the existing render worker/cache, including immediate eviction and failure handling, to satisfy the table below. Do not rebuild the PDFium worker or label a saved-but-unused cache setting as applied. This necessary consumer adapter extends09's file scope to render worker/protocol/cache and targeted regressions. Also replace existing shortcut registrations rather than accumulating duplicate bindings; the actual GPUI keymap removal/replacement API must be verified before implementation.

Pinned GPUI source supplies `App::bind_keys` and `App::clear_key_bindings` (`vendor/gpui/src/app.rs`, inspected2026-10-15). Rebinding must reinstall the complete application/modal binding set once while retaining a single set of action handlers; clearing keys without restoring modal safety bindings is not a valid replacement.
| 键 | 实际消费者 | 生效/阶段 |
| --- | --- | --- |
| display.monitors / single_monitor_mode / fullscreen | 07角色/窗口协调器 | 即时，运行时失败回滚；保存失败dirty |
| presenter ratios/fonts/clock/thumbnail/grid两键 | 08工作台 | 即时，grid列与可见页两个控件 |
| timer目标/阈值/mode、pointer颜色/宽度/alpha | 04时长+08呈现 | 即时，不重启计时 |
| keybindings | 07统一规范解析/注册 | 立即替换旧binding，冲突禁用并警告 |
| cache / general.debug_overlay | 08GPU与05CPU预算/overlay | 即时按新预算evict |
| general.language | app资源选择 | 标明重启生效，重启后确实切语言 |
| general.theme | ui基础palette/system事件 | dark/light即时，system随系统变化 |
| reopen_last_document/recent_files_limit/clear recent | 07导入/最近文件服务 | 下一启动/立即限制列表 |
| hot_reload | 10watcher服务 | 10接线前禁用提示，接线后实际启停 |
| media.max_decode_height | 11decoder | 11加入后才显示可编辑，下一槽位重建生效 |

general recent文件元数据仅设置/讲者控制侧，不能进入snapshot。general.recent_files=[]由02定义、07维护、09清空/限制，unknown merge保留；不引入PDF写回或长期notes存储。

## 交互/错误/回滚

Concrete09 consumer refinement: `RenderHandle::set_cache_budgets(CacheBudgets) -> Reply<CacheBudgetAck>` is planned as one acknowledged message to the existing worker, updating CPU/thumbnail limits and immediately evicting excess entries on that same thread. The acknowledgement records effective budgets/usage. Shell serializes settings runtime transactions, preflights both GPU caches, waits for worker acknowledgement and rechecks before committing Settings/save; a later rejection rolls worker budgets back to their prior values. Pending edits must not race acknowledgements. Rollback failure is visible, never persisted as success. No extra PDFium worker/restart is introduced.
拓扑图用03单位/DPI转换并复用07编号；应用屏/PDF屏选择使用07草稿Apply/Cancel，一次更新两项并清除旧角色；每ID一个role，不能旧binding抢先，唯一来源display.monitors；既有duplicate配置展示warning，UI不新建重复。Identify独立3s标号overlay，仅Ready/Idle可用，Presenting禁用，不卡audience安全边界。新屏刷新但不自动role/Start。
07协调器负责Presenting角色迁移，先保证新目标有效与窗体构建，再提交role/window；失败回原配置/窗口，不能先持久化。≤500ms为实机目标；用户选择“不播放”时（单屏同步presenter_only），先Curtain→Stop→关闭观众，保留页并冻结计时回Ready；有效目标迁移才保持Presenting/运行计时。设置Focus永远讲者，popover不会挂audience。
显示器目的地使用共享草稿Apply，其余控件按各键约定校验→runtime应用→save；runtime失败回滚该项；save失败保留当前效果、明显未保存与重试，重启取上次落盘。future版本所有写/重置控件禁用；读配置打开目录也不能触发写回。不要让reset覆写未来文件。
表格是可测试字段映射模型，不以字段数量相等代替实际效果验证。回滚SettingsView与必要app/store接线同批，保留旧可读settings。

## 视觉与交互契约

遵循根 DESIGN.md。07拥有唯一theme.rs及基础控件；08复用工作台布局；09复用主题并消费general.theme，无额外主题设置schema。暖米色/陶土色仅用于讲者UI；原始PDF/媒体/备注光栅与观众安全幕布不着色。字体使用系统替代，交互错误有文字，焦点2px且有间隔。原生窄窗/双主题/DPI截图与键盘验证按DESIGN.md逐项记录，不能以纯测试或浏览器替代。

## Task03 topology interface clarification

Consume the parent design §3 owned monitor identity and explicit confidence/scale-source fields. Windows interface paths are connection-scoped, not guaranteed stable across port changes. Geometry must come from a complete cross-checked enumeration. A TopologyEvent::Unavailable makes Audience unavailable: task07 curtains/stops/closes the audience and never continues from last-known geometry; recovery does not auto-start. Task09 shows diagnostic warnings and identity confidence. Task07 uses the strict connection-name/GPUI UUID bridge defined by the parent; no geometry fallback is allowed. GdiDeviceCapsRatio remains a diagnostic process-context ratio, never an effective-DPI or display-identity substitute; ambiguity rejects Start. Polling is owned by platform, with cancellation; no extra hidden window listener.
