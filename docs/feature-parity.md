# Feature parity

The behavior target is a dual-screen PDF / Beamer presenter. Nothing below is a port of another project's source. The skeleton implements the state types, not the windows.

| Feature | Priority | Skeleton |
| --- | --- | --- |
| Open a PDF from the CLI | P0 | Not started |
| Presenter and audience windows | P0 | Roles exist; GPUI is not linked |
| Next / previous page | P0 | Implemented in `ferrostage-core` |
| Speaker notes | P0 | Stored in `ferrostage-document`, excluded from the snapshot |
| Beamer overlay steps | P0 | Not started; logical slide equals physical page |
| Notes-page crop (left half) | P0 | `NormalizedRect::LEFT_HALF` exists; parser does not |
| Blank black / white | P0 | Implemented |
| Laser pointer | P0 | Position is stored; no window draws it |
| Pen strokes | P0 | Points are stored; no window draws them |
| Screen swap | P0 | Flag flips; snapshot page stays |
| Hot reload | P0 | Policy constants and generation check; watcher loop not started |
| Thumbnails and cache budgets | P1 | Budgets only |
| Video / GIF | P1 | `SharedFrame` length check only |
| Installer | P2 | Not started |
