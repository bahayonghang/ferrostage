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

Capture each recent path's display-ready successful-open time with the same first-three projection. The home table shares a time-column width between header and rows, with a flexible filename/parent column and quiet separators. Format persisted UTC seconds in local time at minute resolution; missing, negative or unrepresentable values use localized unknown copy. Do not tail-truncate dates, read filesystem metadata or invent legacy dates. Keep full-path tooltip/focus disclosure and include both path and time in the accessible activation label. Test known/unknown times, actual row input and column geometry using ordinary-window font preferences; native visual smoke remains a separate check.

Shared activation owns stable identity, focus, disabled guards and deferred mouse/Enter/Space/accessibility dispatch; home-specific faces must not change shared button geometry. Full recent paths must be available to both pointer and keyboard users, with full-path accessible labels. Test actual input, modal focus return, bounded natural-width Open/rows, 320x240 scrolling, and preserved document-stage header/notes geometry. Do not infer native visual quality from headless geometry tests.

Home retains its own control focus handles and uses GPUI scroll anchors inside the home scroll container so keyboard traversal can reach controls outside the current viewport. A focus-handle assertion alone is insufficient: at 320x240 the focused control's rendered bounds must also lie in the visible client area. Keep document-stage Tab routing unchanged.

## Interface typography and session-control composition

### 1. Scope / Trigger

Changes to interface fonts, settings composition, loaded-document header/progress or Start/Stop controls must preserve root DESIGN and the shared native activation boundary.

### 2. Signatures

`theme::Typography::resolve(&GeneralSettings, &[String])` resolves family/missing-state/size scale; `font()` and `size(role)` are Presenter UI consumers. `Shell::ui_fonts` caches installed names; enumeration occurs when the picker first opens, or before first draw when a persisted override requires resolution. `Presenter::tools_panel` composes the single session/navigation/tool strip.

### 3. Contracts

Capture resolved typography in Presenter data, including tooltip consumers. Scale interface roles by base size / 16. Home wordmark keeps its Georgia role; presentation timing uses a labeled 28px mono role and the leading wall clock uses a quiet 14px mono role. Never apply these preferences to PDF/raster notes/audience pixels or OS-owned titlebar text. The bounded font selector uses installed names, filter changes do not save, and selection/default actions use checked existing preference edits. Restore focus when the picker closes. Its filter accepts ordinary text and clipboard paste; other modified commands must not append fallback letters. Snapshot paste text at the Presenter input boundary and carry it through the deferred event payload; reading the clipboard later can consume restored or unrelated contents. Keep this capture scoped to an open font picker and paste chord so shortcut recording and numeric editors are unchanged.

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

Page-browser input editing must run before configurable global navigation bindings. The focused mode-aware page-number editor supports selection/caret, replacement, Delete/Backspace, Home/End and plain-text paste; editing only locates a target. Valid Enter/Go submits one direct jump. Stop, Swap and Escape dismissal discard input without changing the committed page. Whole-thumbnail activation jumps once; hover and cancelled presses are inert. There is no staged preview/commit state. Preserve core StopPresentation semantics and cover actual Shell/input dispatch, including navigation after dismissal.

## Contextual page browser

### 1. Scope / Trigger

Progress, Jump and Page browser open one presenter-owned contextual surface above the toolbar. Preserve workbench rendering, Settings/Displays modal handling and snapshot-only audience ownership.

### 2. Signatures

`ContextualSurface::PageBrowser(PageBrowser)` is Shell-owned; browser identity includes a per-open id and document generation. `NumberEdit` owns ASCII text, selection anchor and cursor. `Intent::BrowserMode { id, mode: PageNumberMode }` changes local numbering; `BrowserEdit { id, edit }` edits a target; `BrowserJump { id, page }` submits a tile and `BrowserSubmit { id }` commits the editor through the same interaction guard. `GridDensity { column, value }` updates one dimension through an atomic paired preference apply. `CloseContextual` dismisses without navigation. Presenter captures `BrowserData` and uses its contextual host rather than reading Shell during synchronous first draw.

### 3. Contracts

The browser has Content (default) and PDF original numbering above its editor. `workbench::PageNumberMode::{Content,Pdf}` and `BrowserEntry { page: PhysicalPage, content_number: Option<usize> }` define the pure shared projection: Content enumerates actual `slide_map.steps()`, PDF enumerates all physical positions with None for excluded notes. Never divide totals by two, parse arbitrary labels as numeric identity, collapse overlay steps, or reuse `DocumentOverrides.physical_pages` for numbering. Input range, captions, target conversion/reveal, group movement and scheduling consume that same interpretation. Every committed target still requires current content-map membership before `JumpTo(PhysicalPage)`. Entering recognized notes in PDF mode shows 备注页不可投影, disables Jump and never redirects. Valid edits only reveal a group; current page, typed target and focused tile remain separate. Whole-content-tile activation dispatches once.

Column/row choices are 1..=4, default 4x4 and at most 16 visible slots. PDF-mode notes consume visible slots as non-projectable text placeholders and never request thumbnail raster data. Only selectable content entries request thumbnails with matching physical-page/generation/crop identity. `grid_columns` and `grid_visible_pages = columns * rows` commit together through existing preference persistence. Nonrectangular/tall legacy settings are labeled custom and remain unchanged on opening or unrelated save. First row choice retains columns; first column choice uses min(4, ceil(old_visible/old_columns)) rows. No schema or budget increase.

Numbering changes preserve a valid physical target by converting its visible number, or reset invalid input to the current page; they renew browser interaction identity so queued old edits/clicks cannot cross interpretations. Reopen/same-file reload remembers document-session numbering; new document import resets to Content. Numbering works with read-only settings, does not save, mutate grouping, call ReconfigureDocument or clear ink/timing.

On a browser interaction-identity change, retire Presenter GPU thumbnail cache entries as well as Workbench frames/requests before ordinary cache synchronization. `picture()` can fall back to a cached frame, so clearing only model frames permits old physical pixels under a remapped slot caption. Use the existing cache retention boundary and preserve non-thumbnail surfaces. Seed the actual Presenter cache in a regression, switch numbering before replacement rasters arrive and require Rendering/no prior image in the remapped slot.

Outside dismissal consumes the click instead of reaching slides/tools; Escape closes once without Stop. Track browser per-open identity, not only an any-surface-open flag: repeated Jump and Settings/Displays-to-browser transitions focus/select the numeric editor; browser-to-modal transitions focus the modal. Preserve focus during ordinary edits/density/tile traversal within the same browser id. Reveal a newly focused thumbnail through the Presenter focus/render lifecycle using its existing scroll anchor, guarded by the focused slot so periodic draws do not restart scrolling. A focus-handle assertion or browser-only on_focus subscription does not establish visible keyboard focus; tests must deliver the scheduled frame and check tile bounds inside the scroll viewport. Restore a valid invoker focus after closure. Stop/Home/Swap/replacement discard stale edit/target state. Closed browser has no requested or retained UI thumbnail surfaces; stale generation/map/crop work must be retired.

### 4. Validation / Error Matrix

| Input/state | Required result |
| --- | --- |
| Empty, zero, overflow or beyond active mode count | Local mode-specific range feedback, no jump |
| Physical page excluded as notes | 备注页不可投影, Jump disabled, no projection or redirect |
| Numbering changed with pending old edit/tile action | Convert/reset target; stale action rejected, no jump or ink loss |
| Valid edit, hover or cancelled press | Committed page unchanged |
| Valid Enter/Go or tile activation | One direct jump, close and focus restoration |
| Stale id/generation or busy transaction | No navigation or preference mutation |
| Read-only settings / failed save | No read-only mutation / effective values with Unsaved and Retry |

### 5. Good / Base / Bad Cases

Good: 2 columns x 3 rows persists as columns=2/count=6 and updates only browser layout until explicit jump. Base: old columns=4/count=13 opens as custom without writeback. Bad: normalize old count on open, navigate on typing/hover, or let Escape stop presentation.

### 6. Tests Required

Exercise actual GPUI input dispatch for caret/selection/paste and conflicting navigation bindings, successful submit, notes rejection, tile cancellation and outside/Escape focus recovery. Retain `page_browser_jump_refocuses_editor_after_modal_and_reopen` so switching/reopening surfaces cannot leave the numeric editor unfocused. Cover Content/PDF mapping for irregular notes, overlays and plain PDFs; target conversion/invalid reset; stale mode identities; session reopen/reset/reload; no ink/timer/grouping changes and no note thumbnail requests. Check paired preference readback with unknown TOML leaves, save failure/read-only behavior, legacy counts and real-deck request/frame retirement. Normal-window native observations remain separate from headless tests; no size/DPI matrix is implied.

### 7. Wrong vs Correct

Wrong: reuse the old global digit accumulator with a decorative border, keep duplicate full-page grid/Jump flows, or attach independent image/caption callbacks. Correct: a focused native editor, one contextual host and one explicit activation path, with captured render data and guarded typed intents.


### Native focus and save-feedback recovery
When a focused presenter control disappears (modal Close/Commit, Start/Stop, Retry after success), restore a valid focus target in the same presenter so Tab and presentation shortcuts remain usable without a mouse click. Preserve existing valid button focus; do not steal audience focus as a side effect of unrelated model updates. Cover actual GPUI focus/key dispatch across removal, not only stored focus handles.

A failed settings save is one unsaved-state message with Retry. A successful Retry clears the corresponding stale persistence error while retaining unrelated current errors. Runtime role changes survive persistence failure; durable role changes require successful read-back. Native FileShareRead-lock evidence is distinct from mocked save errors.

## Layout and navigation contextual menu

### 1. Scope / Trigger

The loaded-workbench Layout & navigation trigger opens an upward menu without replacing the workbench. It shares the contextual state, backdrop and dismissal lifecycle with the page browser. Settings/Displays retain their existing flows.

### 2. Signatures

`ContextualSurface::LayoutNavigation { id: u64, generation: DocumentGeneration, error: Option<String> }` stores open identity and local error. `Shell::layout_menu_open()` and `reconfigure_layout(id, DocumentOverrides, cx)` own guarded eligibility and changes; `presenter::layout_navigation::LayoutMenuData` captures requested/detected layout, annotation presence and local errors. `Presenter::contextual_host` accepts trigger bounds and anchor alignment. `PresentationState::has_annotations() -> bool` is a read-only query for any stroke with at least one point on any page or a live laser, not only current-page snapshot strokes.

### 3. Contracts

Three visible rows remain: Automatic and No split as mutually exclusive radio choices, then Identify independent notes pages as a checkbox. New `DocumentOverrides::default()` uses `layout=Some(NotesLayout::None)`, independent-note recognition true and physical grouping false. Automatic is explicit `layout=None`; its detected directional label must be independent of the selectable rows. Remove directional choices and the physical-navigation checkbox, preserving internal crop and overlay-grouping capabilities. Browser numbering is independent of those overrides. Use localized labels/helpers and retain real document/structure warnings without a Debug dump.

Identical overrides close immediately without document building, core reconfiguration or clearing frames/annotations. Deferred selection carries the per-open menu id and requires its captured generation to match current state. Actual change builds a candidate document/descriptor, gets core acceptance, assigns the session document, invalidates frames, publishes the updated projection and closes. Failure preserves effective state/checks and keeps a local error in the open menu. Overrides remain session-only. When any annotations exist, explain the unchanged core clear-on-reconfiguration effect before activation; the getter must not report empty strokes or retained empty per-page containers as visible ink.

Anchor the menu right-aligned 8px above its measured trigger with a 12px viewport margin. Use a preferred 320px width, existing palette, >=44px row targets, one reserved check column and visible focus. Internal scrolling handles unavailable vertical space without reflowing the workbench. Arrow/Home/End and Tab navigate visible menu controls, Enter/Space activates once, Escape/outside dismiss without Stop/click-through, and focus returns to the trigger. Opening another contextual surface, Stop/Home/Swap/replacement dismisses old menu state.

### 4. Validation / Error Matrix

| State/action | Expected behavior |
| --- | --- |
| Choose current layout | Close only, strokes/frames unchanged |
| Choose another layout or flip a toggle | Apply immediately, close, correct checks on reopen |
| Candidate/core rejection | Old state retained, menu stays open with local error |
| Annotation on another page | Clear-on-change notice still visible |
| Empty erased-stroke list | No false ink notice |
| Pending native transaction | No mutation or underlying shortcut leakage |
| Escape/outside/ordinary menu navigation | No Stop, slide advance or drawing |

### 5. Good / Base / Bad Cases

Good: a fresh document checks No split; explicit Auto can report detected Right without requiring a selectable Right row. Base: selecting the current requested layout does not erase off-page annotations. Bad: always reconfigure on selection, show raw true/false/Debug values, or infer all-document annotations from the current audience snapshot.

### 6. Tests Required

Use real GPUI input for the three visible rows, arrows/Tab/Home/End wrap, Enter/Space, apply/reopen, independent-note recognition, outside/Escape and focus recovery. Deleted rows have no hidden focus or action target. Reactivating each drawing/pointer tool returns to internal neutral Navigate, ends live strokes/laser, and needs no standalone Navigate button. Exercise same-layout no-op, induced transaction failure and off-page annotation notice. Retain browser mutual exclusion, independent-notes privacy, core reconfiguration semantics and first-render borrow regressions. Default core tests cover the read-only getter; normal-window native screenshots establish actual menu placement/live visual result separately.

### 7. Wrong vs Correct

Wrong: add a second modal system, save document overrides as global settings, change core clear policy or expose note state to audience. Correct: captured menu data in the existing contextual host, checked session overrides and a minimal core annotation-presence query, with deferred Shell commands and no new UI/IO core dependencies.
