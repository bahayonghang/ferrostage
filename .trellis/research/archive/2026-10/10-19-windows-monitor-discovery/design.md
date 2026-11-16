# Design

## Confirmed cause and boundary
The watcher correctly publishes recovery (including unchanged monitors), but Shell stores its failure in the shared operation-error string and never clears it on Changed. Initial startup failures use the same English string path. `windows::enumerate` also mislabels before/after metadata changes as device/geometry mismatch, possibly printing identical key lists. Native current state is healthy (three monitors); the screenshot's initial system trigger remains unknown. Do not replace DC acquisition or weaken completeness based on an unconfirmed clipping hypothesis.

## Platform
Retain device/geometry validation and fail-closed semantics. Give before/after device changes their own typed error, retaining both device-connection lists for diagnostics. Extract the small snapshot validation so the identity-change-with-identical-keys case has a deterministic regression. Keep native geometry/scale, identity, device-context RAII and watcher debounce unchanged. The existing native GDI-count regression is mandatory.

## UI error ownership
Store monitor discovery failure separately as a typed `Option<PlatformError>`. Initial failure and Unavailable set it; successful Changed clears only that field. Global operation errors remain independent. Render localized monitor messages in presenter capture and the shared display panel. Include watcher-start/other platform variants via the locale mapping. Retain typed structured diagnostics in tracing when a failure is first observed; do not log each healthy poll.

## Recovery UX
Use the existing shared panel in Settings > Displays and Display settings, and the existing RefreshDisplays action. Show concise Chinese/English steps: wait for cable/display switching to settle, use Windows Settings > System > Display > Multiple displays > Detect when a screen is missing, use Extend when separate presenter/PDF screens are wanted, then Detect again in FerroStage, select application/PDF destinations and Apply. Do not change Windows configuration automatically or advise resetting all settings. Add a concise user-facing troubleshooting section to existing documentation. A recovered state must stop showing failure-specific help; a small permanent recovery hint may link the path.

## Risks
Never auto-select a replacement audience or accept partial/stale topology. Do not erase unrelated errors on topology success. Native UI automation is limited in this environment; use the real process, logs, read-only UI inspection and existing GPUI input tests, reporting any unavailable checks accurately. No physical plug/port/size matrix or installed-app replacement is implied by a source fix.
