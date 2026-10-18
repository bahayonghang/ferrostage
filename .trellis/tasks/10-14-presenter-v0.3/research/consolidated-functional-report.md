# Consolidated functional evidence — 08–11

Latest: user resumed Computer Use, then requested quick core acceptance while doing other work. Four final-build runs completed and exited normally. This is bounded core evidence, not full native qualification. Source and independent automatic checks are complete; unobserved actions remain NOT_RUN. The user subsequently authorized commit and implementation-scope archival; see implementation-closeout.md. No push. Apply ../acceptance-policy.md; do not tick composite original ACs.

## Exact build and retained automatic checks

GUI SHA256 `4BEBEFA5810F93CCB2E9FDA3E8134AACB6EBC24B55E46D0A0D06308387D9CE55` (42,889,216 bytes); helper `312752079A67C00C03F7FCF6C843D57D9E19996FC1445AD82026B56A51B96641` (799,232 bytes). Preflight verified169 source inputs. All launches used these binaries, isolated settings and PDFium7881. Complete runtime has47 selected files; partial has46, omitting only bin/gstapp-1.0-0.dll. Each run uses a fresh owned GStreamer registry.

11 independent-review.md retains180 default CI,119 parent UI tests plus one isolated replay (not120 distinct tests),40 actual PDFium tests,17 media tests (8 unit+9 actual silent helper integration), strict feature/helper/workspace lint and typecheck, all-feature cargo deny, post-CI builds and GUI PE isolation.08/09/10 independent reports retain their earlier version-scoped gates. Desktop actions changed only owned test settings/fixtures/evidence, not product source.

## Observed core results

| Behavior | Result | Exact evidence and limits |
| --- | --- | --- |
| Ready/current/next/private notes; Start/public Audience | PASS bounded | final01-ready/start/audience-public. Private FS-NOTE markers only on Presenter in these samples. |
| Numeric preview3 and Enter commit | PASS bounded | final01-preview3/commit3; physical3, logical3 step1/2 and private03A. Cancel/overlay broader coverage remains earlier08 evidence. |
| Highlighter, eraser point hit, laser | PASS bounded | final01-highlighter-presenter/audience, eraser-hit, laser-presenter/audience. Drag across stroke did not remove it; erased/erased-settled retain this failed attempt. No drag-erasure PASS. |
| Black/white covers | PASS bounded | final01-black-audience/white-audience. Presenter retains workbench. |
| Settings open and theme save/restart | PASS bounded | final01-settings-general/light; UI saved feedback, config readback and final02 light startup. Not all seven categories/consumers re-exercised on desktop. |
| External sibling rename hot reload | PASS bounded | final02-after-rename-initial/orange-audience. Owned blue one-bar PDF replaced by orange three-bar PDF, both windows update; page1/5 and running timer retained. No all-frame/no-black or timing claim. |
| Partial/chunked external write | NOT_RUN | Spawned producer did not establish any write; file stayed original. Misleading historical labels during-partial-write/after-chunked are ordinary old-frame observations, NOT partial-write proof. Direct Rename was separately successful. |
| GIF real frames on both windows | PASS bounded | final03-gif-start (red), gif-later (green), gif-audience (green). Samples establish animation/public output, not frame-perfect synchronization. |
| WebP | PARTIAL | final03-webp shows decoded frame; no multiple-frame desktop sequence. Actual animation regressions separately pass. |
| H264/AAC MP4 frames/progress and pause | PASS bounded | final03-video-playing/later/audience show moving white ball and0→2/8s; Navigate click leads to Play at3/8s in pause-settled. No audible-output, timing, exact-loop or pause-frame retention claim. |
| Media-enabled GUI with one DLL missing | PASS bounded | final04-gif-settled shows pure animation; runtime-unavailable shows local media-unavailable rectangle, audience-fallback retains public PDF. No crash; normal exit0. Full absent-runtime GUI variant NOT_RUN. |
| Normal Close and process cleanup | PASS | All four *-exit.json report0; final process inventory empty for ferrostage/helper and owned chunk producer. Original notes/media PDF hashes verified unchanged. |

All screenshot/AX JSON and PNG files are under native-final. Run labels are final01-workbench(PID91764), final02-reload(53268), final03-media(91336), final04-missing-dll(66280); corresponding launch/exit/stdout/stderr files retain provenance.

## Not accepted or not exercised

Grid/thumbnail/wheel navigation is NOT_VERIFIED: grid attempt showed a numeric Jump modal; thumbnails/media rows were not visible in the initial scroll position. Read-only quick-native-diagnosis.md confirms a scrollable body and separate Grid/Jump actions; no PASS inferred from AX-only controls. Current-slide lower content was below the scroll viewport in captured default-size Presenter images; Audience showed full slide. Do not infer a size-matrix pass. Video pause control state was observed; the black-background frame retains its white ball in the settled screenshot. Extended retention/loop checks were not run.

NOT_RUN this final pass: clear strokes, timer resume/reset/Stop sequence, document crop/group overrides, every settings consumer/category, shared display Apply/Cancel/topology, language restart, read-only/failure/Retry; damaged/rapid/partial reload and abort recovery; original Beamer Movie desktop, audio-only/audibility, embedded media/source failure, crop/notes-audio exclusion, media annotation/blank/page/reload invalidation matrix. Existing real automatic and older native evidence remains separately scoped.

DEFERRED_BY_USER / NOT_RUN: physical plug/port/identity/latency, true single/zero displays, exhaustive native/theme/size/DPI/focus/cursor matrix,80-page/30-minute and repeated benchmarks,4K alignment/VRAM,1080p60/15-minute media pacing/drop/VRAM/audio-stop timing. This report makes no full Gate D/original AC claim.
