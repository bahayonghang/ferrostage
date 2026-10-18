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
