# Implement — 06

1. **I-1 研究门**：读behavior证据/PGF尺寸推导，核05metadata/rotation；主源PDF规范核媒体支持字段路径，用自建夹具确定子集，不能从Pympress README反推实现。
2. **I-2 纯content map**：core新增SlideMap/TOP/BOTTOM，document构建器；空/缺失/非数字/连续与非连续重号，首尾及有邻居逆向测试，logical逆向仅比组。
3. **I-3 layout/notes**：实现比例候选/来源warnings/五种手动覆盖、独立识别开关、physical_pages开关、按物理步多notes配对；宽图/竖版误判和混合尺寸反例必须测试。
4. **I-4 只读结构**：Movie/Screen/RichMedia支持子集/Rect/Embedded/source/time字段，构建字段路径明示的自制PDF，missing/unsupported可诊断，不制造默认“已还原”。
5. **I-5 core/document接线**：从04descriptor升级map元数据，导航/preview用content，project仍无notes；更新所有现有core/document测试，不将整个LoadedDocument交观众。
6. **I-6 PoC3**：A常规/overlay，B左右与上下，C独立多notes/orphan，D媒体参数，E普通宽图/重复标签；记录期望map/crops/warnings与actual，真实PDF另作限制验证。
7. **I-7 评审**：core纯依赖/notes与paths隔离、范围与手动覆盖、结构只读/无compiler，跑命令；05实际native提供labels/尺寸的组合测试通过。

命令：cargo test -p ferrostage-core -p ferrostage-document -p ferrostage-pdf；cargo test -p ferrostage-pdf --features pdfium；cargo tree -p ferrostage-core；just ci。
回滚：map/descriptor跨core/document同次；candidate结构失败保旧文档。夹具写入独立temp目录或本项目自制fixture，用户PDF哈希不变。
