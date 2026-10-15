# Design — GPUI 外壳与拓扑 PoC

## 选择、依赖、输入输出
前置01/02/03/04/05，Gate A纯契约；G-1…11/R-13/21/23/42。05仅供只读import metadata，Gate B允许占位，无06/08真实画面要求。02配置角色，不宣称09 UI可用。app组合settings/topology/import/core，ui封装所有GPUI调用；输入owned metadata/配置/事件与命令，输出窗口事务/快照/讲者告警。

## 文件范围
现有ui/src/lib.rs与Cargo.toml、app/src/main.rs与Cargo.toml、platform/src/lib.rs及03拟建topology模块（事件桥所需）、Cargo.toml、.github/workflows/ci.yml、ADR-001。
拟建ui/src/{shell,presenter,audience,actions}.rs、app运行协调模块（main过大才拆）。不直接读lopdf，05 probe供导入，不实作工作台/设置页。

## 研究与设计
优先Zed tag v1.23.3已允许路线，先spike Windows build/fullscreen/事件/keystroke；gpui-kit0.7.3为已允许替代但必须记录具体路线，不能混引两套GPUI。native DisplayId↔MonitorId桥优先，GPUI bounds为Pixels而OS为物理桌面，混DPI/旋转转换需验证；多/零匹配拒绝Start，不凭index或第一矩形。
启动load config→enumerate/resolve→讲者window（无屏等待，不unwrap）→CLI/dialog/drop经可替换DocumentImporter导入read-only probe→纯物理页descriptor→Load→Ready，占位显示页号；最近文件只属讲者设置控制侧。07仅Shell metadata，06 builder由08接到初次真实Load，不能把07称完整LoadedDocument导入。启动没有观众/计时。
WindowCoordinator处理Start：校验Ready/目标/单屏显式模式→创建初始Curtain窗口→异步确认实际目标及要求的窗口模式→成功Start并发布Live占位；失败清窗口保持Ready/计时不增长。Stop先Curtain→Stop→关闭；Close/拔Audience同路径，讲者窗口化，新屏不自动接管。
Action dispatcher是所有入口：Escape优先关闭当前模态/设置或CancelJump并消费，只有无模态/preview才匹配Stop绑定；导航等送core apply_at，Swap/设置/打开先由app协调；对话框/输入弹出前请求activate讲者并等待实际激活确认，不能同栈调用后立即弹出。非法/冲突绑定禁用冲突项并显示警告，不能多个动作同时执行；录制和规范语法给09复用。
Swap通过修改display.monitors→resolve→窗口迁移→save同一协调器；单屏/缺Audience拒绝，core无独立flag。运行时迁移失败回滚role/window，save失败保留实际会话role并标未保存。热插拔由03 owned事件，零屏/歧义及时Curtain安全停止。
AudienceView只snapshot Entity，Live占位也不接LoadedDocument；Curtain全黑无focus/UI。gpui feature app转发，nativebuild/clippy与pure CI分开，不在headless suite启动窗口。

## 错误和回滚
import/窗口创建/映射/事件失败typed报告仅讲者；原doc在坏Load时不变。新观众创建失败不能残留Presenting timer。路线/library政策研究未通过不凭猜API写调用；07回滚UI/app/platform桥与feature/CI一起，core纯状态不引GPUI。

## 视觉与交互契约

遵循根 DESIGN.md。07拥有唯一theme.rs及基础控件；08复用工作台布局；09复用主题并消费general.theme，无额外主题设置schema。暖米色/陶土色仅用于讲者UI；原始PDF/媒体/备注光栅与观众安全幕布不着色。字体使用系统替代，交互错误有文字，焦点2px且有间隔。原生窄窗/双主题/DPI截图与键盘验证按DESIGN.md逐项记录，不能以纯测试或浏览器替代。

## Task03 topology interface clarification

Consume the parent design §3 owned monitor identity and explicit confidence/scale-source fields. Windows interface paths are connection-scoped, not guaranteed stable across port changes. Geometry must come from a complete cross-checked enumeration. A TopologyEvent::Unavailable makes Audience unavailable: task07 curtains/stops/closes the audience and never continues from last-known geometry; recovery does not auto-start. Task09 shows diagnostic warnings and identity confidence. Task07 uses the strict connection-name/GPUI UUID bridge defined by the parent; no geometry fallback is allowed. GdiDeviceCapsRatio remains a diagnostic process-context ratio, never an effective-DPI or display-identity substitute; ambiguity rejects Start. Polling is owned by platform, with cancellation; no extra hidden window listener.

## Task05 owned render interface

Consume the concrete parent design §5 metadata and RenderHandle protocol. Page dimensions are visible rotated points; labels remain Option<String>. Probe/Load/Prepare run on the existing single render worker. Candidate preparation returns metadata first; document mapping chooses explicit prerender requests before CandidateReady. Results echo the full request identity; cache hits are marked separately from actual render duration. Commit/Abort/Retire acknowledgements and delayed retirement preserve the old active document through preparation. No native document or PDFium handle crosses to consumers.

### Task06 shared map/descriptor clarification

Consume parent design task06 concrete content-map interface: private validated SlideMap+crop descriptor, explicit identity compatibility for07, mapped load for08, same-generation/same-physical-count ReconfigureDocument for document viewing options without timer reset, and strictly newer-generation CommitDocument for10 reload. Reconfigure atomically remaps committed content and clears preview; malformed/stale descriptors must leave state unchanged. Labels/notes/media remain in document; snapshot never contains the map.

### Task07 pinned Windows display bridge

For the evaluated Zed v1.23.3 source route, task07 proposes mapping the existing owned MonitorInfo.connection_name to the pinned public GPUI display UUID: encode into exactly32 zero-filled UTF-16 code units, serialize big-endian bytes and compute UUIDv5 with the DNS namespace. Reject overlong/unrepresentable input, duplicate MonitorIds/names/GPUI IDs/UUIDs and incomplete set/count matches before Start. This is a pin-specific transient mapping, not a persistent MonitorId or effective-DPI estimate. No platform API expansion or geometry fallback is needed. The isolated native compile passes, but dependency policy and actual mixed-DPI window placement still require acceptance. No production route is selected until the dependency gate passes; source feasibility alone is not Gate B.

### Task07 role migration without a native position setter

The evaluated Zed tag public GPUI API has no general window-position setter. If that route clears the dependency gate, role migration prepares replacement windows using the existing presenter/snapshot entities, stages audience output behind Curtain, and verifies each actual display before retiring old windows. Keep core stage/timer/page unchanged during Swap; do not Stop/Start merely to move windows. On partial failure close replacements, restore old complete snapshot/handles and keep old authoritative role bindings. Distinguish retired-window close callbacks from current user-close events by window identity, and update Presenter focus/modal targets to the replacement handle. Save only after the runtime transaction succeeds; save failure retains session values with an unsaved warning. Pure transaction tests and real placement/focus verification remain separate evidence.

### Task07 asynchronous activation and fullscreen boundary

The independent source review found that Windows activation and fullscreen requests schedule executor work; returning from activate_window/open_window does not establish completion. The native path chooser captures the active window synchronously. The coordinator must therefore confirm the current Presenter is active before opening a chooser/modal, and keep a new Audience behind Curtain until its actual display and required window mode are confirmed. Start/time begins only after successful confirmation. Apply the same boundary to replacement-window migration before retiring old windows. Cancellation, topology changes, superseding requests or a bounded confirmation failure must invalidate pending callbacks and clean up replacements without opening a late modal or publishing Live. A cached/requested mode alone is not placement evidence. The audited public activation path is Context::observe_window_activation plus Window::is_window_active and a fresh App::active_window identity check (Windows reads GetActiveWindow at that call). For placement, Context::observe_window_bounds plus Window::bounds/display provide event-updated observations. Window::is_fullscreen alone is insufficient: the Windows flag is set before SetWindowPos, whose failure is logged rather than returned. No public completion Result/future establishes native fullscreen success. The implementation must establish a bounded confirmation protocol using actual placement observations and verify it natively; until then this remains an implementation prerequisite, not a proven solution.

## Approved final maintenance source / product I-2

The user approved the final candidate and three exact license exceptions after independent review. The authoritative set is research/maintenance-independent-report.md plus its single reconstruction entry, final source snapshot and combined maintained-package diff. Original maintenance-source.patch alone is defective historical input. Adopt the reviewed five packages in a separate vendor area, retain source/license notices and required original regression fixtures, and keep evaluation programs/generated derivative fonts out of product artifacts. Record any manifest/path-only adaptation needed for the product and review its full graph. App/ui gpui and PDFium feature boundaries, core purity and all safety contracts above remain unchanged. Product full deny and native compilation must pass before I-3. Actual shell implementation remains within07; Gate B still blocks08.

### Task07 synchronous first draw boundary

The pinned App::open_window performs a synchronous first draw before it returns. A presenter created while Shell is leased by Entity::update must not read or update that controller during render. Task07 supplies a small presenter display-data value prepared from the current Shell borrow; later model notifications refresh that value after the update lease ends. It contains only fields actually rendered, not a duplicate PresentationState, SettingsSession, worker or deck. Actions retain a WeakEntity<Shell>. Audience still receives only AudienceProjectionSnapshot. Cover the actual GPUI open_window/draw path with a feature-scoped harness and repeat native startup after rebuilding; a pure policy test does not cover this entity-borrow contract. Initial native failure and source identity are retained in07 research/poc-1-report.md and native/initial-freeze.json.

Window removal can also synchronously invoke App::on_window_closed before the calling Shell update releases its entity lease. Defer controller mutation from that callback until the current update completes, carrying the original WindowId and applying the existing current-versus-retired identity check at consumption time. Test actual GPUI removal under a coordinator lease, including timeout rollback and replaced-window cleanup. Do not suppress the callback, swallow the panic or relax placement confirmation.

The shell owns explicit application exit: closing a pending/retired window during topology cancellation or migration must not trigger last-window automatic quit. Closing the current presenter is an intentional user exit; closing the audience safely stops projection while retaining the presenter. A zero-screen state waits for topology recovery without creating an audience or exiting implicitly.

Input actions may run while GPUI has leased the originating window. Dispatch shared shell intents after that event update completes, before requesting activation, opening/replacing windows or closing projection. Do not call AnyWindowHandle::update on the same leased window from its key/mouse/accessibility callback. Exercise real input dispatch (keyboard and mouse), not only direct controller methods, and retain presenter-owner activation confirmation before a chooser.

### Native-driven input correction (2026-10-15T01:46:29.323959+00:00)

07 native v4 exposed focused Start + Space dispatching global Next and Enter doing nothing. The existing DESIGN.md Enter/Space rule is unchanged: enabled focused controls consume unmodified activation keys before presentation bindings; disabled controls stay noninteractive; actual GPUI focus/key regression must fail before repair. Shared deferred action dispatch remains mandatory.

Open uses the existing import_allowed contract consistently at button, global intent, chooser and import entry. During Presenting/incompatible transactions it must not open a chooser whose result is guaranteed to be rejected. Audience-origin refusal confirms Presenter activation before localized feedback. No mid-presentation document replacement is added. Root owns this contract clarification;07 owns the local repair.

### Topology recovery review clarification

A surviving MonitorId alone does not prove that the current presenter is on the current strictly mapped display or in the required safe window mode. After an unsafe topology condition, recover a confirmed windowed presenter using the existing replacement transaction. Distinguish first startup fullscreen preference from post-failure recovery; Unavailable/zero topology waits for a valid strict map and never automatically restarts projection. Regression covers the actual coordinator/WindowRecord event path, while physical unplug remains separate acceptance.

Jump modal editing consumes digits and Backspace before global navigation bindings. Explicitly abandoning Jump UI through Stop, Swap or modal dismissal also abandons its preview in the coordinator; it must not leave hidden navigation state. This does not change core StopPresentation semantics. Actual Shell/input regressions verify committed-page navigation after dismissal and that Escape does not additionally stop a presentation.


### Native focus and save-feedback recovery
When a focused presenter control disappears (modal Close/Commit, Start/Stop, Retry after success), restore a valid focus target in the same presenter so Tab and presentation shortcuts remain usable without a mouse click. Preserve existing valid button focus; do not steal audience focus as a side effect of unrelated model updates. Cover actual GPUI focus/key dispatch across removal, not only stored focus handles.

A failed settings save is one unsaved-state message with Retry. A successful Retry clears the corresponding stale persistence error while retaining unrelated current errors. Runtime role changes survive persistence failure; durable role changes require successful read-back. Native FileShareRead-lock evidence is distinct from mocked save errors.
