# Design

Trace SettingsCategory(Displays) through Shell capture and the shared display panel before changing production code. Distinguish unbounded recursion from finite excessive stack frames with native reproduction and compiled function evidence. Fix the owning view construction boundary only. Both full Settings and the standalone display panel consume that boundary.

## Proven cause and bounded repair

The native debug PE reserves `0x100000` bytes for its main-thread stack. Actual compiled frames for `Presenter::render` (`0x39750`), `Presenter::settings_panel` (`0x718B0`), shared `view::panel` (`0x9ED0`), and `view::body` (`0x4F0E0`) sum to `0x103FB0` (1,064,880 bytes), exceeding that reserve before additional GPUI/button calls. This explains why General renders but Displays fails on entry. This is finite stack exhaustion from large by-value fluent UI construction, not a discovered recursion.

Split shared display-body construction into bounded helpers along its existing sections and monitor rows. Preserve the same element hierarchy, order, identifiers, style values and intents. Test the actual input/render path inside a subprocess with the native 1 MiB stack budget so a stack-overflow abort cannot take down the normal test harness. Run that regression before and after the repair.

The first shared-body split alone still overflowed in both the 1 MiB embedded-host test and a fresh native run. The caller `settings_panel` remained `0x718B0` (465,072 bytes). Extend the bounded repair to `presenter/settings_view.rs`: move the General-only recent-file composition to a helper taking and returning the original content element. This removes its temporary reservation from the Displays call chain without adding a wrapper or changing history semantics. Compare the resulting compiled frame and rerun both evidence paths.

Preserve MonitorId authority, draft semantics, deferred input and existing runtime settings transactions. No dependency, linker-stack or visual redesign changes. Rollback is the local view/test patch; no settings migration is required.
