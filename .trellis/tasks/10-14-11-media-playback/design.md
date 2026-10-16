# Design — 离屏媒体与共享帧

> Current acceptance policy (2026-10-15 user revision): `.trellis/tasks/10-14-presenter-v0.3/acceptance-policy.md` supersedes older hardware/matrix blockers below. Implement all features first; retain automatic/feature/native-functional checks. Physical, exhaustive native/size and performance/soak matrices are deferred, not PASS. Gate B implementation is satisfied by V16;08 may start. Gate C/D are functional integration gates. Historical results and product contracts remain unchanged.

## 选择、依赖、输入输出
前置01/02/04/06/07/08/09与Gate C；V-1…8/R-40/44。单视频后端已按末尾I-1评审锁定为独立GStreamer helper；具体版本/分发见ADR-009，不引第二后端。
输入document私有MediaSlot/source、stage/page/generation/playback epoch；输出SharedFrame控制封装与唯一音频流。audience最终仅08snapshot中的slot/rect/pixels，不读取MediaSlot/path/backend/进度。

## 文件范围
现有media/src/lib.rs与Cargo.toml、ui/src/lib.rs及08projection/texture/07actions/09settings_view、settings02schema/store、document06media_slots（必要消费适配）、app组合入口、Cargo.toml、justfile/CI、docs/packaging.md与feature-parity。
拟建media/src/{decoder,playback}.rs与选定backend适配；独立fixture媒体/生命周期fake tests。不得创建子HWND或读取Pympress实现。

## 原生研究门（实施前）
锁exact Rust/native版本、safe BGRA/callback、codec/plugin/notice/license与binary来源；cargo-deny仍拒GPL/AGPL，native DLL与Cargo图独立审计，动态链接不是自动政策豁免。优先最小合法plugin集，不拷整包。
V-8缺DLL启动是硬需求：直接import native DLL可能在进程启动前失败，不能靠gst::init Result降级。研究安全runtime loader或隔离组件方案，拿具体缺DLL启动结果再锁后端；若均不可行，提出明确ADR/需求决策后再实施，不能只测试feature-disabled build。该技术门已由末尾I-1评审和ADR-009解决：独立helper隔离所有native链接；实际GUI缺库观察仍由root集中smoke记录。

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

## I-1 accepted implementation decision — 2026-10-15

Root reviewed the concrete helper design and approved implementation. ADR-009 and `packaging/media/runtime.json` lock the single backend and selected47-file/18-plugin native closure. Native links exist only in the optional-native helper package, never the GUI. Default CI remains SDK-free. Product backend checks use `scripts/media-native.ps1`; protocol/controller and actual runtime regressions are separate from the deferred foreground/matrix/soak acceptance.

Only one active slot/decoder/audio owner exists. First public autostart wins in document order; selecting another slot retires/reaps the previous child before creating the next. A process-wide checked increasing public instance ID is used by AudienceMediaFrame.slot_id, distinct from the private PDF annotation index. A controller replacement cannot reuse an old texture key. Latest command/frame slots and bounded pipe messages prevent queue growth; stdout/stderr drains and kill/reap run off the UI. Blank/page/Stop/Close/published generation invalidate acceptance immediately; leaving blank does not resume the old instance.

The media setting remains schema v1, height1..2160/default1080, width≤1920, applies to next playback. Existing unknown/future/read-only behavior remains. Source dimensions and cumulative animation allocation are bounded. Geometry is contain into the physical annotation aspect with transparent letterbox, then conservative public intersection crop; the output canvas is bounded before allocation and only the final public BgraFrame is shared with both windows. Media uses each window's existing texture budget and is painted between the PDF and ink/laser. No source/control/notes enter Audience.

First product compilation: pure media and helper default check PASS; native helper feature check PASS; GUI gpui,pdfium,media check PASS. First actual controller→helper MP4/WebM/Ogg and GIF/WebP regression batch7 PASS, silent fakesink. These are intermediate results, not final acceptance.

Final implementation/verification and runtime identities: research/implementation-report.md. Background gates passed; visible/matrix/soak claims remain separate.

## Independent review refinements — 2026-10-15

The fixed native scanner path is the exact manifest entry `libexec/gst-plugin-scanner.exe`; a fresh isolated registry is required in package validation so a prior user registry cannot hide missing package functionality. Staging rejects existing DLL/EXE files outside the selected manifest before writing, without deleting them or changing the selected closure.

Animation pre-decode peak accounting includes retained projected frames, metadata, source/decoder/output scratch and the exact next full contain canvas plus cropped result. The geometry used for this reservation is shared with actual projection. Native child frame sequence must increase within its process; duplicate/regressing messages are typed Protocol failures and cannot replace the retained valid frame. Cross-process isolation still uses the checked opaque playback epoch. Public settings/caps and snapshot privacy are unchanged.

Final independent decisions, command receipts and exact source/binary identities are in `research/independent-review.md`. Root's consolidated foreground observations and user-deferred metrics remain separate.
