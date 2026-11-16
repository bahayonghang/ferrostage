# Home recent files and actionable PDF errors

## Goal
Improve the screenshot's unstructured recent-file list and partially English PDF parsing failure. Keep the established FerroStage home identity while making file history easier to scan and adding real last-successful-open times.

## Confirmed scope and decisions
The user explicitly approved a separate task and implementation for Chinese errors, history table polish and last successful open timestamps. Legacy entries without trustworthy timestamps show an unknown value, never invented file times. The current parse error does not prove compilation is the cause: incomplete compiler output is one possibility alongside malformed files.

## Requirements
- R1: Convert import read/structure failures to Chinese/English catalog copy with actionable recovery, keeping technical diagnostics in tracing. Do not assert that the compiler is running without evidence. Keep the previous document intact on failure.
- R2: Use a compact, flat history table/list: readable filename and parent path hierarchy, aligned last-opened column and quiet separators/header; retain warm dark/light themes and existing font preferences, natural-width Open action, focus/press feedback and whole-row activation.
- R3: Record timestamps only after successful document import. Keep existing v1 recent path list compatible. Persist times across restart; failures do not refresh history. Prune metadata consistently when history is trimmed/cleared. Read-only and unsaved-state contracts remain intact.
- R4: Old/missing/invalid timestamps display a localized unknown state. Formatting is readable local date/time with no filesystem metadata lookup during rendering. Preserve the current three-entry home scope and settings history behavior.

## Acceptance criteria
- AC1 (R1): A truncated PDF reproduces the screenshot's raw parser error before the fix and shows localized, non-assertive wait/retry guidance afterward; missing/locked/permission/geometry failures remain distinguishable. A failed attempt preserves history, timestamps and active document.
- AC2 (R2/R4): Normal-window home with the real font preference, long paths, known/unknown times, empty and error states is readable; name/path and date column geometry is checked. Existing row click, Enter/Space and full-path inspection remain usable.
- AC3 (R3): Settings round-trip preserves valid times and unknown TOML leaves, old v1 files remain loadable, invalid metadata falls back locally, and successful open/reopen, failure, clear/limit, save failure and read-only behavior have regression coverage.
- AC4: Workspace CI, product build/lint and relevant GPUI/native PDF tests pass. Native screenshot smoke is attempted once with the rebuilt application; unavailable visual proof is reported honestly without a size/DPI matrix.

## Boundaries
Home presentation, import feedback and recent-history storage only. No automatic compiler detection/retry, no PDF writes, no workbench/display redesign, no new dependency, no changes to the previous uncommitted monitor repair, no commit/push/publication or automatic installation in this request.
