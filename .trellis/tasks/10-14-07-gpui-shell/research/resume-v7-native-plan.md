# V7 diagnostic native plan

Root controls UI only after serial implement/check and a verified V7 freeze. No parallel native process. Use isolated task config and existing hash-bound PDFium/PDF. Set RUST_LOG=info,ferrostage_ui::shell=debug only in the runner process and restore it afterward. Preserve all V6 receipts.

1. Startup/Ready: record source-attributed errors and actual viewport_size/scale on initial native bounds callback.
2. Native titlebar menu Size: try keyboard/menu sizing through supported Sky API, then verify actual viewport logs. An unchanged screenshot does not establish requested size. Test supported1440x900,800x600,390x844,320x640 where physically achievable, both themes; no guessed logical client sizes.
3. Start/Swap/Stop: confirm actual placement and collect callback-source log locations, then normal titlebar Close and exit/inventory before next launch. Preserve role changes in before/after receipts and restore task-owned original profile after use.
4. Only after exact diagnosis, release source freeze for minimal regression-first repair and serial independent check; no vendor error suppression or speculative source change.

Live OS theme event preparation is blocked: supported Sky launch of verified C:/Windows/ImmersiveControlPanel/SystemSettings.exe returned no targetable window, and refreshed window inventory found no Settings window. No system theme was changed. Keep live-theme NOT_RUN unless a supported UI surface becomes available. Cross-window drag/drop and transient Curtain capture remain subject to supported API observability; do not synthesize their evidence. Physical unplug/reconnect/port/latency and true single/zero screens remain NOT_RUN by user skip/environment.
