# ADR-003: Audience snapshot has no notes

## Status

Accepted and implemented in `ferrostage-core` for the notes boundary, lifecycle stage, and pixel types. Visual assembly remains tasks 08/11; the app role coordinator remains task 07 ([ADR-007](ADR-007-monitor-roles.md)).

## Decision

`AudienceProjectionSnapshot` contains generation, page, crop, blank mode, laser position, and strokes. It also includes `stage: AudienceStage` and `visual: Option<AudienceVisual>`. Speaker notes stay on `SlideNotes` in `ferrostage-document`. `LoadedDocument::project` takes notes and does not copy them.

The core produces projection metadata. The UI assembler adds only the matching page BGRA and controlled media pixels to `visual`; `Curtain` carries no visual. The audience view receives only this snapshot, never `LoadedDocument`, notes, media sources, a cache handle, or an arbitrary-page request API. It keeps a complete published projection until a matching new frame is ready, while `Curtain` and black/white blank immediately cover every visual layer. This visual assembly is planned work in tasks 08/11, not implemented by the current notes-boundary test.

Swapping screens leaves the snapshot page and crop unchanged. Core rejects direct `SwapScreens` with `ScreenSwapRequiresCoordinator` and has no swap flag. The app intercepts the intent. `display.monitors` (`MonitorId` + `MonitorRole`) is.

## Consequences

UI code that needs notes reads `ferrostage-document` from the presenter window only.
