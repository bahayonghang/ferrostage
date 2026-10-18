# Research — 仓库基线与冲突处置

核对日期2026-10-14；本轮从无active task、main分支开始。父和11子均planning，任务目录全部存在；未创建新任务。初始dirty为docs/PROJECT_PLAN.md及12个未跟踪任务目录，保留既有改动。没有运行产品测试/PoC；以下是源码/manifest/ADR事实。

## 已验证事实
- Cargo.toml workspace有9个crate；edition2024/rust-version1.99，rust-toolchain.toml channel1.99.0，workspace unsafe_code=forbid。
- pdf只链接lopdf0.45.0结构检查；platform链接notify8.2.0并命名RecommendedWatcher，无watch循环/枚举。toml1.1.8已链接。
- pdfium-render0.9.4、Zed v1.23.3/gpui-kit0.7.3、hayro0.8.0仅注释/常量，不是可用集成；native库不自动由PDFium绑定提供，主源证据见runtime-feasibility。
- core state.rs:37的new直接设置第一页，无stage；:89投影使用jump_preview，:129–133普通step隐式提交preview；:181影子提交仅generation/nonempty，不更新page_count/map或资源。
- settings/lib.rs:14–21六个扁平键，仅from_toml；document/lib.rs:27只page_count/notes/media；media/lib.rs:13 SharedFrame仅尺寸+BGRA长度检查。
- render/lib.rs:5–7注释优先级仍旧表，只有cache常量/is_current；UI lib.rs只pin与WindowRole。不存在金标PDF/实机报告。
- ADR001–005是骨架政策，不是实现验收；.github/workflows/ci.yml仅Ubuntu fmt/clippy/test/deny，未配置native/GPU。
- 父children与各child.parent双向一致；task.json及已有implementation-prompts.md保持，不把Prompt B复制指令视为本轮实施授权。

## 基线扫描命中与责任（行号是本轮编辑前位置）
扫描范围AGENTS、docs、README、crates，另读相关spec；匹配按语义分类，不要求历史文字零匹配。01实施时重扫当前行号并形成最终命中清单。

| 基线命中 | 分类 | 处置 / 所有者 |
| --- | --- | --- |
| docs/PROJECT_PLAN.md:12 latexmk/xelatex旧方案 | 历史 | 保留历史，已改“删除现行正向功能”；01/最终AC-4审语义 |
| docs/PROJECT_PLAN.md:13 FreeText/Annots写回 | 历史删除说明 | 保留，不恢复功能；只读媒体结构不属编辑 |
| docs/PROJECT_PLAN.md:259旧xelatex PoC-5 | 明确排除 | 保留删除事实；替换测试限定独立temp |
| AGENTS.md:18 PDF tmp/rename | 现行政策冲突 | 01改用户PDF只读、设置原子保存，本次不实施 |
| docs/adr/ADR-004-lock-free-reload.md:13 PDF writer | 现行政策冲突 | 01删PDF写回政策、10完善外部替换事务 |
| .trellis/spec/backend/database-guidelines.md:29 PDF tmp writer | 现行spec冲突 | 01同步只读政策/设置保存边界 |
| crates/ferrostage-platform/src/lib.rs:16–20、:54 SHADOW_TMP_SUFFIX | 未用产品常量/误导测试 | 10删除常量与写PDF注释/相关测试，设置独立Store；不能只改doc就称代码已合规 |
| .trellis/spec/backend/directory-structure.md:19平台atomic rename | 归属旧说明 | 01区分PDF监听只读/设置Store保存 |
| frontend/state-management.md:10–12 settings只读、swap flag | 现行spec冲突 | 01同步运行时设置/唯一binding权威，04移core flag，07协调交换 |
| frontend/quality-guidelines.md:26 missing monitor回主屏 | 可能Audience错误回退 | 01明确仅讲者可回退，保存Audience缺席不接管 |
| render/lib.rs:5–7与architecture.md旧优先级表 | 现行描述冲突 | 01文档同步，05代码注释/worker统一P0观众/P1讲者/P2±1/P3preview/P4thumb |
| feature-parity.md Hot reload P0/Video P1/指针P0 | 功能/排期混用 | 01对准PROJECT_PLAN，不能将08排期P1映射成全部功能P1 |
| 初始03局部allow，05ignore/自动下载，10线程外PDFium | 不可执行规划 | 本轮已改PRD/design/steps；runtime证据支撑，native/实机仍待验 |

README/README.zh-CN在初始禁用词扫描无命中；媒体/结构只读访问Annots允许。写入API须追目标路径，而非按write字样拒绝所有设置/测试写入。当前产品常量/规范未修是01/10未来工作，不声称本轮完成产品AC-4。

原PRD保留证据锚点：crates/ferrostage-core/src/state.rs:17–28/37–52、command.rs:12–29、snapshot.rs:19–26；crates/ferrostage-settings/src/lib.rs:14–21/15–16；crates/ferrostage-platform/src/lib.rs:11–23。它们描述本轮未改变的骨架，不描述规划后的实现。

## 来源及限制
行为表只读取ref/pympress/README.md、ref/pympress/docs/options.md；没有Python实现/automodule/模板/UI资源/测试注入。研究的库API只取主源，结果分确认/建议/未验证。没有三屏、PDFium时间、GPU显存、真实媒体播放数据，全部硬件目标待实施门。
