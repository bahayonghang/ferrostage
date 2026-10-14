# Design — PDF raster、调度与 native 测试

## 选择、依赖、输入输出
前置01/04。P-1…9/R-40；04提供纯BGRA/frame/descriptor。05新增pdfium feature到pdf/render并由app转发，默认纯Rust路径保留，feature开启原生fixture必跑。输入owned bytes/RenderRequest；输出owned PageMetadata/BgraFrame/Result与candidate-ready token，不输出PDFium对象。07导入也使用05只读probe元数据接口。

## 文件范围
现有pdf/render/src/lib.rs与各Cargo.toml、app/Cargo.toml、Cargo.toml、justfile、.github/workflows/ci.yml、docs/packaging.md、ADR-002。拟建pdf/src/{raster,metadata}.rs、render/src/{worker,scheduler,cache}.rs；各crate tests/golden、自建fixtures/pdf。GPU upload/RAII归08。

## 设计
动态绑定pdfium-render0.9.4，禁image转换，明确pdfium_7881兼容runtime，binary来源/build/checksum/notices另锁。绑定库不带DLL；不虚构自动下载行为或runtime环境变量支持。safe load_pdf_from_byte_vec让文档拥有bytes；worker-local Pdfium生命周期包住文档，active/candidate在worker作用域内，避免自引用unsafe。通道只传owned值。
RenderRequest/Result键为generation/page/crop/target_px/priority/epoch。队列下次出队按priority/epoch，同步正在执行项不可抢占；epoch取消排队低优与丢弃旧结果。active/candidate各有缓存/结果命名空间，PrepareCandidate先load并返回metadata，不先猜页/crop；06建图定位后PreRenderCandidate接requests再发ready，旧active不清；Commit/Abort/Retire ack供10，05用假事务测试，不提前实施监听/UI。
safe matrix/clip→raw BGRA；rotation/CropBox/top-left/alpha/stride实际验证，raw有owned copy不能宣称zero-copy。目标纵横比由content变换计算；逐行pack可解stride，禁止RGBA交换。FULL/crop对照同像素密度：左右crop Cw×Ch比FULL 2Cw×Ch，上下比Cw×2Ch。
L2/L3 bytes LRU，L1只提供预算数据/建议，GPU纹理不在render持有。大于budget单帧不缓存且正常返回，防budget下溢。font目录明确记录、默认不带待审字体；unknown PDF/缺库typed错误。

## 验收协议与回滚
纯scheduler/cache/identity测试不需DLL；native suite开启feature缺库即失败，无ignore/early-success。CI锁DLL checksum、字体/fixtures，native与默认just ci均必过；可加Windowsnative job，不伪称当前Ubuntu job已覆盖。
4K P95≤25ms目标：记CPU/GPU/driver/OS/binary/PDFSHA/页/字体，20次预热+200次worker render（不含queue/upload），median/P95；crop速度目标median≤FULL的0.8，记录所有样本。非边缘像素每通道差≤1，排除裁切边缘2px，透明合成及文字独立记录，不能因黑底相等掩盖颜色错。GPU性能到08测。
失败candidate关闭但active继续；缺库错误不改state。回滚feature/依赖须同步app/CI/packaging；共享请求改动同步06/07/08/10。
