# Native startup entity lease repair

Root's retry ran while its launch host waited and exited101 with `cannot read ferrostage_ui::shell::Shell while it is already being updated` at GPUI entity_map.rs156. Native stdout contains the complete successful three-display UUID mapping before the panic. The initial freeze and both native launch receipts are retained under Root's native directory. The earlier implementation freeze did not constitute native acceptance.

Pinned GPUI App::open_window invokes window.draw before returning (vendor/gpui/src/app.rs). Shell::open executes while Shell is leased for update; the original Presenter::render synchronously read that entity. This was the direct re-entry cause. The real GPUI TestAppContext/TestPlatform regression invokes the actual Shell::open path under the same lease and reproduces exactly the same panic. Its first attempt failed only because bit-set was absent from the offline cache; the subsequent compiled regression is the actual red test, not the cache failure.

The correction gives Presenter15 drawing fields: theme, stage, filename, modal, error, warnings, start reason, loading/busy/dirty, save error, page count/current/preview, recent filenames/paths for presenter actions. Initial data is constructed directly from the already-borrowed Shell before opening the window. Entity notifications refresh it after the controller lease ends. Render has no controller read/update; actions still route through WeakEntity<Shell>. No full PresentationState, SettingsSession, document, worker or deck is cloned. Audience is unchanged and remains snapshot-only.

The headless regression covers synchronous first draw, subsequent document/path notification refresh and replacement-window first draw. All19 UI tests pass. This establishes the entity-borrow repair on the real GPUI code path; native focus/display/GPU behavior remains for Root's v2 launch.

A nondefault UI test-support feature forwards the existing GPUI test-support feature and uses TestPlatform, which never creates an OS window. It adds57 locked upstream test-graph packages. Default UI still does not link GPUI; normal native app builds use gpui,pdfium without test-support. No alternative package alias or new maintained source was added. Windows CI and just check-shell run the headless regression. Full all-features deny passes with the unchanged policy and exactly the previously approved three license exceptions; there is no extra source/license/advisory waiver.

| Command / receipt | Result |
| --- | --- |
| Root native run-a-light-retry | exit101, actual startup panic |
| product-startup-red.log | exit101, missing offline bit-set; no assertion ran |
| product-startup-red-retry.log | exit101, one actual TestPlatform regression reproduces the native panic |
| cargo test -p ferrostage-ui --features test-support --lib --locked --offline | exit0,19 tests; product-startup-green.log |
| cargo deny --all-features --locked check | exit0, all four gates; product-test-support-deny.log |
| just ci | exit0,145 tests; product-v2-just-ci.log |
| cargo clippy -p ferrostage-app -p ferrostage-ui --all-features --all-targets --locked --offline -- -D warnings | exit0; product-v2-clippy.log |
| cargo build -p ferrostage-app --features gpui,pdfium --locked --offline | exit0; product-v2-build.log |
| git diff --check / build-process inventory | exit0 / none; product-v2-diff-check.log and product-v2-processes.json |

HarfRust's reproduced upstream direct lint limitation remains unchanged; no vendor runtime file changed. All original route/maintenance failures and product v1 evidence are retained. product-startup-delta.json identifies the six changed compilation-context files between v1/v2; product-startup-repair.patch is the focused source repair after the red test.

The new freeze is product-native-freeze-v2.json. Source snapshot602 files: `62595ac6b2fbaae3fe98053699a29cb33fabbfc44f45d86ef1dfbcce29f19f89`. Executable: `b96cb72ea54ddacacd9491090a5f92ef8950f7fdf720758c2d5f2dc6986bb396`. Lock: `2557e10b37fa5cfe63e78ce19965b8bbbd83438bdd540eed50405fbbc3dbc23e`. Root was given the unchanged resolved executable path and launch argv; the implementer has frozen all product code/builds and only writes research handoff artifacts during Root's native verification. Physical unplug/port changes and unavailable hardware modes remain NOT_RUN; Gate B is incomplete.
