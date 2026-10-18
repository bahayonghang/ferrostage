# ADR-006: Session lifecycle

## Status

Core lifecycle, timer and projection contracts implemented. Window coordination and live display acceptance remain tasks 07/08.

## Context

Opening a PDF must not take the audience display or start the timer. The speaker needs a ready state to check notes, settings, and monitor roles, then start explicitly. The original skeleton had no `SessionStage`; task 04 separates preparation from presentation.

## Decision

`SessionStage` is `Idle | Ready | Presenting`.

- A successful `LoadDocument` moves `Idle` or `Ready` to `Ready` and resets elapsed time to zero. `LoadDocument` during `Presenting` is rejected.
- Production code enters through `idle()` then `LoadDocument`. `new(page_count)` is only a convenience constructor for `Ready`. Tests do not special-case an implicit `Presenting` state.
- `StartPresentation { audience_available }` is valid only from `Ready`, and only when the input precondition holds: a resolved Audience, or exactly one monitor with `display.single_monitor_mode = "windowed_audience"`. That setting is the single-monitor exception. Zero monitors, an ambiguous identity mapping, and a missing Audience without that exception stay in `Ready`, with no audience window and no timer growth.
- `StopPresentation` returns to `Ready`, keeps the page and strokes, and freezes the timer reading.
- `CloseDocument` returns to `Idle` and clears the document, strokes, and timer.

The audience window exists only in `Presenting`. `AudienceProjectionSnapshot` includes `stage: AudienceStage` (`Curtain` or `Live`). `Idle` and `Ready` snapshots are always `Curtain` and carry no visual. `Curtain` draws solid black. If window creation fails, the session stays `Ready` and no audience window remains. Publish `Curtain` before an audience window is destroyed.

The core does not read the system clock. The app passes a monotonic instant into `apply_at(command, Instant)` / `elapsed_at`. There is no clock-reading `apply` convenience method.

- Each `Start` zeroes the timer and runs it.
- `Pause` freezes it. `Resume` continues it.
- `Reset` during `Presenting` zeroes elapsed time and keeps the pause flag.
- `Stop` freezes the reading. `Ready` and `Presenting` use the same `timer.mode` format: count-up shows frozen `elapsed/target`; count-down shows frozen `target-elapsed/target`. The first `Ready` has `elapsed = 0`, so count-up shows `00:00` and count-down shows the target. After `Stop` the displayed reading does not jump.
- `Close` zeroes the timer. Count-down may show overtime.

Jump preview belongs to the presenter. In preview, next and previous move only the preview. Commit (Enter) updates the audience page. Cancel (Esc) drops the preview. Preview does not change the committed audience page.

Hot reload does not change `SessionStage` or the timer. `SwapScreens` is not a session transition ([ADR-007](ADR-007-monitor-roles.md)).

## Consequences

The app main thread coordinates audience-window creation with `Start`. Core enters `Presenting` only through `StartPresentation`. Non-presenting snapshots stay `Curtain` with no visual, including when no audience window exists.
