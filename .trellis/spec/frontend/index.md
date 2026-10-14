# Frontend Development Guidelines

> GPUI presenter and audience windows (`ferrostage-ui`).
> There is no React, DOM, or CSS. "Frontend" means the GPU view layer.

---

## Overview

The UI crate does not own the deck. It sends `PresentationCommand` and, for the audience window, reads `AudienceProjectionSnapshot` only.

GPUI is not linked in the skeleton. The pin is Zed tag `v1.23.3` (`GPUI_KIT_VERSION` = 0.7.3). Do not depend on crates.io `gpui` 0.2.x.

---

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Where views will live | Filled |
| [Component Guidelines](./component-guidelines.md) | GPUI views, not React components | Filled |
| [Hook Guidelines](./hook-guidelines.md) | No React hooks; GPUI context updates | Filled |
| [State Management](./state-management.md) | Core state vs snapshot | Filled |
| [Quality Guidelines](./quality-guidelines.md) | Window rules | Filled |
| [Type Safety](./type-safety.md) | Newtypes and the notes boundary | Filled |

---

**Language**: All documentation should be written in **English**.
