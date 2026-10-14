# Implement — 11

1. **I-1 后端/加载/许可研究门**：基于runtime证据锁一套exact bindings/native/plugin/codec，safe BGRA与缺DLL启动原型验证；产生backend与分发清单结论再评审，不以AppSink存在当全部可行。
2. **I-2 纯生命周期**：fake decoder，generation/page/slot/playback_epoch/sequence与SharedFrame复用BgraFrame，单audio、迟到帧过滤、black/white/Stop/Close/重载事件立即停；Ready不播放。
3. **I-3 GIF/视频适配**：GIF延时/loop，单native decoder安全stride/pack、start/duration/autostart；media source路径/Embedded只控制侧，missing/unsupported占位。
4. **I-4 UI帧接线**：08唯一投影组装器补slot/rect/pixels，两窗Arc共享/统一RAII，讲者play/pause/进度，不把source/notes/control包给观众；overlay/blank层次截图检查。
5. **I-5 设置/缺库分发**：v1 media默认+unknown兼容/future readonly，09新增真实控件与disabled说明；media-enabled binary实际移除native DLL后启动/正常PDF功能，CI分别base/nativebuild/nativefixture，许可证清单/deny通过。
6. **I-6 PoC4**：design1080p60预热1min+15min协议、GIF同步/loop/裁剪、持续pointer/画笔、stop音频≤100ms/后台≤1s、VRAM/帧间隔/丢帧；记录原始数据与机器，不提前建“通过报告”。
7. **I-7 Gate D媒体评审**：AC-5/9、与10已集成时generation停播回归、texture/resource计数、用户PDF只读/无child HWND；跑命令并明确未完成native/GPU项。

拟在I-1选择后定义media feature/app转发；命令：cargo test -p ferrostage-media -p ferrostage-settings -p ferrostage-ui；cargo build -p ferrostage-app --features gpui,pdfium,media；cargo test -p ferrostage-media --features media；just ci；cargo deny check。
native缺库启动实验不能被默认pure门替代，动态库/插件另做分发审计。回滚backend/feature/CI/UI/schema同批，save失败不动旧settings，decode失败保slide。尚无后端锁定/实机报告，本次不启动。
