# Beamer notes

The document model below is implemented and tested with pure Rust and native PDFium fixtures. Presenter UI and playback integration belong to later tasks. FerroStage reads a finished PDF. It does not call a document compiler, and it does not promise to reconstruct an arbitrary export. Media fields are checked against the PDF dictionary subset this project supports. Unknown structure produces a presenter warning.

When a Beamer PDF uses a second-screen notes layout, one physical page holds both the slide and the notes. The audience crop is the slide half. The presenter reads notes from `ferrostage-document`, not from `AudienceProjectionSnapshot`.

## Split geometry

Automatic detection is only a candidate, and only when every page in the document has the same size. The test is the page aspect `W/H`:

| Layout | `W/H` band | Nominal ratio |
| --- | --- | --- |
| Left / right | [3.1, 3.6] or [2.4, 2.7] | Wide frame (width doubled, height unchanged) |
| Top / bottom | [0.85, 0.95] or [0.63, 0.70] | 8:9 or 2:3 (height doubled, not the reciprocal of the left/right band) |

A ratio does not prove that notes exist, and it does not prove which side they are on. A same-size document receives an automatic `Right` candidate only when `W/H` is in a left/right band, and an automatic `Bottom` candidate only when `W/H` is in a top/bottom band. Outside those bands there is no automatic crop. A normal 16:9 or 4:3 page, and a tall page outside the top/bottom bands, are counterexamples. The UI shows that the source is a ratio guess, plus any warning. Mixed page sizes do not receive a document-wide crop.

## Manual override

A document-level override is the final authority: `None`, `Left`, `Right`, `Top`, or `Bottom`. The slide crop is the opposite half.

| Override | Slide | Notes |
| --- | --- | --- |
| `None` | Full page | No split crop |
| `Left` | Right half | Left half |
| `Right` | Left half | Right half |
| `Top` | Bottom half | Top half |
| `Bottom` | Top half | Bottom half |

`None` turns off the split crop only. It does not put independent notes pages into the audience navigation.

## Notes pairing

`notes:<suffix>` is this project's convention for an independent notes page. The suffix is the raw page label of the immediately preceding content step, compared as an exact string. It is not an index into the content list. `notes:01` does not match a content label `1`.

Several notes pages for one content step are a `Vec` of physical page numbers stored only in `ferrostage-document`. An ambiguous or orphan notes page is not projected. The presenter gets a warning. The core `SlideMap` contains navigable content steps only.

## Page labels and navigation

`PageLabel` is `Option<String>`. A missing or empty label stays independent on that page. Consecutive identical non-empty content labels are a heuristic overlay group. The same label later in the file does not merge into that group. A `physical_pages` override can turn grouping off.

Overlay steps (`\pause` and similar) are steps inside one logical slide. `NextLogicalSlide` skips the rest of the current overlay group. `NextStep` moves to the next content physical step; `NextLogicalSlide` moves to the first step of the next logical group. Both clamp at their boundaries. `JumpPreview` rejects independent notes pages.

Jump preview belongs to the presenter. While it is active, next and previous move only the preview. Commit (Enter) updates the audience page. Cancel (Esc) drops the preview. The audience page does not change during the preview.

Audience navigation never lands on a notes page. After an external PDF replacement, the preferred restore key is a unique previous logical label plus the overlay ordinal. If that key is ambiguous or missing, clamp the old physical page into the new physical range, then take the greatest content page at or before that number, or the first content page if none exists. Do not use the content `Vec` index as a page number. Strokes survive only on pages that still map; dropped pages are discarded and the presenter is warned. The session stage and the timer are not reset. A generation change invalidates media playback; until a player exists, that event has no effect.


## Model boundary and current producer evidence

`SlideMap::new(page_count, groups)` validates nonempty groups of strictly increasing, in-range physical content pages. `DocumentDescriptor::mapped(generation, map, crop)` validates the crop. `identity` is an explicit metadata-only shell path; real imports use `LoadedDocument::descriptor`. Core keeps the map privately and never publishes it to the audience.

`LoadMappedDocument` resets the lifecycle to Ready; `ReconfigureDocument` requires the same generation and physical page count, preserves stage/timer, clears preview and clamps to content; `CommitDocument` requires a strictly newer generation. The temporary `StageShadowDocument` adapter retains the current map and crop. File identity and the full candidate transaction remain app/task10 responsibilities.

The compatibility `set_slide_crop` returns `Result<(), CoreError>` and rejects invalid geometry without changing state. Document viewing options use `ReconfigureDocument` so the map and crop update together.

`LoadedDocument::load` resets document-local overrides. `build` takes explicit overrides; `reload` reuses them for a caller-identified replacement of the same file. `notes_for` retains its existing Markdown lookup; `notes_pages_for` returns ordered PDF notes pages for a content step. `label_candidates` returns every matching logical group, never an arbitrary first duplicate.

The checked-in Beamer 3.72 fixtures demonstrate both useful grouping (`overlays.pdf`: labels `1,1,2`, two frames) and a limitation (`notes-right.pdf`: two source frames, labels `1,1`). The latter deliberately remains unmodified; `physical_pages` gives two groups. Geometry detection still correctly selects Right. Ordinary wide and portrait fixtures show that a ratio candidate can be a false positive. See `fixtures/beamer/README.md`.

## Read-only media metadata

`ferrostage_pdf::extract_structure` returns owned structures and diagnostics. `MediaAnnotation.rect` is normalized against the visible CropBox after clockwise page rotation. Document clips that rectangle to the selected slide crop and rescales it to `MediaSlot.slide_rect`; no intersection yields None. `MediaSlot.annotation` is the ordinal in that page's extracted media list; `StructureDiagnostic.annotation` is the original PDF Annots ordinal. Neither is a global cross-generation identity. Consumers must also bind generation and physical page.

Movie supports source `Movie/F`, `A/Start`, `A/Duration`, `A/Mode`, and matching page-open/visible Movie actions. Explicit `[ticks, scale]` timing becomes seconds only for a positive integer scale; an unscaled integer or 8-byte signed integer remains `MediaTime::Unscaled` with a diagnostic. Movie action parameters override annotation activation parameters. A title target must uniquely identify a Movie on the same page. Beamer's Widget `AA/PO` title target is supported.

Screen supports a Rendition action with `OP=0` and an `AN` reference to the same annotation, a media rendition `R/S=MR`, `C/S=MCD` or one `MCS` time-begin wrapper, and explicit play parameters. Per-key MH values take precedence over BE values. Click activation does not establish page autostart. RichMedia supports one Video/Sound instance in the selected configuration and a file specification also present in the Assets name tree. Its activation condition is recorded; Flash scripts, FlashVars and animation parameters are not interpreted as video timing or loop settings.

External strings and decoded embedded bytes remain on the document/control side. No path is opened, action executed, or PDF rewritten. Embedded decoding is capped at 64 MiB per asset. Missing, malformed, unsupported and unresolved timing fields produce diagnostics, not fabricated defaults. This is a metadata subset, not a playback or PDF-conformance implementation. Exact paths, primary tables and limitations are recorded in task06 `research/media-field-contract.md`.
