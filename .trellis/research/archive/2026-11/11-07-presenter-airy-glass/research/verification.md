# 讲者界面轻盈毛玻璃验收

Windows 11 的原生 FerroStage 窗口已验证浅色、深色、跟随系统和独立毛玻璃开关。新配置默认浅色、开启玻璃。升级时保留旧文档里的显式明暗选择；未知配置项和玻璃关闭值经过保存、重新读取测试。

| 界面 | Windows 实测 |
|---|---|
| 首页及设置 | [浅色首页](native/light-on-home.png)、[同一首页关闭玻璃](native/light-off-home.png)、[深色首页](native/dark-on-home.png)、[浅色设置](native/light-on-settings.png)、[深色设置](native/dark-on-settings.png)。通过正常设置入口切换三种主题和材质，检查写入的值。 |
| 工作台 | [跟随本机深色](native/system-workbench.png)、[浅色](native/light-workbench.png)。重新启动后按保存的 system 和玻璃开启设置恢复。两种配色均保持预览比例、计时、翻步按钮与 1 px 分隔线的原拖动范围。 |
| 观众窗口 | 同一页分别用[系统深色](native/system-audience.png)和[浅色](native/light-audience.png)放映。在外框向内 20 个物理像素的完整内容区域逐像素比较，差异为零；原生窗边缘的激活装饰有差异。 |
| 焦点与回退 | [键盘操作后首页](native/system-keyboard-home.png)、[窄窗](native/narrow-home.png)。方向键、Tab／Space、设置开关语义、保存失败与焦点恢复另有 GPUI 回归覆盖。 |

**材质测量。** 在测试窗口后放置纯色和黑白硬边，只采集测试窗口内容。开启材质时，硬边两侧相距 72 个物理像素的场底读数分别约为 0.12 和 0.89（以两种均匀背景归一化）；关闭后背景变化在相同取样点为零。窄窗同样回退到实色。另以 `GPUI_DISABLE_DIRECT_COMPOSITION=1` 启动测试进程，保持开关为 true，变化背景时内区 RGB 像素均值差为零。Win32 的原生模糊属性没有可读回执；上述是实际合成像素的观察，不给系统模糊半径赋值。[原生数据](native/native-check.json) · [复测脚本](native_appearance.py) · [平台能力](material-capabilities.md)

**字体与首帧。** 本机没有 Figtree。初版直接请求缺失字体时，首次打开 100 页 Beamer 的中位数在玻璃开启时约为 291 ms，关闭时为 231 ms。Windows DirectWrite 在匹配不到字体时刷新字体集合。最终版一次读取应用可用字体，按已安装的用户字体、Figtree、系统 UI 字体的顺序解析；字体选择器沿用同一份列表。性能版、最终版开及关交错运行，各 6 个新进程，使用同一文稿、相同预览尺寸和固定 PDFium：

| 中位数（毫秒） | 已提交性能版的受测二进制 | 最终版，玻璃开 | 最终版，玻璃关 |
|---|---:|---:|---:|
| 接受打开 → 首张预览绘制 | 146.86 | 143.90 | 143.64 |
| 启动进程 → 测试端观察到首帧记录 | 705.46 | 684.06 | 691.50 |

这组小样本用于验证字体回退没有将首次打开的等待转移到启动阶段。操作系统 Present 和物理投影延迟未测。[原始样本与二进制哈希](native-open.json) · [复测脚本](native_open_check.py)

**自动验证与复核。** 最终 `just check-shell` 通过，235 项 GPUI 测试通过。最终 `just ci` 通过，包括工作区 fmt、严格 Clippy、测试和 cargo-deny；设置包 53 项通过。主题测试覆盖交互状态、焦点、明暗和最差背景合成对比。design-atlas 扫描结果为 Floor 0 FAIL、House 0 FAIL、Reflex 0、waiver 0；扫描器不识别 Rust，扫描文件数为 0。独立审核直接检查原生截图和设计规范，调整首页场底边距、标准工作台分隔条之后，再次复核字体修正后的截图，结论为 ship。[扫描记录](design-scan.json)

macOS 的减少透明度查询与 GPUI 原生模糊分支已接入，本轮未做 macOS 实机视觉验收。Linux 当前使用实色表面。
