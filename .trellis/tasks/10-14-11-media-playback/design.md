# Design — 离屏媒体与共享帧

## 选择、依赖、输入输出
前置01/02/04/06/07/08/09与Gate C；V-1…8/R-40/44。单视频后端仍待research gate锁定；GStreamer safe AppSink为有证据的条件候选，LibVLC回调安全覆盖/包身份未充分核实，不引双后端。
输入document私有MediaSlot/source、stage/page/generation/playback epoch；输出SharedFrame控制封装与唯一音频流。audience最终仅08snapshot中的slot/rect/pixels，不读取MediaSlot/path/backend/进度。

## 文件范围
现有media/src/lib.rs与Cargo.toml、ui/src/lib.rs及08projection/texture/07actions/09settings_view、settings02schema/store、document06media_slots（必要消费适配）、app组合入口、Cargo.toml、justfile/CI、docs/packaging.md与feature-parity。
拟建media/src/{decoder,playback}.rs与选定backend适配；独立fixture媒体/生命周期fake tests。不得创建子HWND或读取Pympress实现。

## 原生研究门（实施前）
锁exact Rust/native版本、safe BGRA/callback、codec/plugin/notice/license与binary来源；cargo-deny仍拒GPL/AGPL，native DLL与Cargo图独立审计，动态链接不是自动政策豁免。优先最小合法plugin集，不拷整包。
V-8缺DLL启动是硬需求：直接import native DLL可能在进程启动前失败，不能靠gst::init Result降级。研究安全runtime loader或隔离组件方案，拿具体缺DLL启动结果再锁后端；若均不可行，提出明确ADR/需求决策后再实施，不能只测试feature-disabled build。是否隔离进程是技术门待评方案，不先写成实现事实。

## 数据与生命周期
GIF/WebP可用纯Rust解码路线，exact许可/feature由本任务确定；输出frames/delays，限制尺寸/总内存，帧不靠UI逐帧解码。视频safe sample mapping需正确stride/pack为BGRA，bytes owner存活，不能留下mapped借用；单decoder、单audio，两窗Arc共享。
SharedFrame复用core BgraFrame，加generation/page/slot/playback_epoch/sequence；08组装器筛generation/page/playback epoch、slot crop投影矩形后只输出AudienceMediaFrame。所有media paths仅控制侧解析相对PDF目录/Embedded bytes；缺文件/unsupported仅讲者诊断，观众保原slide或允许placeholder。
play/pause/autostart/start/duration/loop为slot状态；进入Presenting当前页才autostart，Ready不出音频/媒体；点击仅讲者。离页/black/white/Stop/Close/generation失效立即epoch失效/停音频并后台取消，迟到帧不能复活。离blank后不自动复活旧decoder，显式恢复或重新进入页触发新实例。
11向schema v1缺省追加media.max_decode_height=1080，width按比例≤1920；向09高级页加真实decoder控件，变更下一slot重建生效，旧v1/unknown/future只读契约不变。不造空v2。两个窗口texture更新用08 RAII。

## 验收、错误与回滚
fake decoder先测晚到帧/单音频/释放/切页路径；真实1080p60 15min视频及GIF、媒体激光笔/画笔叠加、时间裁剪/loop、缺DLL启动。记录OS/CPU/GPU/driver/display Hz/native插件/媒体SHA/build，热身1min后采样15min，60fps P95帧间隔≤20ms、丢帧≤1%（有定义的decoder/Present计数），GPU专用VRAM相对稳定baseline±15MiB；音频stop≤100ms、后台退出目标≤1s，不阻塞UI。
missing library是禁用媒体+typed说明，坏codec/文件只坏slot，不坏演示；安全loader/许可不可达则研究阻塞，不能忽略门禁。回滚media/UI/schema同批；旧软件可保留新增未知键，schema读取兼容。

### Shared frame allocation contract (task05 refinement)

BgraFrame::new accepts an owned Vec<u8>, validates packed BGRA geometry and stores private Arc<Vec<u8>> without copying pixels. pixels_bgra returns &[u8]; clone the BgraFrame to share an existing allocation. No mutable pixels or native handles are exposed. See parent design task05 measured frame-storage refinement; task05 owns core frame tests plus all current constructor call-site updates.
