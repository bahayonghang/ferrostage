# Implement — 05

1. **I-1 native前置研究门**：核0.9.4 safe API/feature、binary7881来源/checksum/license、owned bytes寿命、matrix/clip限制；ADR-002/packaging纠正下载假设并评审分层CI。
2. **I-2 纯协议**：定义owned metadata、request/result完整键、active/candidate metadata-first Prepare→06定位→PreRender→ready token/ack、scheduler/cache预算；fake worker测试priority出队/不可抢占/epoch/重代/候选不被误丢弃。
3. **I-3 worker/raster**：在唯一线程创建PDFium及active/candidate，安全load/render/label/close；验证wrong bytes/空/缺库typed错误，接口不泄露nativehandle。
4. **I-4 金标**：独立制作红色/透明/字体/旋转/CropBox/32:9/上下夹具（不调用产品编译器）；核stride/channel/同密度crop对照/标签，有固定环境与容差，不能用两张全黑图通过。
5. **I-5 native CI**：定义pdfium feature转发、获取经checksum DLL、独立job必须跑fixture；缺库负测试必须失败。默认pure suite和native suite结果分开，no ignore。
6. **I-6 PoC2性能**：按design协议20warm/200samples、4K worker时间与crop比值，保存环境/原始数据和P95；未达目标记录阻塞/具体评审，不用明显更快结论。
7. **I-7 接口/质量评审**：07 probe、06labels、08frame、10Prepare/Commit/Retire用例，cache回收/错误/日志/许可证评审；run下列门，PoC2与GPU状态准确区分。

拟新增feature后命令：cargo test -p ferrostage-pdf -p ferrostage-render；cargo test -p ferrostage-pdf -p ferrostage-render --features pdfium；cargo clippy -p ferrostage-pdf -p ferrostage-render --all-targets --features pdfium -- -D warnings；just ci；cargo deny check。
PDFium配置按I-1确定显式runtime路径；feature名在本任务创建，当前尚不存在。启用native且缺库失败，不记跳过为成功。
回滚：I-3加载失败Abort保旧worker；I-5feature/CI/manifest/packaging同范围回滚；不写任何用户PDF。
