# Integrated design

## Visual and content contract
Quiet warm-neutral presentation console using existing GPUI/theme.rs tokens and system/CJK fonts. PDF colors/fonts remain authored. Compact status/time row, current slide left, Next over original Notes right, one compact session/navigation row. No Text/Original toggle, A-/A+ or Notes editor.

```text
[ clock / title                 elapsed / pause / reset / more ]
[                               | Next, aspect-fit            ]
[ Current, aspect-fit           |-----------------------------]
[                               | Original PDF Notes          ]
[                               | Larger source typography    ]
[                               | Existing vertical scrolling ]
[ End   Previous  position  Next   blackout   tools   page grid ]
```

Fresh 60/40 columns and 35/65 Next/Notes are initial values; persisted custom fractions remain valid. Slide letterboxing is preferable to cropping or distortion. Notes original images use width-based natural height in the existing scroll container, including multiple mapped note pages, rather than equal flex shrinking. Fullscreen and source typography together recover useful reading area.

## Ownership and repositories
FerroStage owns Shell lifecycle, content-only projection, transition/timing logic and version. The thesis root is `D:/Documents/LYH/200-Learning/00博士毕业/毕业论文/thesis`. Its `src/thesis_analysis/defense_notes_sync.py::NOTES_BLOCK` owns the generated Notes template; slides/deck-notes/defense.tex is an output of sync. Read thesis AGENTS.md, slides/AGENTS.md and the defense-slides spec before writing there. Preserve existing uncommitted work; these include the sync module and source notes. Do not copy their sources into FerroStage or commit across repositories as one change.

The original `\note` payloads remain canonical. Source changes control font, leading, paragraph rhythm and margins, not an app text engine. The content slides remain sourced from slides/deck; if a shared style must change, edit the canonical style and sync, with unchanged content-slide rendering as a regression gate. The initial template candidate is 16pt text / 20pt baseline and restrained paragraph spacing, subject to complete-page fit and native visible-size checks. Do not silently shrink dense notes or remove sentences to pass.

## Animation architecture
PDF structure parsing yields owned transition and optional dwell descriptors. Document maps them to physical content steps. Shell/assembler schedules a content-only timeline with one navigation identity and monotonic clock; Audience consumes snapshots only. One render worker, BGRA8, generation fences and budgeted retained old/new images remain mandatory. Presenter remains stable for reading.

The transition child covers spatial composition. The timing child owns /Dur and compiled overlays and depends on the transition completion/first-visible-paint contract. No generic PDF scripts execute. Unknown active-content requirements receive a quiet document-load diagnostic with affected pages; existing known media is not mislabeled unsupported. No new animation configuration panel.

## Lifecycle and ordering
Implement fullscreen/layout first. Optimize original Notes source and rendering next. Transition parsing/composition precedes timed overlays because dwell scheduling needs the same visible-page identity. Bump to 1.1.0 last, rebuild and inspect release metadata. UI-only changes do not write the PDF. Thesis compilation intentionally regenerates defense.pdf; compare its public content pages and note labels rather than demanding an unchanged whole-file hash. Record before/after hashes for provenance.

Stop/blackout/reload/topology change retire pending timelines before visible paint. Auto-advance is separate from the talk timer and has a minimal contextual pause/resume control only for authored timing. Manual navigation cancels the current scheduled action and re-arms from the actually displayed target page. Pointer/drawing and modal interactions suspend automatic playback. Details live in the timing child.

## Rollback and risks
Independent change sets per child and per repository. Retain original Notes template/baseline hashes before source editing. Revert only task-owned hunks, never the user's existing work. Invalid effects use Replace; unsupported scripts remain outside compatibility claims. All standard effect paths, especially Fly and Glitter/Dissolve, require a maintained-GPUI capability proof. A failed spike changes readiness, not the supported-effect checklist. Native standing acceptance and cross-display playback remain separate from tests.
