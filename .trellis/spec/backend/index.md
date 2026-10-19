# Backend Development Guidelines

> Domain, document, PDF, render, media, platform, and settings crates.
> FerroStage has no HTTP server and no SQL database. "Backend" means those crates.

---

## Overview

`ferrostage-core` is the state machine. IO crates sit beside it. The audience window is not a backend client; it renders `AudienceProjectionSnapshot`.

---

## Guidelines Index

| Guide | Description | Status |
|-------|-------------|--------|
| [Directory Structure](./directory-structure.md) | Crate and module layout | Filled |
| [Database Guidelines](./database-guidelines.md) | No ORM; document model and TOML | Filled |
| [Error Handling](./error-handling.md) | `thiserror` in libraries, `ExitCode` in the binary | Filled |
| [Quality Guidelines](./quality-guidelines.md) | `just ci`, forbidden patterns | Filled |
| [Logging Guidelines](./logging-guidelines.md) | `tracing`; do not log PDF bytes | Filled |
| [PDF Chapter Directory](./pdf-outline.md) | Bounded two-level bookmarks and presenter content mapping | Filled |
| [Windows Distribution](./windows-distribution.md) | Shared desktop payload, Setup, registration and native uninstall | Filled |

---

**Language**: All documentation should be written in **English**.
