# ADR-004: Lock-free reload

## Status

Policy accepted. The watcher loop is not started.

## Decision

- Debounce filesystem events for 300 ms (`HOT_RELOAD_DEBOUNCE`).
- Treat the file as ready only after the bytes contain `%%EOF`.
- Publish with `PresentationCommand::StageShadowDocument` and a strictly greater `DocumentGeneration`.
- Drop renders whose generation does not match (`ferrostage_render::is_current`).
- If FerroStage writes a PDF, write `*.ferrostage.tmp`, validate, then atomically rename. Do not truncate the destination in place.

`notify` 8.2.0 is linked so the pin is compiled. 9.0.0 is still a release candidate and is not used.

## Consequences

The core does not store the shadow bytes. The document and PDF crates own the buffer after the generation check succeeds.
