# Implementation plan

## Checklist

1. Add `Intent::DismissDocumentWarning(String)` and handle it on `Shell`. Store the string set and the generation it belongs to. Ignore a dismiss whose text is not in the current `document_warnings` list.
2. Split `PresenterData.warnings` into `shell_warnings` and filtered `document_warnings`.
3. Render shell warnings as `shell-warning-{index}` with the existing bare line, on Idle and on a loaded document. Render document warnings only on Ready and Presenting, using the geometry in `design.md`.
4. Add `dismiss-warning` to both locale files. Wire the close control through `activation_target` with the quiet 44×44 face.
5. Add one GPUI test on the existing `input_presenter` / `finish_import` path:
   - Two pages that share a label, so `LoadedDocument` emits `OverlayHeuristic`, plus one extra `DocumentWarning` pushed on the loaded document for the stack assertion.
   - Assert flag ids, content height before and after dismiss, flag bottom against `workbench-controls`, dismiss survival across `reconfigure_layout` with the same generation, return after a new generation, shell-warning isolation, and Escape leaving the flag up.
   - Assert the audience window, when the harness already has one, has no `document-warning-0`. Do not open a new audience scenario only for that absence.

## Validation

- `cargo test -p ferrostage-ui --features gpui,test-support --lib document_warning_flag -- --test-threads=8`
- `just ci` before calling the task done.

Headless bounds do not prove the dark-theme paint. After `just ci`, the Ready window with a labeled multi-step PDF is the visual check: flag in the bottom-left, toolbar clear, close removes it.

## Risks

- `presenter.rs` render is large. Keep the flag builder in `presenter/layout_navigation.rs` next to `document_warnings`.
- `activation_target` tab rules must stay intact. The new id must not become a tab stop while a layout menu, utility menu, start menu, or page browser is open.
- Reconfigure rebuilds `LoadedDocument`. The survival assertion has to use a warning the builder emits again (`OverlayHeuristic` from a shared label), not a warning that existed only because the test pushed it.
- Clipping a tall diagnostic stack is intentional. Do not add a scroll view inside the flag.

## Before `task.py start`

`prd.md`, `design.md`, and `implement.md` are the planning set. Curate `implement.jsonl` and `check.jsonl` with the frontend spec entries. Do not start until the user approves this plan.
