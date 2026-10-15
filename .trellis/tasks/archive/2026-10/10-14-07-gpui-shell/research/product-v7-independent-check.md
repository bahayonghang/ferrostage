# Independent V7 diagnostic check

2026-10-14 America/New_York; audit timestamps are UTC 2026-10-15.

**Disposition: suitable for Root's source-attributed native diagnostic run.** No material defect was found in this diagnostic-only change. This is neither a retirement fix nor task07/Gate B acceptance. Product code, executable, vendor sources and existing evidence were not edited by the reviewer.

## Findings (fixed)

No product fix or additional test was necessary for the scoped instrumentation.

- File: `.trellis/tasks/10-14-presenter-v0.3/research/gate-b.md`, new lines3-6.
- Issue: concurrent Root edits introduced CRCRLF/trailing whitespace; initial whole-tree `git diff --check` returned2.
- Fix: Root corrected its own lines after notification. The reviewer independently reran whole-tree `git diff --check`, exit0. Frozen product files were untouched.

## Source review

- `crates/ferrostage-app/src/main.rs:49`: formatter file/line output exposes metadata already attached by pinned `gpui_util::log_error_with_caller`. Its empty target for a source outside `crates/` does not erase `Record.file` or `Record.line`.
- `crates/ferrostage-ui/src/shell.rs:662,677,746`: the existing bounds observers still set the confirmation flag first. The new DEBUG helper reads the borrowed window directly; it does not update a window handle, borrow Shell, schedule a callback, or change a lifetime/confirmation decision. Pinned `Window::bounds_changed` refreshes viewport/scale before notifying these observers. The logged viewport is drawable logical size; multiplying by scale gives the physical-pixel dimensions. A bounds notification alone does not prove the requested resize succeeded.
- V6 retained source copies match their freeze. The only V6-to-V7 source-input differences are these two first-party files and the previously documented DESIGN.md status update. All 498 vendor inputs, dependencies, lock, policy and existing tests are unchanged. No notes/document bytes enter diagnostics. No platform template or contract update is required by this narrow change.

## Verification

The independent hash audit returned 0 and wrote `product-v7-independent-audit.json` (SHA256 `368caf1078c8e7459dd5c86b687d4a7165787e456dbdd64cc777e83f7f0c9b82`). It independently enumerated the frozen source scope and hashed every input; there are no omitted/extra scoped files or mismatches.

- Freeze SHA256: `167fc7e478f8e45293cd4cbb24374b0a3e5d04f36a12e7784c3d1dc027f6ad54`.
- Sources: **602/602 match**, aggregate `20456d033d3f4b826a2c0e0756ec4e724e602ab1a2b925b813a947e29935982e`.
- Executable: `14dfd1f97b35df5611d94a034e7cd474a4d0556f9f5481ff2b76002f5839b309`, 36,992,000 bytes. Resolved path, DLL, fixture, lock, build log and clippy log also match the freeze.
- V7 evidence manifest: **22/22 match** bytes and SHA256. Prior native-independent213/native-V6 54/native-V5 108/implementation-V6 26 records also match.

Reviewed implementation commands were already returned before the V7 freeze. The reviewer reused their bound results and did not duplicate Cargo work or rebuild the frozen executable.

| Check | Result |
| --- | --- |
| Format | PASS: `product-v7-fmt.json`, exit0 |
| Lint | PASS: `product-v7-just-ci-retry.json` default strict clippy; `product-v7-clippy.json` app/UI all-features/all-targets locked/offline `-D warnings`, exit0 |
| TypeCheck/native compilation | PASS: `product-v7-build.json`, app `gpui,pdfium` locked/offline, exit0 |
| Tests | PASS: independently summed **145 default**, **49 GPUI**, zero failed/ignored |
| Dependency policy | Default deny PASS in V7 CI. Prior full all-feature policy evidence remains applicable to the unchanged graph/source set; no new direct upstream strict-lint claim |
| `git diff --check` | PASS, independently run for product and final whole tree, exit0 |

The initial `product-v7-just-ci.json` exit1 was a shared target lock permission failure. It is retained and is not a product test result. The disclosed HarfRust dependency warning remains; no warning/advisory waiver was added.

## Findings (not fixed)

- Retirement `window not found`, native close HRESULTs and sporadic dispatcher errors remain unresolved. The diagnosis distinguishes source-backed candidate paths from runtime attribution. Startup cancellation, confirmed Swap and actual title-bar close require the planned V7 observations. An exit0 does not prove cleanup correctness; the V6 light run's unknown exit trigger remains excluded from normal-close acceptance. A lifetime/vendor repair is a subsequent decision after attribution, outside this frozen diagnostic review.
- Physical plug/unplug, reconnect/connector/latency and other unexecuted hardware items remain NOT_RUN. Previous V6 functional evidence remains version-scoped and applicable to unchanged behavior; it is not a fresh V7 native pass. Gate B remains INCOMPLETE and tasks08-11 remain dependency-gated.

Root retains exclusive desktop ownership. Use the V7 freeze and `RUST_LOG=info,ferrostage_ui::shell=debug` for attribution. The reviewer performed no UI action, native launch, Cargo command, commit, push, archive or journal update, and has no running command or owned native process.
