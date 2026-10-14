# ADR-003: Audience snapshot has no notes

## Status

Accepted and implemented in `ferrostage-core`.

## Decision

`AudienceProjectionSnapshot` contains generation, page, crop, blank mode, laser position, and strokes. Speaker notes stay on `SlideNotes` in `ferrostage-document`. `LoadedDocument::project` takes notes and does not copy them.

Swapping screens flips `PresentationState::screens_swapped` and leaves the snapshot page and crop unchanged.

## Consequences

UI code that needs notes reads `ferrostage-document` from the presenter window only.
