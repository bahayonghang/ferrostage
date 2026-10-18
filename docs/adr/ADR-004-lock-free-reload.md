# ADR-004: Lock-free reload

## Status

Policy accepted for external PDF replacement. The watcher loop is not started. FerroStage does not write the user's PDF.

## Decision

- Debounce filesystem events for 300 ms (`HOT_RELOAD_DEBOUNCE`).
- Require a stable read and `%%EOF` before candidate validation. EOF alone is not a completeness or validity guarantee.
- Load, inspect, pre-render, and release PDFium objects on the single render worker. Candidate metadata first feeds the document map; the mapped position then determines P0/P1 pre-rendering.
- Keep candidate work in a separate namespace/result channel while the active generation continues serving. Do not apply the active-only `ferrostage_render::is_current` filter to candidates. Reject stale attempts and late results against their own identity.
- On the main thread, validate the latest attempt token and commit the descriptor, document, and audience visual together with a strictly greater `DocumentGeneration`. Commit the worker candidate, then retire the old UI, CPU-cache, and PDFium resources in their owning execution domains. Close invalidates pending attempts.
- An external process replaces the PDF. FerroStage opens that path read-only, reads a shadow copy, and closes the handle. It does not truncate, rewrite, or atomically replace the user's PDF.
- A failed reload keeps the previous generation, stage, timer, and frame.
- Settings persistence is a different file. The settings store writes a unique sibling `<config-file-name>.tmp-<pid>-<seq>` (for example, `settings.toml.tmp-<pid>-<seq>`), syncs and closes it, then atomically replaces the configured settings path. Custom config filenames are respected. These writes never target the user's PDF. FerroStage's own artifacts may use their own sibling temporary files.

`notify` 8.2.0 is linked so the pin is compiled. 9.0.0 is still a release candidate and is not used.

## Consequences

The core does not store or parse PDF bytes. The PDF worker owns the bytes throughout candidate validation and active rendering. The skeleton command `StageShadowDocument` still carries read bytes for a generation/non-empty check; task 10 replaces that payload with a validated descriptor. That skeleton check does not implement the reload transaction above and is not a write path.
