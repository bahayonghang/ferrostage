# Standard Beamer/PDF page transitions

Implementation approved on 2026-10-19 and complete. Integrated 1.1.0 gates and native samples passed; see parent research/validation.md for qualification limits.

Parent R4/R6. Broadened from three effects to the 12 standard PDF styles so future ordinary Beamer commands use an already-tested renderer.

## Acceptance criteria
- T1: Parse owned destination `/Trans` metadata, including indirect dictionaries, `/S`, `/D`, `/Dm`, `/M`, `/Di`, `/SS` and `/B` where relevant. Correct defaults and malformed-value diagnostics are tested.
- T2: Implement Replace(R), Split, Blinds, Box, Wipe, Dissolve, Glitter, Fly, Push, Cover, Uncover and Fade. Each has visibly correct start/mid/end behavior; similar names are not aliases for a generic fade. Cover and Uncover preserve their different stationary layer; Fly handles authored scale/opacity mode rather than pretending to be Push.
- T3: Fixtures cover Split H/V and I/O, Blinds H/V, Box I/O, cardinal directional effects, Glitter's diagonal direction, Fly in/out, scale-only None and B modes, durations/defaults and unsupported values. The sample's 85/7/3 content effect inventory remains a regression.
- T4: Sequential content/overlay navigation uses destination effects. Presenter panes remain stable and immediate. Initial Start/Jump/reload/window replacement settle immediately. Notes pages never trigger effects or enter retained frames.
- T5: Rapid input is latest-target-wins without queueing. Black/white/curtain/Stop interrupt immediately. Repeated snapshots, same-page media updates and no-op boundaries never restart an effect. All callbacks are generation/navigation fenced.
- T6: Old/new/staging/media resources share the existing budget; errors fall back safely with bounded diagnostics. No per-tick PDF rerasterization, new engine or maintained-dependency patch is assumed.
- T7: Native fixture recordings exercise every advertised style family, parameter variants and cancellation. Parsing-only success cannot count as complete support. No new effects picker or preference panel is added; the PDF is authoritative.

## Scope
Composition plus typed metadata and integration hooks for the timing child. Compiled overlays and /Dur are owned by that child. JavaScript animate-package runtime, video/3D/Flash and arbitrary future plugin compatibility remain outside this release and are documented honestly.
