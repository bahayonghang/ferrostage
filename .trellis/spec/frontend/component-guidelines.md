# Component Guidelines

A "component" is a GPUI view or element, not a React function component. There are no props objects and no JSX.

## Rules

- One view owns one window role (`WindowRole`).
- The audience view's render input is `AudienceProjectionSnapshot`.
- The presenter view may also read `ferrostage_document::SlideNotes`.
- Video is a `ferrostage_media::SharedFrame` drawn as a BGRA image. Do not embed a child window.
- Stable element ids, once GPUI is linked. Do not rebuild identity from a translated label.

## Native dependency boundary

Task07 links the approved maintained tagged GPUI source through an optional feature. Follow [Maintained Dependencies](maintained-dependencies.md) and ADR-001/008; do not substitute the historical unpatched route or link a second GPUI stack. Pure/default checks remain separate from GPUI native compilation and running-window verification.

## Forbidden

- React, `dioxus`, `egui`, or a webview for the slide surface.
- crates.io `gpui` 0.2.x.
- Reading the PDF file from a view. Ask `ferrostage-pdf` / `ferrostage-render`.

## Presenter design source

Root `DESIGN.md` defines the Claude-inspired FerroStage presenter theme. Task 07 creates the single `ui/src/theme.rs` token layer; 08 and 09 consume it. Use system fonts, explicit control states and visible keyboard focus. Never theme PDF/media pixels or pass settings/theme controllers to the audience snapshot. Source token contrast evidence does not establish native UI verification.

### Idle home composition

`Presenter::home_content` consumes captured `PresenterData`, derives recent basenames/parents without filesystem reads, and retains the three-entry recent limit and original `Intent::Import` paths. Idle has one `welcome-open`, one dedicated wordmark, quiet Settings/Displays and flat recent rows. It omits document header/timer/Start footer; Ready/Presenting retain those controls and their existing state. Keep common import/topology/save feedback and Retry inside the scrollable home column.

Shared activation owns stable identity, focus, disabled guards and deferred mouse/Enter/Space/accessibility dispatch; home-specific faces must not change shared button geometry. Full recent paths must be available to both pointer and keyboard users, with full-path accessible labels. Test actual input, modal focus return, bounded natural-width Open/rows, 320x240 scrolling, and preserved document-stage header/notes geometry. Do not infer native visual quality from headless geometry tests.

Home retains its own control focus handles and uses GPUI scroll anchors inside the home scroll container so keyboard traversal can reach controls outside the current viewport. A focus-handle assertion alone is insufficient: at 320x240 the focused control's rendered bounds must also lie in the visible client area. Keep document-stage Tab routing unchanged.

## Interface typography and session-control composition

### 1. Scope / Trigger

Changes to interface fonts, settings composition, loaded-document header/progress or Start/Stop controls must preserve root DESIGN and the shared native activation boundary.

### 2. Signatures

`theme::Typography::resolve(&GeneralSettings, &[String])` resolves family/missing-state/size scale; `font()` and `size(role)` are Presenter UI consumers. `Shell::ui_fonts` caches installed names; enumeration occurs when the picker first opens, or before first draw when a persisted override requires resolution. `Presenter::tools_panel` composes the single session/navigation/tool strip.

### 3. Contracts

Capture resolved typography in Presenter data, including tooltip consumers. Scale interface roles by base size / 16. Home wordmark keeps its Georgia role; timer keeps its mono face. Never apply these preferences to PDF/raster notes/audience pixels or OS-owned titlebar text. The bounded font selector uses installed names, filter changes do not save, and selection/default actions use checked existing preference edits. Restore focus when the picker closes. Its filter accepts ordinary text and clipboard paste; other modified commands must not append fallback letters. Snapshot paste text at the Presenter input boundary and carry it through the deferred event payload; reading the clipboard later can consume restored or unrelated contents. Keep this capture scoped to an open font picker and paste chord so shortcut recording and numeric editors are unchanged.

Recent-file rows also retain their natural content height in the settings flex column. Do not shrink a filename/path pair to its minimum control height: at a saved larger UI font, the path can overflow and be clipped even after scrolling it into view. The owning settings scroll handles excess height; verify the path's rendered bounds against its row and viewport.

Settings use quiet category navigation, sans headings, bounded grouped label/control rows and real recent basename/parent paths. Preserve all category controls and history semantics. Local restart notes and save/Retry feedback remain truthful. Setting rows must not flex-shrink below their label/help content; use the existing host scroll for overflow rather than clipping helper text at the hairline. Recent settings rows do not acquire the home row's import action or top-three cap.

For every document-open state, one Start/Stop action shares a navigation group with Previous/Next when steps exist. Retain the no-step route and busy guards. Session face uses a geometric play/stop icon plus explicit text. Progress remains `Intent::Jump`; full logical/step/physical detail is accessible and available on pointer/focus, without reflowing the activation target. Focus detail and hover tooltip are mutually exclusive when both pointer and keyboard focus occupy the control. Explicit progress focus handles opt into Tab eligibility. Blanking/tool selection is derived from actual state and includes text/indicator, not color alone.

### 4. Validation / Error Matrix

| Condition | Result |
| --- | --- |
| Requested font unavailable | System/CJK fallback plus local notice; saved request is retained |
| Future/read-only settings or busy state | Existing disabled/edit guards, no unchecked mutation |
| Ready/busy/no steps | Correct session control remains; busy Start cannot create transaction/run timer |
| Progress focused or hovered | Full detail discoverable without moving the control |
| Jump text edited but not confirmed | Committed page stays unchanged |
| Modal or conditional control closes/disappears | Restore valid focus, retain exactly-once deferred activation |

### 5. Good / Base / Bad Cases

Good: real settings input changes actual captured Presenter typography and survives store readback; a page-progress click opens the original jump flow. Base: defaults preserve document content and the home brand. Bad: grow a detail row on focus, accidentally remove a tracked control from Tab order, or leave a second full-width Start footer.

### 6. Tests Required

Retain real input tests `settings_fonts_filter_apply_size_restore_and_history_through_real_input`, `workbench_controls_jump_directly_and_keep_note_pages_private`, and `workbench_session_strip_survives_empty_steps_and_busy_start`, plus existing home/focus/notes/privacy regressions. Run normal-window native core/visual smoke separately from headless checks. Do not add size/DPI/performance matrices to this polish work without user request.

### 7. Wrong vs Correct

Wrong: style independent copies of Start/progress and global text without respecting state or role boundaries. Correct: one session strip, captured role-aware typography, common stable activation IDs and original Intents. Do not infer native visual quality or installed-font behavior from passing headless tests.

## Synchronous window creation

The pinned GPUI App::open_window draws the first frame synchronously. During a Shell entity update, creating a presenter whose render calls Shell::read panics because the controller is already leased. Prepare the minimal rendered data from the current controller borrow, pass it into the presenter, and refresh it from model notifications after the update ends. Keep commands on WeakEntity<Shell>; do not clone the entire controller or document into the view. Audience remains snapshot-only.

Regression evidence must exercise real GPUI open_window/first draw or an actual native launch. A successful policy helper test or compilation cannot establish safe entity borrowing. Preserve the failing runtime receipt and bind the retest to the rebuilt executable. Do not add a broad catch-unwind or skip first rendering to hide this error.

Window removal can also synchronously invoke App::on_window_closed before the calling Shell update releases its entity lease. Defer controller mutation from that callback until the current update completes, carrying the original WindowId and applying the existing current-versus-retired identity check at consumption time. Test actual GPUI removal under a coordinator lease, including timeout rollback and replaced-window cleanup. Do not suppress the callback, swallow the panic or relax placement confirmation.

The shell owns explicit application exit: closing a pending/retired window during topology cancellation or migration must not trigger last-window automatic quit. Closing the current presenter is an intentional user exit; closing the audience safely stops projection while retaining the presenter. A zero-screen state waits for topology recovery without creating an audience or exiting implicitly.

The shell also owns the foreground tasks that await its recurring timer, document read, and file-chooser result. Retain their cancellable task handles and cancel them synchronously on intentional presenter exit and application quit, before the platform dispatcher is destroyed. A weak entity or request-token check after an await is insufficient: resuming that check already posts a platform wake. Keep those checks for results delivered while the application is alive. Cancelling a foreground continuation does not promise interruption of an already running filesystem read or synchronous teardown of the native dialog. Closing an audience or retiring a replacement must keep the live shell poller running. Test pending timer/topology, read and chooser delivery after actual GPUI shutdown, and verify the active application still handles them normally. Native close acceptance separately rejects late dispatcher invalid-handle diagnostics even when process exit is zero; do not suppress those errors or extend platform-package patches to hide them.

Shell task cancellation and platform callback retirement are different boundaries. A platform callback may arrive after its GPUI Window has been removed even when Shell has cancelled every owned task. The maintained GPUI callback registration group must share the owning-window lifetime check, including a second check when deferred work executes. Preserve live callback errors and neutral return values for retired input/hit-test requests. Tests retain the actual platform window/callback, remove the GPUI Window, then deliver callbacks; live leased-window controls must still expose errors. A clean close in one run cannot erase a later failing callback or replace full post-exit log review.

Intercept native title-bar close requests through `Window::on_window_should_close`, return false to prevent the Windows default OS-first destruction, and schedule removal of that original handle after the callback/window lease ends. Let the existing deferred closed observer classify the original WindowId; do not duplicate role policy or remove synchronously from the should-close callback. A real GPUI close-request regression proves interception and coordinator behavior; a source-bound native retest must separately reject double-destruction/invalid-handle diagnostics.

Input actions may run while GPUI has leased the originating window. Dispatch shared shell intents after that event update completes, before requesting activation, opening/replacing windows or closing projection. Do not call AnyWindowHandle::update on the same leased window from its key/mouse/accessibility callback. Exercise real input dispatch (keyboard and mouse), not only direct controller methods, and retain presenter-owner activation confirmation before a chooser.

## Focused control activation and import eligibility

An enabled focused presenter button consumes unmodified Enter/Space before global presentation shortcuts. A Space binding for Next must still work when no control owns that key. Disabled controls are skipped by keyboard navigation and cannot dispatch through mouse, keyboard or accessibility. Test actual GPUI focus/key dispatch rather than invoking the button callback directly. Keep modal consumption ahead of global commands and retain the shared deferred coordinator dispatch boundary.

The same import eligibility contract controls Open buttons, global Open actions, chooser creation and direct imports. Presenting or an incompatible window transaction must not open a chooser that can only reject its result later. An audience-origin intent that needs to show a refusal first confirms the current presenter is active, then shows the localized reason there. It must never place that message on the audience or weaken begin_import validation.

Jump input digits/Backspace must reach modal editing before configurable global navigation bindings. The page dialog edits a number without navigating; valid Enter/Go submits one direct jump. Stop, Swap and Escape dismissal discard the input without changing the committed page. Thumbnail hover must never navigate; an explicit click chooses the page directly. There is no staged preview/commit state. Preserve core StopPresentation semantics and cover actual Shell/input dispatch, including navigation after dismissal.


### Native focus and save-feedback recovery
When a focused presenter control disappears (modal Close/Commit, Start/Stop, Retry after success), restore a valid focus target in the same presenter so Tab and presentation shortcuts remain usable without a mouse click. Preserve existing valid button focus; do not steal audience focus as a side effect of unrelated model updates. Cover actual GPUI focus/key dispatch across removal, not only stored focus handles.

A failed settings save is one unsaved-state message with Retry. A successful Retry clears the corresponding stale persistence error while retaining unrelated current errors. Runtime role changes survive persistence failure; durable role changes require successful read-back. Native FileShareRead-lock evidence is distinct from mocked save errors.
