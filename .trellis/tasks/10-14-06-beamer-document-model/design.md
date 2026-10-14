# Design — Beamer 内容与备注映射

## 选择、依赖、输入输出
前置01/04/05；B-1…8/R-41。05提供rotation后页尺寸、Option<String> labels和owned只读结构；document构建LoadedDocument/NotesMapping/MediaSlot，core只持content SlideMap/geometry，不依赖lopdf。
输入metadata、结构和document-local overrides；输出core content steps/logical groups + document备注配对/诊断/媒体槽位。标签是显示/线索而非索引。

## 文件范围
现有document/src/lib.rs与Cargo.toml、core/src/{lib,ids,geometry,state,command}.rs、pdf/src/lib.rs与Cargo.toml、docs/beamer-notes.md。
拟建core/src/slide_map.rs、document/src/{builder,notes,media_slots}.rs、pdf/src/structure.rs；fixtures/beamer及就近模型/结构tests。不引入LaTeX编译器或媒体backend。

## 设计
父design §7统一比例候选：左右[3.1,3.6]/[2.4,2.7]、上下[0.85,0.95]/[0.63,0.70]；根据宽×2/高×2推导，geometry补TOP/BOTTOM_HALF。页尺寸混合/旋转不一致→None+warning。自动Right/Bottom只是默认约定，无法证明备注存在；None/Left/Right/Top/Bottom覆盖最终权威。新Load重置override，同文件重载保留；独立notes识别与overlay分组均有doc-local开关。
labels缺失/空串逐content页独立，不用合成显示号再配对；非数字原文保留，01与1不同。连续非空同label内容步骤启发式成组，notes夹页不打断content组；非连续重号不全局合并，physical_pages关闭分组。重复label跳转返回候选，不任取第一。
notes:<suffix>默认剔出content，只和紧邻此前content物理步原始label精确匹配；连续多notes组成document Vec，首notes/错label为orphan warning。core SlideMap不含notes页/crop/text；content empty返回typed error且旧文档保留。用户关闭独立识别时可手工恢复误判的普通标签页。
NextStep按content序列首尾钳制；closest_content_at_or_before(physical)取≤号最大content物理页、无则首content，供08/10使用；logical导航到组首。可逆属性须有对应邻居，logical逆向只比组。06更新core消费map并保持04preview isolation。保留按page_count构造identity SlideMap的纯descriptor兼容入口供07独立Shell PoC；08真实导入必须用builder显式传content map，不能保留identity默认作为真实Beamer结果。
媒体只读支持子集须独立核PDF规范字段层级：Movie/Screen/RichMedia，Rect从PDF底部原点转top-left、crop内槽位坐标再投影，Path/Embedded只在document。缺字段有None/warning，未支持Flash/3D不播放；不承诺所有导出autostart/time均能还原。

## 错误、回滚
metadata/count不一致、empty content、越界map拒绝；孤立/重复/unsupported只讲者diagnostic，观众不获notes/path。自建fixtures已知半幅内容，不依赖允许文档外Pympress源码。回滚map API同时core/document调用者；bad build不改变旧LoadedDocument。
