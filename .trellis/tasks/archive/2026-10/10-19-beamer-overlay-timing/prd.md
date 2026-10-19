# Beamer overlays and authored timed playback

Implementation approved on 2026-10-19 and complete. Integrated 1.1.0 gates and native samples passed; see parent research/validation.md for qualification limits.

Parent R5/R6. Depends on transition child's content identity and first-visible/settled events. Standard overlay support already exists in part; verify and repair actual gaps rather than reimplementing it.

## Goal
Allow future common Beamer incremental/animated content to play without further app changes when it compiles to ordinary PDF pages plus standard timing metadata.

## Acceptance
- O1: Clean-room Beamer fixtures using pause, only, uncover, visible, onslide, alt, temporal, incremental lists and animatevalue preserve every compiled step, correct forward/backward order and slide/step numbering. Missing or ambiguous labels retain the existing explicit physical-page fallback instead of inventing grouping.
- O2: Independent notes between overlay pages are matched by labels and content identity. No Notes enter Next preview, audience projection, automatic sequence or transition source. Multiple notes and overlay-specific notes are tested; existing physical/content page-browser behavior stays intact.
- O3: Optional page /Dur is parsed independently from transition /D. Missing means manual advance; positive means authored timed advance; zero supports Beamer animate's as-fast-as-renderable sequence. Removing /Dur on the next page stops automatic advance. Invalid values produce a diagnostic and manual fallback, not a fast runaway loop.
- O4: Scheduling runs only for an actually presented eligible content page. It never skips an unpainted step to catch up. Positive duration starts after successful transition settlement; zero advances only on a later UI turn after that step has painted and its effect settled. At most one pending advance is armed. This follows PDF Reference 1.4 Figure 8.1 (printed page 487), correcting the initial first-visible timing assumption.
- O5: A minimal Pause/Resume automatic-playback action appears only for documents with authored timing. It is separate from the talk timer. Pausing freezes remaining dwell; resume continues without catch-up. Manual navigation cancels the old callback and evaluates the destination afresh, preserving explicit pause state.
- O6: Blackout/curtain, Stop, reload, display replacement, drawing interaction, focus/modal loss and render failure suspend or cancel scheduled navigation according to the design. No stale callback can advance a new page/document or continue after Stop. End of deck stops cleanly.
- O7: Real native fixtures prove positive duration, zero-duration sequence, stable final step, pause/resume, rapid manual interruption and Notes isolation. Timing tests use a fake clock; no fixed sleeps or inferred success from page counts alone.

## Compatibility boundary
Beamer's built-in animate and animatevalue compile into page sequences/timing and are in scope. The separate animate package's animategraphics/animateinline and arbitrary JavaScript/OCG animation engines are not. Detect known active-content requirements through bounded existing structure diagnostics, report affected pages without executing scripts or claiming complete detection. Existing media support is preserved, not rebuilt.
