# Home baseline and reference decisions

Research date: 2026-10-18. Baseline branch `dev`, HEAD `595910c`, clean before task creation. Evidence: current source and user screenshot, not memory-derived UI claims.

## Screenshot diagnosis

Original preserved as `research/home-before.png`, from attachment `codex-clipboard-ff37d619-e0fd-4873-a0e6-becd072592cc.png`.

Home reads as a stretched form: brand repeats across native title, app header and body; status/time competes with commands; recent documents look like centered buttons; disabled Start consumes a bottom strip before a PDF exists. This is hierarchy/composition failure, not just one excessive button height.

Approximate bitmap observations in the supplied 1377x989 image: Open/recent faces span about x35 to x1342, roughly 95% of image width; faces about 55px high; recent row gaps about 28px. These are visual estimates, not logical-client measurements or proof of DPI. Comparable native measurements remain an implementation gate.

## Source evidence

| Anchor | Current behavior and consequence |
| --- | --- |
| `DESIGN.md:74` | Existing brand provenance/customization. Reuse instead of applying a new catalog preset. |
| `DESIGN.md:107` | Existing minimum target contract 44x44. Fix width/grouping first. |
| `crates/ferrostage-ui/src/theme.rs:4` | CONTROL=44, focus width/gap=2 each, spacing=16. Shared button adds 8px focus reservation outside its face. |
| `crates/ferrostage-ui/src/theme.rs:10` | Current sans/display/mono roles: Segoe UI, Georgia, Consolas; explicit CJK fallback. Distinct brand needs no font download. |
| `crates/ferrostage-ui/src/presenter.rs:66` | Recents captured from settings with `.take(3)`. Preserve ordering/count. |
| `crates/ferrostage-ui/src/presenter.rs:207` | Shared button owns IDs/role/focus/guards/dispatch; edits can affect settings/workbench/displays. |
| `crates/ferrostage-ui/src/presenter.rs:273` | Face has CONTROL min height, 16px padding and centered content. Parent composition creates the home stretch. |
| `crates/ferrostage-ui/src/presenter.rs:309` | Shared command bar assembles identity/stage/timer/Open/Settings/Displays. Idle needs deliberate composition. |
| `crates/ferrostage-ui/src/presenter.rs:533` | Native title combines fallback filename/stage/product and repeats identity on Idle. |
| `crates/ferrostage-ui/src/presenter.rs:581` | Root already uses sans at 16px. Bitmap alone does not prove faulty font resolution. |
| `crates/ferrostage-ui/src/presenter.rs:739` | Idle renders Georgia title, instruction and primary Open. |
| `crates/ferrostage-ui/src/presenter.rs:775` | Common error/warning/save feedback must survive composition changes. |
| `crates/ferrostage-ui/src/presenter.rs:812` | Recents reuse centered generic command buttons in stretching wrappers. |
| `crates/ferrostage-ui/src/presenter.rs:840` | Common command bar/body and unconditional Start/Stop footer. Omit footer only on Idle. |
| `crates/ferrostage-ui/src/presenter/workbench_view.rs:340` | Timer panel owns monospace timer/warnings/clock and Presenting controls; preserve outside Idle. |
| `crates/ferrostage-ui/src/shell.rs:933` | Native minimum 320x240, so test minimum height and combined minimum too. |
| `crates/ferrostage-ui/src/shell.rs:4673` | Existing tests measure shared header/notes geometry; retain them. |
| `crates/ferrostage-ui/src/shell.rs:4736` | Idle/home tests cover welcome entry and recents; nearby tests cover audience close on Home. |
| `crates/ferrostage-ui/src/locale.rs:4` | Real catalogs are `assets/locales/en-US/app.toml`, `assets/locales/zh-CN/app.toml`; existing key/field parity test. |

Anchors describe this baseline and need refresh if source moves. No product build/runtime/contrast verification ran in planning.

## Official web references and decisions

These sources inform patterns, not an exact FerroStage render. The 760px column, brand identity and 48-56px rows are proposed product values requiring native validation.

1. [Microsoft, Buttons](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/buttons): concise action labels, localized sizing, and content-aligned single page action. Decision: one natural-width, left-aligned Open; the documented 120px starting minimum does not justify a full-width command.
2. [Microsoft, Collections and lists](https://learn.microsoft.com/en-us/windows/apps/develop/ui/controls/lists): vertically stacked lists suit text-oriented collections. Decision: file rows with consistent interaction, no thumbnail grid/oversized command outlines. Apply the pattern in GPUI, not WinUI APIs.
3. [Microsoft, Typography](https://learn.microsoft.com/en-us/windows/apps/design/signature-experiences/typography): hierarchy supports scanning, default alignment is left, type ramp includes body 14/20 and caption 12/16. Decision: separate brand from 14px filenames/actions and quieter paths using current CJK-capable system fonts.
4. [Apple, Open or close a Keynote presentation](https://support.apple.com/en-sg/guide/keynote/tan72232b56/mac): distinguishes opening an existing file and reopening a recent presentation. Decision: home prioritizes file entry before presentation. This source does not establish FerroStage geometry or require removing its timer.
5. [VS Code, Tips and tricks](https://code.visualstudio.com/docs/editing/getting-started/tips-and-tricks): describes Welcome and Open Recent entry mechanisms. Decision: keep home recoverable and recents actionable; do not import its sidebar/cards/command palette.

Omitting Idle timers/status/disabled Start is an inference from this product's preparation sequence and screenshot, not an external-source requirement.

## Skills and catalog comparison

Used: `ui` screenshot-iteration/full direction guidance (multiple roles affected), `design-md` existing-source/customization/brand boundaries, `trellis-start`, `trellis-brainstorm`. User explicitly authorized task creation.

Catalog search `minimal` and entries `style:minimal`, `style:refined`, `style:geometric` were read. Existing DESIGN wins over generic defaults. No apply/replace/font download ran.

| Direction | Properties | Decision |
| --- | --- | --- |
| A. Warm editorial utility, current customized `brand:claude` | Dedicated Georgia wordmark, sans rows, clay action, flat warm surfaces, bounded single column | Recommended; lowest style/platform drift while addressing the complaint. |
| B. Neutral geometric utility, `style:geometric` comparison | Uniform sans identity, stricter grid, neutral surfaces, minimal decoration | Viable but less distinctive brand and larger shift. Not applied. |
| C. Expressive editorial, `style:refined` comparison | Strong display serif, asymmetric brand area, more generous spacing | More character, risks oversized headings/whitespace weakening file utility. Not applied. |

Do not import generic catalog blue/purple colors or font lists. Current warm palette/system fonts/depth/reduced motion remain authoritative. No borrowed logo/wordmark/assets.

## Planning checks and remaining evidence

- Proposed PRD/design/execution plan plus spec/research manifests and preserved source image exist.
- Product/root DESIGN/dependencies/settings/PDF/audience/workbench are unchanged.
- Native build, after screenshots, pointer/keyboard checks, contrast reruns and size/scale matrix are NOT_RUN; implementation acceptance gates remain explicit.
- Referenced `C:\Users\lyh\.codex\RTK.md` was absent when checked; no instructions were inferred from the missing file.

## Planning verification receipt

- PASS: task context validator, four implement entries and three check entries with real existing spec/research paths.
- PASS: required planning documents, seven requirements and ten acceptance criteria, source anchor bounds, and no unresolved template markers.
- PASS: original PNG copied byte-for-byte, 1377x989; SHA256 `9f534f6a3607d18d830e9655304e727bd039fa7dba26e01ccdc598f1d4b985a7`.
- PASS: task remains `planning`; only the new task directory is untracked, with no tracked product/root DESIGN changes.
- NOT_RUN by scope: compilation, product tests, native launch/render, final contrast and visual/input matrix. No implementation has occurred.
