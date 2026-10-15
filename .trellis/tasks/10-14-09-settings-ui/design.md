# Design — 有实际效果的设置界面

## 选择、依赖、输入输出
前置01/02/03/07/08；U-1…10/R-24/33。M2交付最终P0角色UI，M1用配置不宣称此项已完成。输入Store WritePolicy/Settings/Topology与app应用协调器，输出校验修改→真实runtime effect→save status；只在讲者view树。
语言确定重启生效，theme dark/light/system基础配色真实应用；不提供空控件。media项由11新增，hot_reload由10接线后启用。

## 文件范围
现有ui/lib.rs和07actions/presenter、08workbench、settings/lib.rs/02store、app/main.rs/运行协调器、assets/locales/{zh-CN,en-US}/app.toml（必要UI标签）、ui/settings/app Cargo.toml（仅接线必要）、docs/feature-parity.md。
拟建ui/src/settings_view.rs/字段映射模型。不得将SettingsStore/path/notes传入audience Entity。

## 键→消费者→生效方式
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
拓扑图用03单位/DPI转换，选择新Audience清旧Audience，选择新Presenter也清旧Presenter；每ID一个role，不能旧binding抢先，唯一来源display.monitors；既有duplicate配置展示warning，UI不新建重复。Identify独立3s标号overlay，仅Ready/Idle可用，Presenting禁用，不卡audience安全边界。新屏刷新但不自动role/Start。
07协调器负责Presenting角色迁移，先保证新目标有效与窗体构建，再提交role/window；失败回原配置/窗口，不能先持久化。≤500ms为实机目标；如果无Audience则走安全Stop/Curtain。设置Focus永远讲者，popover不会挂audience。
控件修改校验→runtime应用→save；runtime失败回滚该项；save失败保留当前效果、明显未保存与重试，重启取上次落盘。future版本所有写/重置控件禁用；读配置打开目录也不能触发写回。不要让reset覆写未来文件。
表格是可测试字段映射模型，不以字段数量相等代替实际效果验证。回滚SettingsView与必要app/store接线同批，保留旧可读settings。

## 视觉与交互契约

遵循根 DESIGN.md。07拥有唯一theme.rs及基础控件；08复用工作台布局；09复用主题并消费general.theme，无额外主题设置schema。暖米色/陶土色仅用于讲者UI；原始PDF/媒体/备注光栅与观众安全幕布不着色。字体使用系统替代，交互错误有文字，焦点2px且有间隔。原生窄窗/双主题/DPI截图与键盘验证按DESIGN.md逐项记录，不能以纯测试或浏览器替代。

## Task03 topology interface clarification

Consume the parent design §3 owned monitor identity and explicit confidence/scale-source fields. Windows interface paths are connection-scoped, not guaranteed stable across port changes. Geometry must come from a complete cross-checked enumeration. A TopologyEvent::Unavailable makes Audience unavailable: task07 curtains/stops/closes the audience and never continues from last-known geometry; recovery does not auto-start. Task09 shows diagnostic warnings and identity confidence. Task07 uses the strict connection-name/GPUI UUID bridge defined by the parent; no geometry fallback is allowed. GdiDeviceCapsRatio remains a diagnostic process-context ratio, never an effective-DPI or display-identity substitute; ambiguity rejects Start. Polling is owned by platform, with cancellation; no extra hidden window listener.
