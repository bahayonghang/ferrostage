# Execution

1. Reproduce the ordinary `just dev` Settings > Displays crash and collect source/compiled-frame evidence.
2. Add a focused regression; confirm failure against the original code. Make the smallest owning-boundary fix after stating the proven cause.
3. Run focused display/settings GPUI regressions, GPUI strict clippy, and `just ci`.
4. Rebuild the native debug executable; click both display hosts, revisit another category and close normally; inspect process output.
5. Record cause, same-pattern sweep, checks and limitations; update the relevant frontend spec.
