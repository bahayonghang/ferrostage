# V8 retirement repair and shared display assignment

2026-10-15 America/New_York. Implementation and automated gates PASS; independent review and Root native acceptance are next. This report does not establish Gate B or Windows cleanup acceptance. All edits remain uncommitted.

## Scope and source

- Existing task07 and the independently reviewed display-assignment plan were resumed. The V7 source, original V6/V7 receipts and all previous freezes were preserved. Root owns parent/spec/ADR updates and all desktop operations.
- `display_settings.rs` owns `DisplayDraft`, `PdfTarget`, the shared `numbered` mapping and the reusable panel. The toolbar now opens Displays; Settings remains the explicit task09 placeholder. `Swap` keybindings continue through the same role coordinator.
- The panel shows count, number/name, native resolution, primary/current use, identity confidence and the GDI scale provenance. Numbers sort current MonitorIds and are not described as Windows display numbers. Choice buttons provide separately labeled application/PDF destinations, explicit selected text, Apply/Cancel, Detect again and Identify.
- `DisplayDraft::settings` validates current revision and unique available IDs, rejects same-display conflicts, and commits all roles plus the existing single-monitor mode together. No-play and physical PDF targets use `PresenterOnly`; only explicit single-display windowed playback uses `WindowedAudience`. Unselected connected monitors have explicit None bindings; old offline roles are removed by an explicit Apply.
- Shell probes fresh topology on open, Apply and Identify, consumes prior watcher deliveries before the probe, and invalidates drafts on changed/unavailable topology. Unchanged complete snapshots do not invalidate a draft or churn pending windows. Zero displays and unavailable enumeration remain distinct. Future-version settings cannot edit or apply.
- `Shell::assign_displays` is the single replacement/confirmation/rollback/commit/save path used by Apply and Swap. Runtime assignment is committed only after actual placement confirmation. Valid migration preserves the core state/page/blank/running timer. Explicit no-play waits for successful placement, then publishes Curtain, stops core, closes the audience and leaves Ready with the same page and frozen timer. Failed creation/placement keeps the old complete session and settings. Failed persistence retains runtime assignment with the existing single unsaved/Retry owner.
- `Identify` has only number/name fields. Its nonfocused popup windows use the same numbering and strict mapping, expire after three seconds, and close on cancel, topology changes, Start/Stop or assignment. Identify is refused while Presenting. It receives no document, PDF path, notes, settings controller or projection snapshot.
- Panel-open, assignment-preparing and transaction-commit diagnostics include stage/page/elapsed_ms for Root's native continuity/freeze observations. Existing V7 file/line and actual bounds/viewport/scale diagnostics remain.

## Regression-backed retirement repair

The exact confirmed V7 callback was exercised with real GPUI TestPlatform windows and the foreground executor. `product-v8-retirement-red-activation.log` records two `vendor/gpui/src/window.rs:1914: window not found` errors after an activated window was retired. The close-request regression also failed because the audience remained after `simulate_close`.

The nine-line maintained GPUI delta gives Window an owning `Rc<()>` token and the active-status callback a Weak token. An already-dropped owner cancels that callback before accessing the removed handle. The live handler retains its original update and `log_err`; there is no error-text filter, catch, logger suppression, or new platform package. The first-party should-close hook returns false and queues removal of the original handle on the foreground executor after the platform callback returns. Existing deferred current/pending/retired close classification and explicit quit remain authoritative.

The initial green retirement pair passed. The final suite additionally covers pending close, a queued close consumed after old-window replacement, and current-presenter close while Presenting. Those exercise actual GPUI removal and coordinator behavior; they do not emulate Windows RevokeDragDrop/DestroyWindow or prove absence of the separately observed dispatcher error. Root must retest native logs.

## Checks and evidence

| Check | Receipt | Result |
| --- | --- | --- |
| Default workspace fmt, warnings-denied clippy, tests and deny | `product-v8-just-ci.json/log` | PASS, 148 tests |
| UI test-support suite | `product-v8-headless-final.json/log` | PASS, 62 total: 21 pure + 41 actual GPUI tests |
| Actual app/UI all-feature/all-target clippy `-D warnings` | `product-v8-clippy-final.json/log` | PASS |
| Full unchanged all-feature locked/offline dependency policy | `product-v8-deny.json/log` | PASS: advisories, bans, licenses, sources |
| Native app `gpui,pdfium` locked/offline build | `product-v8-build.json/log` | PASS |

New tests comprise three pure draft/numbering/mode tests, three real-GPUI retirement tests and seven real-GPUI display tests. Display coverage includes mouse draft/cancel with unchanged file bytes and windows, future read-only mouse/keyboard/Apply refusal, revision/zero/unavailable states, valid migration + actual failed-save/retry/read-back, no-play stopping/freezing, creation/placement rollback, and actual Identify window content/expiry/Presenting refusal. The TestPlatform has a random UUID and no OS/GDI connection: successful transaction tests explicitly supply the existing owned mapping result, while the production fresh enumeration/mapping remains native acceptance. Single-screen fixtures do not represent a physically single-screen machine.

The earlier failures are retained, not rewritten: first sandbox build-lock denial; test logger writer compile failure; first activation harness lacked explicit activation; the two actual red lifecycle failures; first full suite's three older fixture/toolbar assumptions; one new test field-name compile failure; disabled-control test setup corrections; strict clippy's test-only PdfTarget import. The toolbar test now asserts the Displays activation intent. Topology-loss and pending-cancellation fixtures now start with populated/changed topology while retaining every original safety assertion. One corrected test-only import followed the final feature suite; the exact final code compiled under strict all-target clippy and native build. No threshold or assertion was weakened to pass.

The unchanged upstream HarfRust `shape` dead-code warning remains in dependency build output; strict first-party consumer lint passes without waivers. Cargo.lock adds only the existing tracing-subscriber edge for real regression log capture; no package/version or policy change.

## Handoff

`product-native-freeze-v8.json` binds current source/contract files, executable, PDFium, raster fixture, five Root-created isolated configs and successful check receipts. `product-v8-source-delta.json` compares V7. `product-v8-evidence-manifest.json` hashes the new implementation receipts and read-backs. Use the V8 freeze with Root's prepared native runner; old config-driven or Swap screenshots cannot establish the new UI.

Remaining: serial independent check; Root native close/retirement logs, three-display list/Identify/roles/Apply/Cancel/restart, migration/no-play timer observations, actual save lock/Retry, future read-only and visual/focus/theme/size checks. Physical plug/reconnect/port operations and real single/zero display remain NOT_RUN. Task07/Gate B are incomplete; 08–11 remain dependent. No desktop launch, user-config write, commit, push, archive or add_session occurred in this implementation agent.
