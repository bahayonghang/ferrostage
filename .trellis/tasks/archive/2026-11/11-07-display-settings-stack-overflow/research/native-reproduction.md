# Native Windows reproduction

## Baseline

- Source: `dev`, HEAD `73ff54e`, clean before this task. Current workspace is version 1.3.0; the user supplied a 1.2.1 crash log.
- Command: `just dev`, normal unoptimized debug profile, `gpui,pdfium`, locked dependencies. Shared mbx cache access required sandbox elevation; no build settings were changed.
- Real window: launched debug `ferrostage.exe` from Cargo's mbx-resolved target directory; no PDF opened and no preferences edited.
- Input via Computer Use: Home > Settings opened General successfully. Clicking the Displays category terminated the process.
- Complete failure tail:

```text
thread 'main' (30236) has overflowed its stack
error: recipe `dev` failed on line 56 with exit code 1
```

The current version reproduces the reporter's symptom. This establishes a native red result, not yet a root-cause diagnosis. Full Settings calls the shared `display_settings::view::panel`; the standalone host calls the same boundary.

## Retest

The first display-body decomposition (detection, overview and monitor rows) compiled but was insufficient: the 1 MiB embedded-host regression still failed, and a fresh `just dev` native run agreed. Home > Settings > Displays terminated with main thread 6536 stack overflow and recipe exit 1. The standalone constrained-test host passed. This preserves the native/test agreement and requires further measured frame reduction before acceptance.

## Final native acceptance

- Fresh `just dev` rebuild, unchanged debug profile and 1 MiB executable stack, final display helpers plus General-only recent-file helper.
- Home > Settings > Displays: PASS; all three actual monitor cards, topology and role state rendered.
- Expand first monitor details: PASS; identity/scale diagnostics appeared without termination.
- Return to General: PASS; existing language/theme/font controls and preview remained visible.
- Close Settings, open Home > Display settings: PASS; same three-monitor topology and details rendered in the standalone host.
- Cancel standalone settings: PASS; returned to Home without applying or saving a display allocation.
- Click native title-bar close: PASS; fresh window inventory contained no FerroStage window, `just dev` exit code 0, full process output contained no stack overflow or late handle diagnostics.

This is ordinary-window native verification of the reported crash and directly related controls. No physical hotplug, DPI/window matrix or presentation-allocation change was performed.

Default CI initially hit Windows file-lock error 5 while the native executable was running. It is rerun after normal app exit; this environmental failure does not establish a source/test failure.
