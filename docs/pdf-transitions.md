# PDF presentation effects

FerroStage follows the destination content page's `/Trans` metadata during
sequential navigation. Initial Start, direct Jump, reload and window replacement
show the destination immediately. The presenter previews and notes stay static.

| Style | Composition |
| --- | --- |
| R | Immediate replacement |
| Fade | Alpha blend |
| Split | Horizontal/vertical, inward/outward split reveal |
| Blinds | Horizontal/vertical bands |
| Box | Inward/outward rectangular reveal |
| Wipe | Cardinal edge reveal |
| Dissolve | Deterministic spatial cells |
| Glitter | Spatial cells with directional sweep, including 315 degrees |
| Push | Both images move |
| Cover | Incoming moves over stationary outgoing |
| Uncover | Outgoing moves away from stationary incoming |
| Fly | Changed pixels move/scale over the stationary page; `/B` optionally fills the changed region's bounding rectangle |

`/D` defaults to one second. A zero duration replaces immediately. `/Dur` is
separate page dwell metadata. PDF angles are converted from bottom-left coordinates
to the screen's top-left coordinates. `/Dm`, `/M`, `/Di`, `/SS` and `/B` retain
their PDF defaults; malformed supported values and unknown styles produce a
document diagnostic and replacement fallback. Scale-only Fly uses `/Di /None`.

Standard Beamer overlays (`pause`, `only`, `uncover`, `visible`, `onslide`, `alt`,
`temporal` and incremental lists) remain ordinary compiled PDF pages. Built-in
Beamer `animate` / `animatevalue` sequences are supported through their pages and
`/Dur` values. Positive dwell starts after successful transition settlement, as in
PDF Reference 1.4 Figure 8.1; zero advances on a later UI turn after successful
paint and settlement. Missing or invalid dwell means manual navigation. No
intermediate step is skipped to catch up, and the final content step never wraps.

Timed documents expose one Pause/Resume animation action, independent of the talk
timer. Explicit pause persists across manual navigation. Focus loss, dialogs and
blanking preserve remaining dwell; drawing/pointer use pauses until Resume.
Reload and replaced windows invalidate old callbacks. Independent notes are
excluded from the automatic sequence. The compiled fixture and its reproduction
instructions are in `fixtures/beamer-timing/`.

Known document/page JavaScript and SetOCGState actions produce a bounded
unsupported-active-content warning while static pages remain usable. Scripts are
never executed or printed in diagnostics. Detection is deliberately bounded and
does not claim to identify every custom animation engine; static optional-content
layers alone do not trigger this warning.

Only the incoming and outgoing authorized public rasters, and for Fly one prepared
difference image, are retained. They share the existing GPU budget with staging
and media. Insufficient budget or upload failure produces a bounded runtime
diagnostic and immediate replacement when the destination can be painted. Until
then the last valid public image remains visible. Different raster dimensions
cannot use a pixel difference and explicitly fall back for Fly. Blanking, Stop,
reload, changed generation, media and drawing interrupt effects. Rapid navigation
keeps the latest destination without a queue. No PDF raster or full-frame CPU
copy occurs per animation tick.

Compatibility is semantic, not frame-for-frame Acrobat equivalence. Dissolve and
Glitter use a bounded 32 by 24 spatial grid. Arbitrary JavaScript, the separate
`animate` package, 3D/Flash and new media codecs are outside this feature.

The clean-room `fixtures/beamer/transitions.pdf` and `transitions-native.pdf`
contain 51 pages covering all families and direction/motion variants, including
enlarged Fly endpoints with `/SS 3` in all cardinal directions. Native
effects last two seconds for recordings; the fast parser fixture uses 0.1 seconds.
Regenerate with `python fixtures/beamer/generate_transitions.py`. They use PDF's
standard Helvetica font without embedding font software or private deck content.
Automated geometry and actual GPUI scene tests do not replace native visual and
dual-display acceptance; see the task evidence for the current qualification.
