# V7 native attribution checkpoint

2026-10-14 America/New_York /2026-10-15 UTC. Frozen V7 exe14dfd1f97b35df5611d94a034e7cd474a4d0556f9f5481ff2b76002f5839b309, RUST_LOG=info,ferrostage_ui::shell=debug. Independent diagnostic review passed before launch.

`resume-v7-dark` PID7412/session59154: startup exact error is `vendor/gpui/src/window.rs:1914`, the active-status callback. Native bounds observer records Presenter B viewport1100x760, scale1.25. Titlebar context menu Size was visible, but the subsequent operation did not enter an effective resize; Right navigated page1→2 and actual viewport remained1100x760. Do not claim requested size acceptance.

Start then Swap committed Presenting2, Presenter A/Audience B. Actual A viewport1320x912/scale1.5; B fullscreen2048x1152/scale1.25. No additional Swap ERROR was observed in this run. `resume-v7-after-swap` screenshot is occluded by another app and is excluded as visual acceptance; after activation, `resume-v7-after-swap-visible` shows the real presenter and Stop at page2. Native viewport was observed; the full size/state matrix remains unverified.

Actual titlebar AX Close during Presenting ended PID7412 exit0 at03:38:43.019Z. Logs attribute close errors independently to GPUI window.rs:1914 and pinned gpui_windows/src/window.rs:623 (RevokeDragDrop,0x80040102),624 (DestroyWindow,0x80070578). No dispatcher error in this run; prior sporadic dispatcher source remains unverified. This confirms the concrete close-path diagnosis but implements no repair. Preserve all V6 and V7 diagnostics and failures.

User then requested the task plan replace the toolbar Swap workflow with detected/numbered displays and explicit application/PDF destinations. Root ended native run before editing planning; no further native launch. Task-owned a-dark restored from this run's before bytes, after bytes retained. New display workflow is PLANNED, not implemented or accepted. Gate B remains incomplete and physical items NOT_RUN.
