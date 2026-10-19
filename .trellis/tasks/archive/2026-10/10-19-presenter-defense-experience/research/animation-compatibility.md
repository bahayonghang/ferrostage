# Required compatibility matrix

This is a release acceptance contract, not a claim of implemented support. Sources: maintained [Beamer manual](https://raw.githubusercontent.com/josephwright/beamer/main/doc/beamerug-animations.tex), installed TeX Live `c:/texlive/2025/texmf-dist/tex/latex/beamer/beamerbaseoverlay.sty:753-774,808-830`, [PDF transition reference](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.7old.pdf), and [animate package description](https://www.ctan.org/pkg/animate?lang=en). Primary documentation only; do not copy another presenter implementation.

## Page transitions
| PDF S | Beamer command family | Parameter coverage / test distinction |
|---|---|---|
| R | transreplace | Immediate replacement, ignores duration |
| Split | transsplitverticalin/out, transsplithorizontalin/out | Dm H/V and M I/O |
| Blinds | transblindshorizontal/vertical | Dm H/V periodic reveal |
| Box | transboxin/out | M I/O centered reveal |
| Wipe | transwipe | Directed edge reveal |
| Dissolve | transdissolve | Spatial reveal, distinct from Fade |
| Glitter | transglitter | Directed spatial reveal including 315 degrees |
| Fly | transfly | Di, M, SS, B; changed-content motion/scale |
| Push | transpush | Old and new both move |
| Cover | transcover | Incoming moves, outgoing stays |
| Uncover | transuncover | Outgoing moves, incoming stays |
| Fade | transfade | Continuous blend |

Support Beamer-emitted cardinal direction values 0/90/180/270 for applicable directed effects, including the example Push 180. Distinguish PDF bottom-left coordinates from UI top-left coordinates. Default D is one second and absent S is R. Parse relevant optional entries with their specified defaults; invalid values produce an explicit fallback. For Fly, None is relevant to scale-only motion, SS controls starting/ending scale and B the opaque rectangular area. Prepare parameter fixtures from normative definitions; do not treat all variants as a translated page.

## Compiled page sequences
Manual overlays (pause, only, uncover, visible, onslide, alt, temporal, incremental lists) are compiled PDF steps, not a script runtime. Built-in animate uses zero dwell; animatevalue materializes differing compiled pictures. Transduration produces page dwell metadata. The player must preserve steps and handle /Dur separately from /Trans /D, including zero, omitted and removed values.

The thesis currently has no overlays, so new clean-room fixtures are necessary to validate this coverage. Notes can be interspersed in the physical sequence; all automatic and manual navigation must follow content mapping, not raw physical increments.

## Explicit boundary
The animate package's animategraphics/animateinline relies on PDF active-content machinery and is a separate compatibility project. Arbitrary JavaScript, 3D/Flash and new media codecs are not in 1.1.0. Existing supported movie/media behavior stays covered by its tests. Known unsupported active-content patterns receive bounded diagnostics, without claiming every possible script is detectable.

## Native evidence
Each advertised transition family needs native start/mid/end or a short recording, plus direction/in-out variants. Shared renderer-math tests can cover exhaustive parameter combinations; native fixtures must demonstrate semantics that would expose swapped layers or a generic-fade substitute. Timed sequences require first-visible evidence, final-frame stability, pause/interrupt and no Notes leakage. Do not promise frame-perfect Adobe parity or arbitrary animation frame rates.
