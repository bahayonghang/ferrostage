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

## Synchronous window creation

The pinned GPUI App::open_window draws the first frame synchronously. During a Shell entity update, creating a presenter whose render calls Shell::read panics because the controller is already leased. Prepare the minimal rendered data from the current controller borrow, pass it into the presenter, and refresh it from model notifications after the update ends. Keep commands on WeakEntity<Shell>; do not clone the entire controller or document into the view. Audience remains snapshot-only.

Regression evidence must exercise real GPUI open_window/first draw or an actual native launch. A successful policy helper test or compilation cannot establish safe entity borrowing. Preserve the failing runtime receipt and bind the retest to the rebuilt executable. Do not add a broad catch-unwind or skip first rendering to hide this error.

Window removal can also synchronously invoke App::on_window_closed before the calling Shell update releases its entity lease. Defer controller mutation from that callback until the current update completes, carrying the original WindowId and applying the existing current-versus-retired identity check at consumption time. Test actual GPUI removal under a coordinator lease, including timeout rollback and replaced-window cleanup. Do not suppress the callback, swallow the panic or relax placement confirmation.

The shell owns explicit application exit: closing a pending/retired window during topology cancellation or migration must not trigger last-window automatic quit. Closing the current presenter is an intentional user exit; closing the audience safely stops projection while retaining the presenter. A zero-screen state waits for topology recovery without creating an audience or exiting implicitly.

Input actions may run while GPUI has leased the originating window. Dispatch shared shell intents after that event update completes, before requesting activation, opening/replacing windows or closing projection. Do not call AnyWindowHandle::update on the same leased window from its key/mouse/accessibility callback. Exercise real input dispatch (keyboard and mouse), not only direct controller methods, and retain presenter-owner activation confirmation before a chooser.

## Focused control activation and import eligibility

An enabled focused presenter button consumes unmodified Enter/Space before global presentation shortcuts. A Space binding for Next must still work when no control owns that key. Disabled controls are skipped by keyboard navigation and cannot dispatch through mouse, keyboard or accessibility. Test actual GPUI focus/key dispatch rather than invoking the button callback directly. Keep modal consumption ahead of global commands and retain the shared deferred coordinator dispatch boundary.

The same import eligibility contract controls Open buttons, global Open actions, chooser creation and direct imports. Presenting or an incompatible window transaction must not open a chooser that can only reject its result later. An audience-origin intent that needs to show a refusal first confirms the current presenter is active, then shows the localized reason there. It must never place that message on the audience or weaken begin_import validation.

Jump input digits/Backspace must reach modal editing before configurable global navigation bindings. When the coordinator explicitly discards that modal on Stop, Swap or Escape dismissal, cancel its preview too; otherwise future navigation may silently affect a hidden preview. Keep core StopPresentation semantics unchanged, and cover committed-page navigation after dismissal with actual Shell/input regression.


### Native focus and save-feedback recovery
When a focused presenter control disappears (modal Close/Commit, Start/Stop, Retry after success), restore a valid focus target in the same presenter so Tab and presentation shortcuts remain usable without a mouse click. Preserve existing valid button focus; do not steal audience focus as a side effect of unrelated model updates. Cover actual GPUI focus/key dispatch across removal, not only stored focus handles.

A failed settings save is one unsaved-state message with Retry. A successful Retry clears the corresponding stale persistence error while retaining unrelated current errors. Runtime role changes survive persistence failure; durable role changes require successful read-back. Native FileShareRead-lock evidence is distinct from mocked save errors.
