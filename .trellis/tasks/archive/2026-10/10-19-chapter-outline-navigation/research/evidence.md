# Evidence and Planning Review

## Read-only Sample Probe

Source: `D:/Documents/LYH/200-Learning/00博士毕业/毕业论文/thesis/slides/deck-notes/defense.pdf`.

SHA256: `3f7246010bfbd5e6352d4abcdeb119c73093f693d588042aa8f49398d5d24f45`.
Size: 71,901,127 bytes. No source PDF bytes are copied into this task.

The probe uses bundled Python/pypdf only for research. Product dependencies are unchanged. Command from repository root:

```powershell
& 'C:/Users/lyh/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/python.exe' '.trellis/tasks/10-19-chapter-outline-navigation/research/probe_outline.py' 'D:/Documents/LYH/200-Learning/00博士毕业/毕业论文/thesis/slides/deck-notes/defense.pdf' '.trellis/tasks/10-19-chapter-outline-navigation/research/pdf-outline.json'
```

Results: 190 physical pages, 95 mapped content pages, 95 notes; labels begin 1, notes:1, 2, notes:2. Each recognized note label is checked against its actual preceding content label. Content numbers are assigned by enumerating retained content pages, never arithmetic division. Outline depths: 10 + 29 + 44 = 83; retained rows: 10 + 29 = 39; all 39 targets belong to content. There are 381 named destinations, but these are not 381 chapters. Example first bookmark uses /Dest [page-object-reference /Fit].

| Authored title | Content number | PDF number | PhysicalPage |
| --- | --- | --- | --- |
| 第2章 水泥烧成系统工艺分析与运行优化框架 | 12 | 23 | 22 |
| 第3章 基于多速率数据融合的熟料质量指标建模 | 22 | 43 | 42 |
| 3.1 引言 | 23 | 45 | 44 |
| 致谢 | 95 | 189 | 188 |

Keep cover, contents and unnumbered second-level conclusion. Actual chapter 2 starts at 2.2; source numbering is authoritative. Depth-3 titles such as 1.2.1 and 2.2.4 must be hidden even when their destinations equal a visible parent target. Full retained data is in pdf-outline.json.

## Current Repository Anchors

| Evidence | Consequence |
| --- | --- |
| `crates/ferrostage-pdf/src/structure.rs:12`, `:75`, `:175` | Structure owns metadata and already uses lopdf text decoding, but has no outline field. Extend this boundary. |
| `crates/ferrostage-document/src/builder.rs:29`, `:135`, `:169` | LoadedDocument owns notes/content mapping. Map directory targets after SlideMap creation. |
| `crates/ferrostage-core/src/slide_map.rs:48`, `:54` | Content steps and membership are explicit. Do not count logical slides or infer pairing. |
| `crates/ferrostage-ui/src/workbench.rs:81`, `:95`, `:302` | Session holds raw/mapped data; reconfiguration rebuilds it; grid gates thumbnail scheduling. |
| `crates/ferrostage-ui/src/shell/page_browser.rs:21`, `:48`, `:64`, `:132` | Reuse browser id/generation guard, physical target conversion and safe submit. |
| `crates/ferrostage-ui/src/presenter/page_browser.rs:7`, `:54`, `:107` | Snapshot data, contextual host and 880px panel already exist. |
| `crates/ferrostage-ui/src/page_browser.rs:58` | Browser-local numeric state must stay separate from new view/focus state. |
| `crates/ferrostage-ui/src/actions.rs:50` | Add view/entry activation beside existing browser intents. |
| `crates/ferrostage-ui/src/shell/reload.rs:779` | Successful replacement already closes contextual UI before publishing new state. |
| `crates/ferrostage-ui/src/theme.rs:5`, `:167`, `:179` | Existing metrics and palettes settle appearance; do not approximate colors from the screenshot. |
| root `DESIGN.md`, page-browser contract | Product source of visual truth; amend its relevant section during implementation only. |

Sources were read from the dirty current working tree, not an older committed snapshot. Refresh anchors before implementation because defense-experience work overlaps these files.

## Scoped State Contracts

The full state-management spec currently exceeds the context injection file-size limit. Read its relevant sections directly before implementation: `.trellis/spec/frontend/state-management.md:17` keeps the audience on the assembled snapshot only; `:29` routes page selection through JumpTo; `:33` keeps editing/hover inert and committed navigation on existing media/annotation/raster invalidation; `:35` restricts thumbnail scheduling to an explicitly visible grid. These complement the smaller injected component/type-safety/playback specs and avoid silently truncating unrelated later sections.

## lopdf Reuse Assessment

Inspected installed source `C:/Users/lyh/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/lopdf-0.45.0/src/outlines.rs`: get_outline accepts GoToR at line 22; recursive traversal descends into every /First at lines 87-90 and follows /Next without a visited set at lines 97-100; build_outline_result indexes array elements at lines 126-128. Therefore blindly using get_outlines would not establish this task's safe two-level contract. Reuse existing object/text APIs with bounded local traversal. This is source inspection, not a product bug reproduction or dependency audit.

## Layout Study

- `directory-study.html`: standalone static review surface, actual 39 titles and verified page numbers. Buttons represent a layout, not implemented app behavior.
- `directory-current.png`: at content 23, current section revealed and distinguished from neighboring sections.
- `directory-long.png`: chapter 6 with the long authored title and aligned numbers.
- `directory-empty.png`: hypothetical no-bookmark state, explicitly labeled as distinct from the sample.
- `layout-metrics.json`: all three renders use a 1100x960 review canvas, with an 880x812 panel; no document horizontal overflow.

Rendered with bundled Playwright and locally installed headless Edge, viewed all three images. Checked 3 layout states / changed 0 native surfaces / left native verification for implementation. The dark palette/font/spacing match the product tokens; no new cards, gradient or decorative motion. Current status is textual and the numbers retain a dedicated aligned column. Full authored titles replace placeholders from the outset.

Only a normal-window design study is claimed. Light theme, alternate locale, native focus, GPUI wrapping and action behavior remain NOT_RUN. No product tests/builds were run because this request produces planning artifacts only.

## Requirement Convergence Review

- Goal, scope and exclusions are explicit in PRD.
- AC1-AC7 map to R1-R8 and to parser, mapping, guarded lifecycle, frontend and validation mechanisms.
- All technical questions required for the proposed MVP have evidence or a bounded design contract.
- PRD was reread against the final requirement/acceptance structure, with no seed TBDs or temporary brainstorm sections retained.
- The recommended tabbed layout and behavior are proposed for user review, not silently treated as approved.
- One standalone task is appropriate: parser/mapping/UI changes are required parts of one independently verifiable directory outcome.
- Task remains planning, no task activation, product edits, commits, installs or implementation delegation.
