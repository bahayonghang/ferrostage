# Architecture

FerroStage splits the presenter into a pure state machine and IO crates that sit beside it. `ferrostage-core` does not depend on GPUI, PDFium, `lopdf`, or `notify`.

The current source connects lifecycle/window coordination, public pixel assembly, one PDFium worker, complete settings, external replacement and media playback. See [feature parity](feature-parity.md) for source and scoped verification status; implementation does not imply full native or performance qualification.

```text
ferrostage-app            composition root
  ferrostage-ui           GPUI presenter window; audience window only in Presenting
  ferrostage-settings     TOML; atomic replace of the configured settings file only
  ferrostage-core         SessionStage, commands, audience snapshot
  ferrostage-document     notes and media slots (presenter only)
  ferrostage-pdf          read-only structure and optional PDFium adapter
  ferrostage-render       one worker, generation checks, cache budgets
  ferrostage-media        pure animations and isolated-helper supervision
  ferrostage-platform     monitor topology and read-only external PDF watching
ferrostage-media-helper   separate optional-native GStreamer process; no GUI dependency
```

UI intents route through the coordinator to checked core commands or reload publication. The core's `audience_snapshot` produces projection metadata; the UI assembler adds only matching audience pixels before publishing `AudienceProjectionSnapshot`, the audience view's sole input. It never exposes the document, speaker notes, media paths, the full cache, or an arbitrary-page request API. The assembler keeps the previous complete projection until the replacement page frame is ready, so new metadata never accompanies old pixels. `Curtain` and black/white blank cover media, strokes, and laser immediately.

FerroStage opens the user's PDF read-only and never writes that path. Hot reload handles external replacement: debounce, obtain stable bytes with `%%EOF`, validate and pre-render a candidate, then commit a strictly newer `DocumentGeneration` with its document and visual in one main-thread transaction. EOF alone does not prove a valid or complete replacement. See [ADR-004](adr/ADR-004-lock-free-reload.md). Core accepts `CommitDocument(DocumentDescriptor)` and validates count/generation before updating metadata. The production reload path prepares and stages through the document/render/UI layers, awaits an opaque reversible worker commit, then applies `commit_reload` to the latest core state before retiring the old generation. Obsolete commits require rollback acknowledgement. The legacy `StageShadowDocument` compatibility command still performs a non-empty byte check; it is not the production reload transaction. The core does not keep or parse PDF bytes.

Settings use a unique sibling temporary file, such as `settings.toml.tmp-<pid>-<seq>`, followed by sync, close, and atomic replacement of the configured settings path. A custom `--config` filename supplies the temporary file's basename. FerroStage's own artifacts may use their own sibling temporary files. Neither write target is the open PDF.

Raster format is BGRA8 end to end, including pen color (`NormalizedStroke::color_bgra`) and `SharedFrame`. Do not convert to RGBA on the audience path.

## Session stage

`SessionStage` is `Idle | Ready | Presenting`. Loading a document enters `Ready` only. `StartPresentation` enters `Presenting`. `StopPresentation` returns to `Ready` and keeps the page and strokes. `CloseDocument` returns to `Idle` and clears the session. See [ADR-006](adr/ADR-006-session-lifecycle.md).

```text
Idle --LoadDocument(ok)--> Ready --StartPresentation--> Presenting
 ^                          | ^                            |
 +--CloseDocument-----------+ +----StopPresentation--------+
```

| Stage | Presenter window | Audience window | Timer |
| --- | --- | --- | --- |
| `Idle` | Empty: open, recent files, settings | Absent | Stopped at 0 |
| `Ready` | Current, notes above next, direct page selection, settings, monitor choice. Start is available | Absent | Stopped. First entry shows 0 or the countdown target. After Stop, the frozen reading stays |
| `Presenting` | Full workbench and Stop | Borderless fullscreen on the Audience monitor | Running (pause and reset allowed) |

The audience window exists only in `Presenting`. `AudienceProjectionSnapshot` includes `stage: AudienceStage` (`Curtain` or `Live`). Outside `Presenting` the snapshot is always `Curtain` and carries no visual. `Curtain` draws solid black. Core supplies stage and metadata with `visual=None`; the UI assembler adds only matching public PDF/media frames.

Commands such as page turns, laser, and blank are accepted in `Ready` and `Presenting`. Only `Presenting` produces audience pixels. Hot reload does not change the stage or the timer.

`StartPresentation { audience_available }` requires `Ready` and a resolved Audience. The only exception is one monitor with `display.single_monitor_mode = "windowed_audience"`, which allows a windowed audience on that screen. Zero monitors, an ambiguous mapping, and a missing Audience without that exception stay in `Ready`. Window creation failure also stays in `Ready` with no leftover audience window.

## Monitor roles

A display is `MonitorId` plus `MonitorRole` (`Presenter`, `Audience`, `None`), not an index. `enumerate_monitors` produces `MonitorInfo`. Pure `resolve_roles` returns `ResolvedTopology` (`presenter`, `audience`, `ignored`, `source`, `warnings`). There is at most one Audience. The only stored authority is `display.monitors`.

`SwapScreens` is an intent to the app role coordinator. One settings transaction exchanges the two role bindings and moves the windows. It does not flip a separate swap flag, and it does not change page or crop. A saved Audience that is missing, or an explicit `None`, does not take another monitor. Presenter absence may fall back to the primary after the saved Audience id is excluded, and warn. If the Audience disappears during `Presenting`: publish `Curtain`, then `Stop`, destroy the audience window, return to `Ready`, and keep the presenter windowed. A newly attached monitor does not take a role, and reconnect does not start the talk. See [ADR-007](adr/ADR-007-monitor-roles.md).

The platform implements stable `MonitorId`, enumeration and pure role resolution; UI coordinates native windows and the shared role-settings transaction. Core has no swap flag and rejects direct `SwapScreens` with `ScreenSwapRequiresCoordinator`. Legacy index settings are retained as unknown data with a rebind warning, not used as role authority.

## Render priority

The render worker uses these priorities. Direct page selection removed the original jump-preview priority from [docs/PROJECT_PLAN.md](PROJECT_PLAN.md) §4.6:

| Priority | What |
| --- | --- |
| P0 | Current audience page |
| P1 | Presenter current page, next page, and notes region |
| P2 | Audience prefetch of current ±1 |
| P3 | Thumbnails, only while the grid is open |

Each page turn increments an epoch to cancel queued stale work and reject late results. Priority chooses the next queued request; it cannot interrupt a synchronous PDFium call. Requests and results carry generation, page, crop, target pixel size, priority, and epoch.

The active generation continues serving while a candidate loads and pre-renders in a separate namespace/result channel. Candidate identities are checked against their own preparation attempt and worker ownership, separately from active-frame identity checks. Validate the latest attempt token before committing descriptor, document, and audience visual together, then retire the old generation's resources. All PDFium initialization, loading, rendering, and release stay on the single render worker; only owned metadata and BGRA pixels leave it.

## Media ownership

The GUI uses pure GIF/WebP decoding and a background supervisor. Native audio/video lives in the separate optional GStreamer helper; the GUI has no GStreamer/GLib imports. One decoder/audio owner is active, and previous child exit or kill/reap precedes another. Ready and notes-only annotations never start playback. Page, blank, Stop, Close, published generation and successful crop reconfiguration invalidate accepted frames; leaving blank does not revive the old instance.

Source paths, embedded bytes and controls stay on the Presenter side. A bounded annotation-aspect canvas is conservatively cropped to the public intersection before a shared BgraFrame enters the snapshot. The public slot ID is an opaque process-wide playback-instance identity, not a PDF annotation index. Both windows use their existing budgeted texture cache and paint PDF, media, then ink/laser. Missing native runtime leaves PDF and pure animations available. See [ADR-009](adr/ADR-009-media-helper.md) and [packaging](packaging.md) for the selected native closure and its separate license audit.
