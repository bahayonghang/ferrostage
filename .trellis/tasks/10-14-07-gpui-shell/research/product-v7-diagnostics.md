# V7 diagnostic handoff

2026-10-14 America/New_York; command/freeze receipts use UTC 2026-10-15.

**Ready for independent check, then Root native attribution. This is diagnostic instrumentation, not a retirement fix.** Root released V6 after both owned native processes exited; its light-run unknown exit remains excluded from normal-close acceptance. All older receipts/freezes remain unchanged.

## Narrow change

- `crates/ferrostage-app/src/main.rs`: enable formatter source file and line. Pinned GPUI `ResultExt::log_err` already supplies caller metadata, including for the otherwise empty target of this vendored source.
- `crates/ferrostage-ui/src/shell.rs`: both existing bounds observers still set the existing confirmation flag, then call one DEBUG-only logging helper. It reads the currently borrowed Window's WindowId, logical bounds, drawable viewport size, scale factor, GPUI display ID and fullscreen state. It does not re-enter a window or borrow Shell. Confirmation, input, rendering and retirement decisions are unchanged.

No vendor/core/manifest/lock/policy/test change. No tautological diagnostic test was added. Existing real-GPUI regressions were rerun because the instrumentation executes inside its observer callbacks. The V6 source copies and focused V6-to-V7 patch are retained; DESIGN.md is the pre-existing documented V6 closeout delta and is not part of this implementation.

## Returned checks

| Evidence | Result |
| --- | --- |
| `product-v7-fmt.log` | PASS, exit0 |
| `product-v7-just-ci.log` | Environment failure, exit1: sandbox denied shared target `.cargo-build-lock`; product checks did not reach completion |
| `product-v7-just-ci-retry.log` | PASS, exit0 with authorized cache access; fmt, strict default clippy, 145 default tests and dependency policy |
| `product-v7-clippy.log` | PASS, exit0: app/UI all-features/all-targets/locked/offline, `-D warnings` |
| `product-v7-headless.log` | PASS, exit0: 49 existing GPUI tests, none ignored |
| `product-v7-build.log` | PASS, exit0: app `gpui,pdfium`, locked/offline |
| `git diff --check` | PASS, exit0; only pre-existing LF/CRLF notices |

Every Cargo/just command has an adjacent JSON argv/start/end/exit receipt. The existing upstream HarfRust dead-code warning remains visible; no waiver was added. The full all-features dependency policy evidence remains applicable because its graph and sources are unchanged. No new claim of direct upstream all-crate strict lint is made.

## Frozen identity and native use

- Freeze: `product-native-freeze-v7.json`, SHA256 `167fc7e478f8e45293cd4cbb24374b0a3e5d04f36a12e7784c3d1dc027f6ad54`.
- 602 source inputs, aggregate `20456d033d3f4b826a2c0e0756ec4e724e602ab1a2b925b813a947e29935982e`.
- Executable `target/debug/ferrostage.exe`: `14dfd1f97b35df5611d94a034e7cd474a4d0556f9f5481ff2b76002f5839b309`, 36,992,000 bytes.
- Same verified PDFium and PDF fixture; lock unchanged `2557e10b37fa5cfe63e78ce19965b8bbbd83438bdd540eed50405fbbc3dbc23e`.

Root should launch with `RUST_LOG=info,ferrostage_ui::shell=debug` using its isolated native runner and the new freeze. File/line output is unconditional; bounds events require shell DEBUG. Bounds and viewport are logical GPUI pixels; `scale_factor` supplies the physical-pixel ratio. A bounds event can also follow move/activation, so the record is an observation, not proof that a requested resize succeeded. Do not equate the outer screenshot rectangle with the drawable viewport.

Collect exact ERROR source locations separately for startup cancellation, confirmed Swap and actual title-bar close, alongside actual client dimensions and owned process shutdown. `retirement-v7-diagnosis.md` contains the source-backed candidate paths and proposed regressions. Lifecycle fixes remain pending attribution. Physical tests and live OS theme event remain NOT_RUN; task07/Gate B incomplete; no08–11, commit, push, archive, journal or UI action by this agent.

All implementation commands have returned. Product source/binary are frozen for serial independent check and Root native inspection; only this research evidence closeout followed the freeze.
