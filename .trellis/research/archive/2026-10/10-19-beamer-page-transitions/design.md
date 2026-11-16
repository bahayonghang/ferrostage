# Standard transition renderer

## Data and parsing
Extend the existing PDF structure parser and owned content metadata. Recognize the 12 styles and applicable parameters in parent research/animation-compatibility.md. Resolve references using existing parser helpers. Missing S means Replace, missing D defaults to one second, zero D means immediate replacement; reject invalid/nonfinite values with a bounded diagnostic. Preserve all supported PDF defaults. Out-of-budget metadata falls back explicitly, never silently maps an unknown effect to Fade.

Map by actual content page and overlay identity, not odd/even parity. Incoming-page metadata drives the effect. /Dur remains a separate optional dwell field owned by the timing child. GPUI/PDFium types and note metadata do not enter core.

## Shared playback contract
Shell/assembler supplies one content-only transition descriptor and unique generation/navigation identity to Audience through its snapshot. Start the monotonic effect clock only when the incoming texture is paintable. Emit identity-fenced first-visible and settled notifications for the timing child. At most one effect is active; latest target wins and late completions are ignored. Presenter current/Next/Notes update without decorative motion.

Retain only outgoing/incoming authorized slide frames within the existing GPU budget including staging/media. Existing preference staging IDs are not animation IDs. Hold the last valid public frame while waiting for a new target; no catch-up queue. Use frame callbacks only while active, no continuous idle timer. A user-requested PDF effect is deliberate content animation even when navigation came from the keyboard.

## Composition mechanisms
- Replace: immediate incoming frame.
- Fade: continuous alpha blend of complete old/new fitted content rectangles.
- Wipe, Split, Blinds, Box: directed, centered or periodic clipping masks with the correct orientation/inward-outward semantics.
- Dissolve: deterministic spatial reveal, distinct from alpha blending. Glitter adds a directional sweep over that reveal, including the authored diagonal.
- Push: both layers move. Cover: incoming moves over stationary old. Uncover: outgoing moves away from stationary new. Clip at content bounds and preserve letterbox/background behavior.
- Fly: compose the changed content with authored motion, entry/exit, scale and opaque-rectangle mode. Validate against specification and clean-room contrasting/shared-background fixtures; a full-page sliding approximation cannot pass Fly acceptance.

First prove maintained-GPUI clipping/mask/transform/difference support with synthetic frames. GPU composition is preferred. Any needed difference/mask computation is bounded and prepared once per transition, not a full PDF raster or full-frame CPU copy each tick. All retained/prepared resources count toward budgets. If a mechanism cannot be implemented within the maintained graph, report the technical block and revise the plan before claiming completion; no hidden style substitution or source patch.

## Integration and failure behavior
Blackout/curtain takes precedence; Stop/reload/topology replacement retires all effects. Pointer/drawing settles the target before applying coordinates. Active media uses immediate replacement in this release; the limitation is explicit and regression-tested. Unknown effects and GPU/resource failure replace immediately and issue existing document diagnostics. No new animation controls except timing child's contextual playback pause/resume.

## Acceptance and rollback
Use pure fake-clock tests and clean-room PDF fixtures for parameters and deterministic geometry, then real GPUI recordings for appearance/timing and cancellation. Runtime fallback is immediate replacement; scoped rollback preserves ordinary presentation. No frame-perfect Adobe equivalence claim, but every advertised style must implement its semantic effect rather than a label-only approximation.
