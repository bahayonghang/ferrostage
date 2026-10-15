# v5 focused controls and coordinator consistency

Root reproduced focused Start receiving Space as global Next and ignoring Enter in v4 native evidence. The independent checker identified related modal editing, Open admission, Idle action, topology recovery and abandoned Jump-preview gaps. Root authorized these scoped07 repairs. Only actions.rs, presenter.rs and shell.rs change in product code. The separate DESIGN.md status change belongs to Root. Core, vendor sources, manifests, lock, policy, locale and rendering services are unchanged.

## Repairs

- **Focused controls:** GPUI dispatches matched keybinding actions before raw on_key_down. Buttons now use the local FerroStageButton context with ActivateButton for unmodified Enter/Space. The shared button action consumes these keys, invokes the existing deferred coordinator once when enabled, and consumes without action when disabled. Nonbutton Space still navigates. Mouse/accessibility routes remain the same.
- **Jump editing:** the modal's FerroStageJump context masks global bindings for digits and Backspace using the pinned public NoAction behavior, allowing the existing raw editor to receive them. This also covers a configured numeric navigation binding. Enter commits when editing; a focused modal button takes priority through its deeper button context. Escape cancels without falling through to Stop.
- **Open admission:** the same import_allowed predicate drives the view's Open availability and actual import. A rejected Open is marked when queued for focus, and admission is checked again when activation resolves. Presenting never enters the chooser. A refusal from Audience activates and confirms the current Presenter before publishing the existing localized stop-before-import message. No mid-presentation import is enabled.
- **Idle actions:** navigation and blank actions are no-ops before a document is loaded. This reproduces and fixes one source of raw English core NotReady errors; the original resumed native Idle message's trigger remains UNKNOWN. Stop already guarded Presenting and is not blamed for that observation.
- **Topology recovery:** an unsafe topology event remembers that recovery must be windowed. A surviving Presenter is retained only if its resolved MonitorId and current strict GPUI mapping agree, and its mode satisfies recovery. Replacement uses the existing actual-placement confirmation transaction. No last-known/unmapped geometry authorizes a window. Initial valid fullscreen startup is retained; unavailable/zero-screen startup waits for valid recovery. No new OS API, identity fallback or auto-Start is introduced.
- **Abandoned previews:** Stop and accepted Swap discard the Jump UI and pending preview in07 before publishing the transaction's Curtain/result. They do not emit an intermediate Live snapshot. The verified generic core Stop semantics remain unchanged. Audience close and unsafe topology use the same Stop path. Escape's existing preview cancellation is separately verified.

## Red / green evidence

All tests use the actual GPUI code paths, including key dispatch, Tab traversal, mouse input, WindowRecord creation/removal and the production close handler. They are headless TestPlatform tests and do not establish OS placement, GPU output or physical topology acceptance.

| Receipt | Actual result |
| --- | --- |
| product-v5-red.log | exit101,5 behavioral failures: focused Open navigates, modal Close ignores Space, Idle action reports NotReady, Presenting chooser opens, queued Open ignores stage change |
| product-v5-topology-red.log | exit101,3 recovery failures; preceding5 fixes pass |
| product-v5-jump-red.log | exit101, Backspace leaves text23 instead of2; other8 pass |
| product-v5-jump-numeric-red.log | exit101, configured2 navigation prevents entering digit2 |
| product-v5-preview-red.log | exit101, Stop and failed replacement Swap leave hidden previews; one temporary test-only unused-variable warning retained |
| product-v5-green.log | exit0,38 tests before the separately authorized preview repair |
| product-v5-green-final.log | exit0,41 tests with all repairs |
| product-v5-just-ci.log | exit0,145 default tests, fmt/strict lint/default policy gates |
| product-v5-clippy.log | exit0, app/UI all-features/all-targets strict lint |
| product-v5-build.log | exit0, native gpui,pdfium app build |
| product-v5-diff-check.log | exit0 |

Commands are recorded before execution in product-status.txt. Final headless command: `cargo test -p ferrostage-ui --features test-support --lib --locked --offline`. Lint: `cargo clippy -p ferrostage-app -p ferrostage-ui --all-features --all-targets --locked --offline -- -D warnings`. Build: `cargo build -p ferrostage-app --features gpui,pdfium --locked --offline`. No command was left running at freeze.

Keyboard coverage includes actual Tab-focused Open and Stop, modal Close, keyup without duplicate activation, disabled Start traversal/click and a focused Open becoming disabled, nonbutton navigation, default Backspace, configured numeric navigation, Jump Enter/Escape, refusal owner confirmation and focus-time stage change. Headless Start remains disabled because the test has no valid topology watcher; Root owns native Start keyboard verification. No fake hardware readiness is introduced.

Topology tests use real Changed/Unavailable events and real GPUI windows. TestDisplay has a random UUID and no GDI connection; the test first verifies that unmapped input cannot place a replacement, then explicitly supplies the separately tested mapper's owned-result shape to exercise the recovery transaction. This tests coordinator recovery, not the physical UUID bridge. Recovered mode/current handle, no auto-Start, Curtain and preview cancellation are asserted. Initial fullscreen and stale GPUI identity are separate cases. Physical unplug/port-change cases remain NOT_RUN.

The unchanged full dependency graph retains product-test-support-deny.log's all-features audit and the existing core tree/vendor regression evidence. HarfRust's documented upstream direct-lint limitation remains; there is no new exception or suppression. Shared component/design contracts are maintained by Root. All original route, maintenance and native failure receipts remain intact.

## Frozen handoff

product-native-freeze-v5.json SHA256 `7cc8e5523c077c5d991aef6bbc4797a561022512b4fd67f66e8c24aeb813a442` binds602 source files, aggregate `39907c8b4fdb0221ede20be1889b9ca20d6768a3794652479a7c451c4b1aa8f0`. Executable SHA256 `3516cca7cf432cf1e4dace74f1a065c19509d10be1c4bb54f93a7b9e0c382fa8`,36,981,248 bytes. The lock remains `2557e10b37fa5cfe63e78ce19965b8bbbd83438bdd540eed50405fbbc3dbc23e`. product-v5-source-delta.json verifies598/602 unchanged inputs (three implementation files plus Root's DESIGN.md changed), and every core/vendor input unchanged. Focused patch SHA256 `7a10ccdfced02e42b842427a129cb3b298d07494299ac932bb29ee8f4e0d04c0`.

The binary remains `D:/mbx-cache/targets/v1/b228c6f37bcb27af7d1a2d6821dcfcb308164a0bd33ec7c00848c486151f07e0/debug/ferrostage.exe`. Launch with Root's isolated --config, exact --pdfium-library and optional PDF. Root was notified immediately after freeze; native operations remain exclusively Root's. Prior native PASS observations are source-scoped and do not automatically verify these new fixes. Gate B remains incomplete until its required acceptance work is performed. No commit, push, archive, journal, task switch, installation or publication occurred.
