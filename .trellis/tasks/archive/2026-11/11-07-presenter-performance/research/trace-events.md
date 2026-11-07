# Native performance trace

Set `RUST_LOG=ferrostage_perf=info` and capture the release desktop process's stdout while opening the fixture and using its ordinary Start, Next and Previous controls. Use a separate settings file with fixed output sizes and budgets. The trace uses a process-local monotonic clock in microseconds. Pair events within one process. The clock restarts on every new process.

| Event | Matching fields | Measurement boundary |
|---|---|---|
| perf_open | token | Accepted import, before background read |
| perf_read, perf_structure | token, ok | Completion observed by shell after background read and structural parse |
| perf_candidate | token, generation | Identifies which candidate belongs to this open |
| perf_metadata, perf_candidate_raster, perf_commit | generation | Shell consumes each worker reply; commit checks matching generation |
| perf_preview | generation, page, width, height | First successful Current composite for this generation in this presenter; cached fallback and failed paint do not count |
| perf_start | generation, page | Entry into a start attempt, before window placement |
| perf_input | direction, generation, page | Mapped Next/Previous GPUI action callback, before deferred coordinator work; page here is the source |
| perf_selected | generation, navigation, page, window | Playback selects a target identity for an audience window |
| perf_submit, perf_result | id, surface, generation, page, epoch | Submit to worker and consume a matching result. Submit carries target dimensions. Result includes cache_hit and, on a real raster, render_us |
| perf_audience | generation, navigation, page, window, phase | Accepted FirstVisible or Settled acknowledgement, timestamped at successful audience paint |

Compute open-to-preview from the opening token and matching candidate generation. For each input, match the following selected navigation to the acknowledgement by generation, navigation, page and window. Keep unmatched, failed and superseded inputs in the denominator. With authored transitions, FirstVisible can be the first successfully painted transition frame at zero progress; Settled identifies completion of the new target. Neither includes OS compositor Present or projector latency. CPU render time excludes queueing, upload and paint. A worker cache hit differs from a GPU texture hit.
