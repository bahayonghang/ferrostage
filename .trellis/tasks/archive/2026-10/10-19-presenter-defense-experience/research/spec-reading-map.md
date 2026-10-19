# Required state-contract reading map

The authoritative `.trellis/spec/frontend/state-management.md` is larger than the 32768-byte injection limit. This map prevents silently treating an injected prefix as the complete specification. It is a reading instruction, not a replacement specification.

Before implementing or checking a child, read the relevant sections from the live file in bounded ranges and follow any referenced contracts:

| Child | Required sections (current line ranges) |
|---|---|
| Fullscreen/layout | Core/navigation/fullscreen 1-102; display assignment/feedback 141-234; runtime settings 273-309; monitor recovery 400-end |
| Original Notes | Thesis AGENTS.md, slides/AGENTS.md and defense-slides spec listed in scope-revision.md; app layout child reads state ranges 1-102 and 235-350 |
| Page transitions | Core/navigation/fullscreen/click semantics 1-140; textures/settings/reload/media 235-399; monitor recovery 400-end |
| Overlay/timing | Core/navigation/fullscreen/click semantics 1-140; textures/reload/media 235-399; monitor recovery 400-end |
| Version | Backend windows-distribution spec and workspace metadata |
| Parent integration | All above sections |

Rediscover section headings if line numbers move. Tests/builds and native acceptance remain separate. The parent research states the proposed behavior changes; existing specs define the baseline that must be deliberately updated with the approved implementation.

For transition/timing implementation or review, also read `.trellis/spec/frontend/pdf-playback.md` in full. It is the completed cross-layer metadata/paint-event/dwell contract, including deferred-delivery and failed-reload recovery regressions.
