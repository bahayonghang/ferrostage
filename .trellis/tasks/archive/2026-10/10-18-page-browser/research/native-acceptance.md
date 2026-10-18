# Page-browser native acceptance, 2026-10-18

Coordinator-owned Computer Use via `@oai/sky`, real `fixtures/pdf/perf-100.pdf`, isolated task-local settings, ordinary 1100x760 client. No size/DPI matrix. Source/exe/runtime identities and captures are in parent `research/native/`.

## Before final polish

On exe ADC9A418..., actual keyboard input replaced selected1 with6 then69. Main preview remained physical1/logical1 while helper showed physical69/logical35. Enter closed panel and updated slide/notes/next to physical69, logical35, restoring progress focus. Captures: `page-browser-edit-69-no-jump.jpg`, `page-browser-jump-69.jpg`.

That run exposed duplicate checks and clipped4x4 rows; `page-browser-before-native-polish.jpg` is preserved as failed visual checkpoint, not final acceptance.

## Final accepted build

EXE SHA256 `B7D001F82ABF24003D166A07BAADD7CD664F9B6D30A4214663FAF5A7CC4C408C`, native PID166376.

- `page-browser-final-4x4.jpg`: all16 real cropped thumbnails visible, one check per selected dimension, input/controls/footer fit above toolbar.
- `page-browser-final-2x3.jpg`: independent column2 then row3 choices immediately show6 thumbnails. Original committed page remains1. Settings readback confirms `grid_columns=2`, `grid_visible_pages=6`.
- Click outside panel at an observed slide point closes it without changing original slide. Progress focus visibly returns; Enter reopens the browser without another pointer click.
- `page-browser-final-thumbnail-jump.jpg`: whole-tile image click for physical3 closes panel and directly selects logical2/step1, updating real current/notes/next content.
- Titlebar close through Alt+F4 exits the owned test process. Final stderr is empty.

The first hidden-process launch did not expose a targetable window and was terminated as test setup; subsequent visible native launches were used for all observations.

## Boundaries

Independent notes rejection, stale target guards, caret/selection/paste, cancelled presses, busy/read-only/save failure and exact request retirement have actual GPUI automated regressions. They are not falsely claimed as exhaustive native observations. UIA text tree was empty; rich AccessKit text-selection support remains the recorded gap. Native snapshots in Ready do not establish physical audience projection acceptance; snapshot privacy regressions remain separate.

View review: checked2 / changed1 / left1. Page browser changed; base workbench composition retained. Layout-menu integration subsequently passed; final combined native observations are in the parent `research/verification.md`. No commit/archive performed.
