# Journal - lyh (Part 1)

> AI development session journal
> Started: 2026-10-14

---



## Session 1: 拆分提交初始仓库并归档规范任务
<!-- trellis-session: v=2 fp=e69d58170f03896f -->

**Date**: 2026-10-14
**Task**: 拆分提交初始仓库并归档规范任务
**Branch**: `main`

### Summary

初始工作区拆成 8 个提交，并归档 00-bootstrap-guidelines。

### Main Changes

- 提交许可证、crate 骨架、just/deny、CI、文档、占位目录、Trellis 运行时和编码规范

### Git Commits

| Hash | Message |
|------|---------|
| `976ba47d6e50e66e3ab0ab4c4d2801fad3dc4afa` | chore(repo): 🔧 添加许可证与 Git 基础规则 |
| `a4b6d093f67697d77609371a0aad23c8164059d2` | feat(workspace): ✨ 建立 FerroStage crate 骨架 |
| `24ad334a69ac71af340745abab8b34eb5c6fc57b` | build(workspace): 📦 添加 just 入口与 cargo-deny 配置 |
| `7cae0354fef76586fa22361bf939c8fb3b5a18bf` | ci: 👷 添加 fmt、clippy、test 与 deny 工作流 |
| `8408b00abfbccf4c76c31215b85915a8fe69514d` | docs: 📝 添加项目说明、架构与 ADR |
| `bc5b54c0aaa4280b1d633856eab9cb39876d5b3e` | chore(repo): 🔧 添加资源、夹具与测试占位 |
| `64f4124f2de873e37d73239fb2676c3d060e770f` | chore(trellis): 🔧 接入 Trellis 工作流运行时 |
| `1613ed7c443b4bd3dfa9e021576770c72ac71379` | docs(trellis): 📝 写入前后端编码规范 |

### Testing

- [OK] cargo test --workspace --all-targets 通过，18 passed

### Status

[OK] **Completed**


## Session 2: 提交阶段实现并归档五项已完成任务
<!-- trellis-session: v=2 fp=af7c5271de6c3415 -->

**Date**: 2026-10-14
**Task**: 提交阶段实现并归档五项已完成任务
**Branch**: `main`

### Summary

提交演示器基础实现与GPUI外壳、真实检查证据；归档01/02/04/05/06，保留03/07未完成验收及08至11门禁。

### Main Changes

- 维护原始PDF/vendor/研究证据字节，LFS完整性核验通过，补全125份验收日志。
- 修复14份归档后上下文路径，保存next-session-prompt.md和目录映射。

### Git Commits

| Hash | Message |
|------|---------|
| `841e287` | feat: 实现演示器基础能力与 GPUI 外壳 |
| `f246a67` | docs(task): 记录分阶段实现与原生验收证据 |
| `92fed9f` | chore: 补全验收证据并更新归档续接入口 |

### Testing

- [OK] 归档后父与11子任务上下文12/12通过；602源绑定及125日志字节核对通过。
- [OK] 复用未变源码的GPUI49/default145、严格lint/native build和独立原生证据；上游既存空白及HarfRust lint限制保留。

### Status

[OK] **Completed**

### Next Steps

- 从03/07未完成原生验收恢复；Gate B通过后继续08，再09/10/11。不重新实施已归档任务。


## Session 3: 提交 V16 并归档基础任务，后续按功能优先实施
<!-- trellis-session: v=2 fp=dd9e985d0d11d9b3 -->

**Date**: 2026-10-15
**Task**: 提交 V16 并归档基础任务，后续按功能优先实施
**Branch**: `main`

### Summary

提交已审 AccessKit 修复和 V16 证据；按用户授权暂缓扩展实机验收，归档03/07，保存08至11连续实施Prompt。

### Main Changes

- 原始证据及延期验收完整保留，Gate B实施门通过，后续全部P1/P2功能保留。

### Git Commits

| Hash | Message |
|------|---------|
| `01dcf66` | fix: 采用 AccessKit 窗口销毁修复并保存 V16 验证证据 |
| `5634825` | docs: 调整为功能优先验收并准备全部后续任务 Prompt |
| `192fbce` | chore: 修复归档引用并保存后续实施检查点 |

### Testing

- [OK] 归档后12/12上下文通过，四份封存清单和运行产物匹配；CI151/UI82/COM8等为核验复用。

### Status

[OK] **Completed**

### Next Steps

- 从08开始，按acceptance-policy.md连续完成09/10/11；新改动默认不提交、不归档、不推送。


## Session 4: 08至11实现交付与核心验收归档
<!-- trellis-session: v=2 fp=d4af768e2cb0c596 -->

**Date**: 2026-10-15
**Task**: 08至11实现交付与核心验收归档
**Branch**: `main`

### Summary

完成工作台、完整设置、通用PDF热重载和隔离媒体播放；独立自动门禁与核心native子集已验证。用户授权提交并按实施范围归档08至11，父任务保留未验收清单，不推送。

### Main Changes

- 08至11已归档，243份研究证据字节不变，父任务作为续接入口

### Git Commits

| Hash | Message |
|------|---------|
| `15ab278` | feat: 完成工作台设置热重载及隔离媒体播放 |
| `2f95978` | chore: 保留媒体上游许可证原始字节 |
| `018f05d` | docs: 保存08至11独立检查原生证据与续接计划 |

### Testing

- [OK] 169源码身份匹配；180默认CI、119父UI加一次隔离重放、40PDFium、17媒体及严格feature/lint/typecheck/deny/build；四次核心native正常退出
- [OK] 归档后父和11子上下文12/12通过；第一方暂存空白检查通过，上游原文空白诊断保留

### Status

[OK] **Completed**

### Next Steps

- 读取父任务next-session-prompt.md，继续未观察功能与确认后的修复；网格NOT_VERIFIED、WebP桌面PARTIAL、分块producer NOT_RUN，扩展矩阵延期


## Session 5: 放置 PDFium 并让发布版双击打开
<!-- trellis-session: v=2 fp=9b6e3fd7215a7c2f -->

**Date**: 2026-10-15
**Task**: 放置 PDFium 并让发布版双击打开
**Branch**: `main`

### Summary

just dev 与 just build 放置已校验的 pdfium.dll；发布版改为 GUI 子系统。任务已归档。

### Git Commits

| Hash | Message |
|------|---------|
| `e5fee4b` | fix(scripts): 🐛 不依赖 Get-FileHash 校验 PDFium |
| `5b28c30` | feat(app): ✨ 将 PDFium 放到 exe 旁并无控制台启动发布版 |
| `f1a28fd` | docs: 📝 更新桌面启动与 PDFium 放置说明 |
| `0894bec` | docs(task): 📝 记录 PDFium 放置与发布版窗口任务 |

### Status

[OK] **Completed**


## Session 6: 显示器名称拆分提交并归档
<!-- trellis-session: v=2 fp=60d15a741398828f -->

**Date**: 2026-10-16
**Task**: 显示器名称拆分提交并归档
**Branch**: `main`

### Summary

三块 Generic PnP Monitor 改为型号加几何位置，并拆成平台、界面和任务记录三次提交。

### Main Changes

- 从 Windows FriendlyName 解析型号，读不到时不让枚举失败
- 显示器设置改成每屏一张卡片，识别浮层用同一句说明

### Git Commits

| Hash | Message |
|------|---------|
| `d368bc2` | feat(platform): ✨ 从 Windows FriendlyName 解析显示器型号 |
| `a31de58` | feat(ui): ✨ 用可区分的屏幕卡片选择显示器 |
| `febf90e` | docs(task): 📝 记录显示器名称任务 |

### Testing

- [OK] platform friendly 与 windows 枚举测试通过
- [OK] ferrostage-ui test-support 下 display 相关测试通过，clippy -D warnings 通过

### Status

[OK] **Completed**


## Session 7: 拆分提交工作台与设置设计改动并归档任务
<!-- trellis-session: v=2 fp=d4810d9dd337a316 -->

**Date**: 2026-10-18
**Task**: 拆分提交工作台与设置设计改动并归档任务
**Branch**: `codex/workbench-settings-design`

### Summary

将工作台/设置/显示器设计改动按 settings、字体选择器、工作台会话条、显示器实际位置、文案、文档、任务记录拆成 7 个中文 emoji 提交；随后归档 10-18 父子任务及 10-14、10-17、10-18 三个遗留任务，当前无活跃任务。

### Git Commits

| Hash | Message |
|------|---------|
| `ed040ae` | feat(settings): ✨ 增加界面字体与字号偏好 |
| `15d37af` | feat(ui): ✨ 增加可筛选的界面字体选择器与紧凑设置行 |
| `8c9ad60` | feat(ui): ✨ 让界面字体作用到工作台并合并会话控制条 |
| `002036e` | feat(displays): ✨ 显示窗口实际所在屏幕并标记待应用分配 |
| `31ac2fa` | feat(i18n): ✨ 补齐界面字体、工作台与显示器状态文案 |
| `b3776b5` | docs: 📝 记录界面字体、会话控制条与显示器反馈契约 |
| `2c6768d` | docs(task): 📝 记录工作台与设置设计任务及其验收证据 |

### Status

[OK] **Completed**


## Session 8: Audience click navigation and PR-only CI repair
<!-- trellis-session: v=2 fp=ab7f1b50552544a0 -->

**Date**: 2026-10-18
**Task**: Audience click navigation and PR-only CI repair
**Branch**: `dev`

### Summary

Implemented default-on audience click navigation, repaired current CI root causes with maintained Linux dependencies, completed review and archived before authorized push.

### Main Changes

- Added persistent playback click toggle and guarded Next routing.
- Restricted automatic CI to PRs, retained dispatch; repaired platform cfg and software H264 runtime.
- Migrated11 Linux D-Bus downstream packages with original-source hashes and no waiver.

### Git Commits

| Hash | Message |
|------|---------|
| `a24e4c3` | feat(ui): ✨ 支持播放窗口单击翻页及持久化开关 |
| `d199bab` | ci: 👷 仅在 PR 自动运行并保留手动入口 |
| `d0f3e6b` | fix(platform): 🐛 限制显示器名称辅助代码的编译目标 |
| `bb8cd0a` | fix(media): 🐛 补齐软件 H.264 解码并拒绝缺失解码器 |
| `e6e1a44` | fix(deps): 🐛 迁移 Linux D-Bus 依赖至维护中的 API |

### Testing

- [OK] PASS just ci:196 tests; check-shell:143 tests; all-feature deny; actionlint.
- [OK] PASS scoped native audience/media smoke and portable AT-SPI/wire checks; native Linux desktop service qualification remains separate.

### Status

[OK] **Completed**

### Next Steps

- Push dev, create PR to main, fix actual four-job CI, merge when green, then dispatch one main run and confirm issue state.


## Session 9: 完成跳页浏览与布局导航菜单优化
<!-- trellis-session: v=2 fp=1547c3d704d97d8e -->

**Date**: 2026-10-18
**Task**: 完成跳页浏览与布局导航菜单优化
**Branch**: `dev`

### Summary

按批准设计依次通过 Trellis 子代理实施和独立检查，完成可视跳页与向上布局菜单；最终全量复核通过，中文提交并归档父子三个任务。

### Main Changes

- 页码输入与真实缩略图统一浏览，支持独立 1–4 行列；菜单勾选后立即应用关闭。
- 同步设计与规范，保留原生验收截图；大型规划预览按用户要求加入 gitignore。

### Git Commits

| Hash | Message |
|------|---------|
| `b3fca4c9b6893a5330a8e78a8e4bdac1988b9b67` | feat(演示): ✨ 优化跳页浏览与布局导航 |

### Testing

- [OK] 默认198项、GPUI159项、真实PDFium回归及严格lint/依赖策略通过，最终22项身份记录匹配。
- [OK] 原生深色中文和浅色英文核心交互通过；三个归档任务上下文清单验证通过。

### Status

[OK] **Completed**


## Session 10: 时间显示与双模式跳页优化实施收尾
<!-- trellis-session: v=2 fp=fae96e8d1ec32544 -->

**Date**: 2026-10-18
**Task**: 时间显示与双模式跳页优化实施收尾
**Branch**: `dev`

### Summary

完成时间层级、导航与备注入口精简、双模式跳页及备注隔离，已提交功能并归档任务。

### Main Changes

- 当前时间移到左上角，演讲计时适度突出并补齐语义标签。
- 删除独立导航和备注方位选项，默认不拆分并识别独立备注页。
- 提供内容页和 PDF 原页码模式，备注输入不投影、不重定向，修复旧缩略图缓存与焦点揭示。
- 同步设计规范、回归测试和原生验收记录；功能提交 6ed4ae6，任务归档提交 e7489e2。

### Git Commits

| Hash | Message |
|------|---------|
| `6ed4ae67952467e23120bb2976ec688a593474e2` | feat(演示): ✨ 优化时间显示与双模式跳页 |

### Testing

- [OK] just ci：通过，包含格式、严格 Clippy、工作区测试和依赖策略。
- [OK] just check-shell：通过，实际 GPUI/PDFium 构建及 166 项 GPUI 测试。
- [OK] 固定 PDFium 原生文档/渲染套件、隔离缩略图保留和初始加载测试通过。
- [OK] 中文深色和英文浅色普通窗口通过原生检查，备注拒绝、模式换算、暂停/恢复/重置与文件只读已验证。
- [OK] 并发 PDFium 附加测试曾遇进程级绑定冲突，按既有约束隔离复测通过；不将原失败记录标为通过。

### Status

[OK] **Completed**


## Session 11: Windows 桌面 Setup、静默安装与发布流程
<!-- trellis-session: v=2 fp=616fd9e2363c19c7 -->

**Date**: 2026-10-18
**Task**: Windows 桌面 Setup、静默安装与发布流程
**Branch**: `dev`

### Summary

统一桌面构建与发布产物，生成 NSIS Setup 并通过 just install 无人值守安装到本机；完成独立检查、生命周期验证与任务归档。

### Main Changes

- 替代 Cargo-bin 安装：当前用户安装目录、桌面与开始菜单图标、系统卸载登记和原生卸载程序。
- 统一 build/release/install 的固定文件清单，生成 Setup、便携 ZIP 和 SHA256；补齐许可文本、Windows 分发 CI、双语文档及执行规范。

### Git Commits

| Hash | Message |
|------|---------|
| `544df2046a5e56a6aa813b11b2cf2e66eeb07336` | feat(安装): ✨ 添加 Windows 桌面安装与发布流程 |

### Testing

- [OK] PASS: just ci；just check-shell（166 GPUI 测试）；独立分发回归及安装文件、登记、快捷方式核对。
- [OK] PASS: 首装、重装、升级、拒绝降级、锁定文件及快捷方式失败处理、两种中途失败回滚、卸载保留设置/PDF/未知文件。
- [OK] PASS: 最终 just install 静默安装 exit 0、无手动点击；桌面快捷方式启动和真实 perf-100.pdf 渲染；PDF 哈希保持不变。
- [OK] NOT_RUN: Windows Search 界面（原生工具不能单独定位该界面）、托管 CI、干净机器与代码签名；系统 AppsFolder 和开始菜单入口已验证。

### Status

[OK] **Completed**

### Next Steps

- 可另行完成 Windows Search 界面、托管 CI、干净机器及签名资格验证；本轮未推送或公开发布。
- 保留审批拒绝删除的单个测试标记；路径及 blocked by policy 结果见归档任务 research/validation.md，未绕过限制。
