# Architecture

FerroStage splits the presenter into a pure state machine and a set of IO crates.

```text
ferrostage-app
  ferrostage-ui          GPUI presenter + audience windows
  ferrostage-settings    TOML
  ferrostage-core        commands and audience snapshot
    ferrostage-document  notes and media slots (presenter only)
    ferrostage-pdf       lopdf now; PDFium raster later
    ferrostage-render    one worker, generation checks, cache budgets
    ferrostage-media     shared BGRA frames
    ferrostage-platform  debounce, %%EOF, atomic rename
```

`PresentationCommand` is the only way UI code changes the deck. `audience_snapshot` is the only way the audience window reads it. A hot reload calls `StageShadowDocument` with a new `DocumentGeneration` and the shadow bytes. The core checks the generation and the non-empty buffer; it does not keep the bytes.

Raster format is BGRA8 end to end, including pen color (`NormalizedStroke::color_bgra`) and `SharedFrame`. Do not convert to RGBA on the audience path.

Render priority, once the worker exists:

| Priority | What |
| --- | --- |
| P0 | Current audience page |
| P1 | Immediate neighbor |
| P2 | Presenter preview |
| P3 | Prefetch |
| P4 | Thumbnail |

Requests whose generation is not the current one are discarded (`ferrostage_render::is_current`).
