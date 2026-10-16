# Task08 bounded native functional smoke — 2026-10-15

Source checks: `independent-review.md` and `review-final-build-identity.json`. Checked GPUI/PDFium executable SHA256 `2e38e384dec3065ad01e41640a8a399ae7112d2dddc686f799b83d12dde27b1f`, 41,183,232 bytes. Native launch PID79480 used the isolated `native/workbench-smoke.toml`, verified PDFium7881 DLL and `fixtures/notes-functional/pympress-notes.pdf`. Original PDF SHA256 `e939e35f931c184fd93b2e9b95e560db73ba6a6f6dfccbb0963262f0b359dfeb` matches the post-exit receipt.

Observed through actual native windows (paired screenshot/AX receipts under `native/run01-*`):

- Ready/current/next raster and private note01; Start opened the real Audience showing only the public PDF crop.
- Page2 and logical page3 overlay steps1/2 and2/2 changed physical3/5→4/5, with distinct private03A/03B notes visible only in Presenter. Audience screenshots contain the corresponding public STEP1/STEP2 content.
- Dragged the notes divider; actual notes pane enlarged.
- Numeric preview1 changed Presenter preview while Audience stayed on physical4/STEP2; Escape cancelled without committing.
- Paused timer showed41:36 at09:00:51 and remained41:36 at09:03:09.
- Real pen gesture produced matching normalized red strokes in Presenter and Audience.

The user's foreground desktop activity occluded the app during subsequent controls. On explicit user selection, remaining desktop smoke is deferred to one consolidated pass after09→10→11 implementation. Preview commit, grid/wheel, highlighter/eraser/clear/laser, black/white screen, document options, resume/reset and settings restart/readback are NOT_RUN in this bounded native pass; automated coverage is separate. Extended physical/native/size/DPI/theme/soak/performance matrices remain DEFERRED_BY_USER/NOT_RUN.

Process reconciliation: the planned controlled stop was not performed because the exact PID had already exited. The launcher subsequently returned exit_code0 at2026-10-15T13:07:03.1014142Z with empty stderr and matching executable/PDF hashes (`native/run01-exit.json`). No close gesture was observed, so this is an observed process exit, not a verified normal-close UI workflow. No remaining FerroStage process was returned by Get-Process. No unrelated process was stopped.

Task08 implementation and retained automatic gates are complete: GPUI95, real-PDFium UI29, default CI158, strict feature/test-support lint/typecheck, all-feature deny and final feature build PASS. Remaining desktop actions follow the user's revised schedule; no original composite matrix acceptance is marked complete. All edits remain uncommitted/unarchived.
