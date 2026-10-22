# Frontend Development Guidelines

> GPUI presenter and audience windows (`ferrostage-ui`).
> There is no React, DOM, or CSS. "Frontend" means the GPU view layer.

---

## Overview

The UI crate does not own the deck. It sends `PresentationCommand` and, for the audience window, reads `AudienceProjectionSnapshot` only.

The skeleton originally left GPUI unlinked. Task07 is adopting the explicitly approved maintained source based on Zed tag `v1.23.3`; ADR-008 and its final independent review define the exact local patches. `GPUI_KIT_VERSION` = 0.7.3 remains an alternate baseline constant, not a second UI stack. Do not depend on crates.io `gpui` 0.2.x or bypass actual product build/policy checks.

---

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Where views will live | Filled |
| [Component Guidelines](./component-guidelines.md) | GPUI views, presenter pane clicks, and the About changelog | Filled |
| [Hook Guidelines](./hook-guidelines.md) | No React hooks; GPUI context updates | Filled |
| [State Management](./state-management.md) | Core/snapshot boundary and shared display-assignment contract | Filled |
| [PDF Playback](./pdf-playback.md) | Transition metadata, paint acknowledgements and authored dwell lifecycle | Filled |
| [Quality Guidelines](./quality-guidelines.md) | Window rules | Filled |
| [Type Safety](./type-safety.md) | Newtypes and the notes boundary | Filled |

---

**Language**: All documentation should be written in **English**.

Native source/policy integration: [Maintained Dependencies](maintained-dependencies.md) defines the reviewed source set, exact exceptions, executable product checks and compatibility regression contracts.
