# Transition integration handoff

The transition worker owns PDF/document/core metadata, audience playback, texture-cache changes, and transition fixtures. It deliberately leaves production Shell wiring to this child. Coordinate any change to shared types with that worker.

- `ferrostage_core::PagePresentation { transition: Option<PageTransition>, dwell_seconds: Option<f64> }`.
- `LoadedDocument::page_presentation(PhysicalPage) -> Option<PagePresentation>` returns content-only metadata; Notes return None.
- `AudienceProjectionSnapshot.playback: Option<AudiencePlayback>` contains `identity: PlaybackIdentity { generation, navigation: u64, page }` and `transition: Option<PageTransition>`.
- Shell owns the monotonic navigation identity. Only sequential navigation authorizes transition metadata. Initial entry, Jump, reload, and replacement use `transition: None`.
- `audience::PlaybackEvent { identity, phase: PlaybackPhase::{FirstVisible, Settled}, at: Instant }` is an audience entity event. Subscription must fence the source audience window and the complete current playback identity.
- FirstVisible currently means the first successful effect paint after incoming upload (progress zero may still show the outgoing endpoint). Settled means the destination is painted. Immediate paths emit both after successful paint. Zero dwell must wait for Settled and a later event-loop opportunity.
- Standard timing correction after independent review: positive dwell starts at Settled plus `/Dur`, not at FirstVisible. PDF Reference 1.4 Figure 8.1 (printed page 487) places the display duration after the incoming transition. This corrects the initial technical assumption while preserving the requested standards-compatible behavior.
- Ensure playback identity is attached only to the actual rendered destination. Workbench assembly may retain an older visual while awaiting a new raster; a requested current page alone is insufficient evidence of a newly paintable destination.
- Confirm assembled `generation/current_page/slide_crop` equal the state and the Audience frame has the expected key and target. `assemble` retains the older snapshot metadata while pending. Keep that old visual's prior playback identity; allocate the destination navigation ID when navigation is accepted, hold it pending, and publish it only with matching destination pixels. Invalidate its old dwell timer while pending. Audience checks generation/page but Shell must also check crop.
- Blank, stage, staging, media, and ink settle/cancel audience effects. Shell must implement scheduler suspension/invalidation with the semantics in design.md, rather than relying on an animation's disappearance as a timer signal.

Fullscreen/layout worker owns Shell window transaction hooks until its handoff. Preserve those edits and existing window identities, focus restoration, timer behavior, and display recovery. Avoid global format while another worker is writing.
