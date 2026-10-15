# 设置界面：通用/显示器角色分配/讲者布局/计时/指针/快捷键

父任务：`10-14-presenter-v0.3`（R-24 R-33）。复杂任务，设计和步骤见本目录design.md / implement.md。

## Goal

讲者无需手改 TOML 即可完成全部配置，尤其是在三屏环境里直观地指定哪块屏投影、哪块屏是工作台；修改按每键约定即时或重启生效并原子保存；设置界面永远只出现在讲者窗口。

## Background

- 数据源：`02-settings-schema` 的 `Settings` / `SettingsStore`；显示器：`03-monitor-topology` 的 `MonitorInfo` / `ResolvedTopology` / `TopologyEvent`；窗口外壳与入口占位：`07-gpui-shell`。
- 不变量 2：设置窗口只能挂在讲者窗口视图树内；焦点在观众窗口时先切回。
- 语言资源 `assets/locales/{zh-CN,en-US}/app.toml` 已存在。

## Requirements

- **U-1** 设置窗口（或讲者窗口内的模态面板）含左侧分类：通用、显示器、讲者布局、计时器、指针与画笔、快捷键、高级。
- **U-2 显示器页**：
  - 按虚拟桌面 `position`/`native_size` 等比绘制每块屏的矩形，标注编号、`friendly_name`、分辨率、缩放、OS 主屏标记、`MonitorId` 稳定级别（`edid:`/`name:`/`gdi:`）。
  - 每块屏一个角色下拉：Presenter / Audience / None；新Presenter同次清旧Presenter，保证每ID一role；选择 Audience 时若已有另一块 Audience，提示并把旧的改为 None（新UI不得产生重复，旧配置warnings沿用resolve_roles的DuplicateAudience）。
  - "Identify" 按钮：在每块屏中央显示 3 s 的编号与名称覆盖窗口（无边框、置顶、点击穿透不需要）；覆盖窗口不是观众窗口，不读快照，`Presenting` 中禁用该按钮。
  - 显示当前 `ResolvedTopology` 与 `source`、告警文案（如"已保存的观众屏当前不在场"）。
  - 单屏模式选项：`presenter_only` / `windowed_audience`。
  - 订阅 `TopologyEvent`，插拔后实时刷新。
- **U-3 通用页**：语言（zh-CN/en-US，重启生效且重启后实际切换资源）、主题（dark/light/system）、启动时重新打开上次文档、最近文件数量、清空最近文件。
- **U-4 讲者布局页**：下一页/备注比例滑块、备注字号倍率、显示时钟、显示缩略图条、网格列数1..4与可见页数1..16独立控件。
- **U-5 计时器页**：正计/倒计、目标时长、黄色与红色预警阈值（校验黄 > 红 ≥ 0）。
- **U-6 指针与画笔页**：激光笔颜色/半径、画笔颜色/宽度、荧光笔透明度，带实时预览色块。
- **U-7 快捷键页**：每个动作一行，显示当前按键列表；点击"录制"捕获下一次按键并追加；冲突（同键多动作）行内标黄；"恢复默认"。按键字符串语法沿用 `07` 的 research 结论。
- **U-8** 高级页：08实际cache/debug；10接线前hot_reload禁用提示，11加media节后由11接控件；目录/重置仅可写版本。
- **U-9** 校验→运行时应用→save；运行时失败回滚，保存失败保留会话值并未保存/重试，重启取上次成功值；未来版本只读。语言重启实际选择资源、dark/light/system真实切配色并跟随系统，不验空控件。
- **U-10** 角色变更在 `Presenting` 中生效的语义：Audience 屏变更 → 观众窗口迁移到新屏（销毁重建）；Presenter 屏变更 → 讲者窗口移动。

- **U-11** 遵循根 DESIGN.md（R-45）：统一Claude风格token、浅/深配色、系统字体、完整控件状态与键盘焦点；原生截图/对比度/尺寸验证记录本任务research。

## Acceptance Criteria

- [ ] 三屏实机：在显示器页把任意一块设为 Audience，点击 Identify 三块屏各显示正确编号；点击开始演示后观众窗口出现在所选屏。
- [ ] 把 Audience 改到另一块屏（`Presenting` 中），观众窗口 ≤ 500 ms 内迁移，备注不出现在任何屏。
- [ ] 拔掉一块屏，显示器页 ≤ 1 s 刷新并显示告警。
- [ ] 修改计时阈值后讲者计时器颜色立即按新阈值变化；重启后保留。
- [ ] 录制快捷键 `F6` 到"开始演示"，立即可用；设置同键到另一动作时行内标黄。
- [ ] 焦点在观众窗口时按设置快捷键，设置窗口出现在讲者屏而非观众屏。
- [ ] 键→消费者→生效映射；改值实际呈现/保存重开一致；语言重启资源切换、system跟随；save失败/未来只读正确，未接线控件禁用。
- [ ] `just ci` 全绿。

- [ ] 本任务界面在DESIGN.md各窗口尺寸及双主题下完成原生截图、键盘/焦点、hover/disabled/error/loading、字体回退和对比度检查；未执行不勾选。

## Dependencies

完整前置：01/02/03/07/08（实际目录见父Task Map）；父子关系不代表依赖。 08提供实际消费者，10/11控件接线前禁用，不是09初版硬依赖。

## Out of Scope

主题样式系统的完整实现（只提供暗/亮两套基础配色）；多 Audience 镜像；云同步。
