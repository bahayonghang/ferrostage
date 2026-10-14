# GIF 与离屏 BGRA 视频播放

父任务：`10-14-presenter-v0.3`（R-44 媒体部分）。P2。复杂任务，设计和步骤见本目录design.md / implement.md。

## Goal

Beamer 中嵌入或外链的 GIF、视频与音频能在观众屏与讲者屏同时播放，激光笔和批注可叠加在画面之上；切页立即停止并回收；没有子 HWND。

## Background

- `ferrostage-media` 现状只有 `SharedFrame` 长度检查；`MediaSlot` 元数据由 `06-beamer-document-model` 提供（`rect_norm`, `source`, `start`, `duration`, `loop`, `autostart`）。
- `AGENTS.md`：无子 HWND；共享帧 BGRA8。`docs/PROJECT_PLAN.md` §2：libVLC 回调或 GStreamer AppSink 二选一后锁定。
- 不变量 5（媒体生命周期）与 6（纹理回收）；风险 R06（4K 解码带宽 → 输出上限 1080p）。
- Rust绑定与native DLL/plugins分开审计；deny无LGPL Cargo豁免，动态链接不等于通过，exact版本/codec/notice/loader先验证。

## Requirements

- **V-1** 研究单后端exact Rust/native版本、safe BGRA、Windows分发/许可、缺DLL仍启动。GStreamer AppSink条件候选；验证safe runtime loader/隔离策略再锁，不扩大deny。
- **V-2** GIF/WebP：纯 Rust `image` 解码为帧序列与延时表，按 `MediaSlot.rect_norm` 叠加；循环播放；不走视频后端。
- **V-3** SharedFrame复用04 BgraFrame，加generation/page/slot/playback_epoch/sequence；单decoder/单音频/两窗Arc。08组装器仅slot/rect/pixels入snapshot，audience不读MediaSlot/路径/全缓存。
- **V-4** 控制：`autostart` 仅Presenting进入该页时自动播放，Ready不输出音频/媒体；`start`/`duration` 裁剪区间；`loop` 循环；点击媒体矩形切换播放/暂停（仅讲者窗口可点击）；进度显示在讲者窗口。
- **V-5** 离页/black/white/Stop/generation/Close停播+epoch失效，迟到帧拒绝；纹理沿08守卫释放。音频≤100ms与后台退出分开测，不阻塞UI join。
- **V-6** 11向v1缺省追加media.max_decode_height=1080，按比例高≤配置/宽≤1920；旧v1缺省/未知保留/未来只读。11在09高级页增控件接实际decoder，不造v2空迁移。
- **V-7** 外链路径解析：相对 PDF 所在目录；文件不存在时在媒体矩形显示占位并在讲者窗口提示，观众端只显示占位静帧或幻灯片原内容。
- **V-8** base及media-enabled缺DLL均须启动；实测移除DLL，不以gst init Result/disabled feature证明。UI禁用/占位，其他展示正常；loader不可行则阻塞研究门，不改需求。

## Acceptance Criteria

- [ ] PoC-4：双窗口同时播放 1080p60 视频 15 分钟，激光笔持续划动并绘制荧光笔，进程 GPU 专用显存波动 ≤ ±15 MiB，按design协议P95帧间隔≤20ms、丢帧≤1%，机器/媒体SHA/采样与纹理计数齐全（记录到 `research/poc-4-report.md`）。
- [ ] 夹具 D 的 `\movie[autostart,loop,start=2s,duration=8s]` 进入页面自动从 2 s 开始播放、8 s 后循环。
- [ ] GIF 夹具在观众屏与讲者屏同步播放，帧延时与文件一致。
- [ ] 音频≤100ms停止、迟到帧即时失效；后台退出目标≤1s、不阻塞UI，资源计数回零，超时有界处理。
- [ ] 删除动态库后启动成功，视频槽位显示占位；`cargo deny` 通过。
- [ ] audience仅snapshot受控slot/rect/pixels，不读SharedFrame控制包/MediaSlot/source/notes；just ci/native门通过。

## Dependencies

完整前置：01/02/04/06/07/08/09（实际目录见父Task Map）；父子关系不代表依赖。 完整接线需Gate C，可先行无硬件研究。

## Out of Scope

Beamer `animate` 帧动画；RichMedia 的 Flash/3D；字幕；录制。
