# Task09 implementation report — 2026-10-15

Status: all U-1–U-11 implementation is ready for independent review. No commit, push, archive or foreground desktop interaction. Original composite native ACs remain unchecked. User-deferred desktop smoke, physical hotplug/latency, real single/zero-screen, exhaustive native/size/DPI/theme and long performance/VRAM matrices remain DEFERRED_BY_USER / NOT_RUN.

## Product changes and consumers

- `ui/settings_panel.rs`: seven-category field/edit model; the existing settings parser remains the validation/range authority. Invalid input and coupled timer-bound fallback are rejected without adopting parser defaults. Monitor source occurrences are retained; recent-limit trimming uses the schema behavior.
- `ui/presenter/settings_view.rs`: General, Displays, Presenter, Timer, Pointer, Shortcuts, Advanced. Wide category rail / narrow wrapping categories, scrollable content, fixed feedback/Close footer; shared accessible focus/button tokens. Real numeric typing, increment/decrement, mouse/keyboard sliders, RRGGBBAA editing, live color/radius/pen/highlighter previews, switches and enum choices. Save failure shows dirty/retry. Future configuration disables editing, recording, clearing, directory action and reset. Reset requires a second explicit confirmation.
- `ui/shell/preferences.rs`: validate → runtime transaction → SettingsSession edit → atomic save. Pending worker/placement work serializes writes, saves, imports and swaps. CPU/L3 changes are acknowledged before persistence. Checked laser style is applied to a cloned core state before committing settings; Idle validates using a one-page state and applies the actual style on the existing loaded-state consumer. Failed runtime work preserves the previous configuration; failed saving retains the effective session values and dirty retry.
- `ui/actions.rs`: action handlers/interceptor are registered once. `replace_bindings` clears the complete GPUI keymap and rebuilds application, Escape, button, Jump and slider safety bindings. The interceptor suppresses action dispatch only for the active presenter's settings editor/recorder, leaving its raw key event to the view. Conflicting normalized bindings are disabled on both actions and marked inline; old bindings are actually removed. Settings modal protects presentation actions, while prior Jump Stop/Swap behavior remains intact.
- `display_settings.rs`, `shell.rs`: the complete settings page shares the existing numbering, DisplayDraft, Apply/Cancel, Identify and confirmed-window migration. Detailed mode adds proportional physical virtual-desktop rectangles (including negative origins/gaps), OS primary marker, actual identity source/confidence, GDI diagnostic scale provenance, resolved role source and warnings. The task07 minimal toolbar panel retains its compact layout. Topology changes preserve the full Settings page and invalidate the old draft. Presenter startup-fullscreen is explicitly next-start; audience fullscreen/roles use the existing migration transaction.
- General language saves the next-start catalog choice; current Shell locale remains unchanged. Theme updates the existing dark/light/system palette and existing appearance subscription. Reopen-last-file uses the existing startup consumer; recent limit/clear immediately update the list. Presenter ratios, notes scale, clock, strip and independent grid columns/pages reach WorkbenchData; timer mode/duration/thresholds do not restart elapsed time. Pointer colors/width/alpha reach existing drawing consumers; laser enters Audience only via checked core snapshot style. Debug toggles actual workbench diagnostics.
- `render/{lib,cache,protocol,worker,worker/engine}.rs`: `RenderHandle::set_cache_budgets(CacheBudgets) -> Reply<CacheBudgetAck>` changes L2/L3 limits and evicts excess entries on the existing worker; acknowledgement contains effective budgets and used bytes. No worker restart, PDF write or new PDFium thread.
- `audience.rs`, `presenter.rs`, `texture_cache.rs`: budget preflight/commit adapters to the existing08 upload transaction. Audience still imports no Settings, controller, path, document or cache handle from the shell. The only incoming budget remains `snapshot.visual.texture_budget_bytes`.
- Both locale catalogs contain the complete labels/feedback. Hot reload is explicitly unavailable pending10; no placeholder media controls are rendered before11. No new dependencies or waivers were added by09.

## GPU / worker transaction contract for09/10 consumers

Schema minimum remains 1 MiB for each cache field. Total L1 is divided equally between Presenter and Audience; each cache reserves half its window allocation for committed images and half for a pending replacement. A proposed reduction is accepted only when all retained committed bytes fit its new committed half. The refusal preserves effective budgets, complete fallback pixels/identity and Settings; it reports a localized error rather than saving a value unused by a window.

Call path: `Shell::apply_preferences` → `preflight_gpu` → `Presenter::accepts_texture_budget` and `Audience::accepts_texture_budget`. After worker ack (if needed), preflight runs again at commit, then `Presenter::apply_texture_budget` / existing snapshot scalar update / `Audience::commit_snapshot_budget`. Audience applies only its snapshot scalar. If `visual=None` and Audience retains an image, any L1 change is rejected; unrelated settings still apply. An empty, unallocated Audience requires no mutation and acquires the effective budget from its first authorized visual. With no Audience only the active Presenter is checked, and a later Audience uses the persisted effective setting.

If a subsequent display/placement step fails, acknowledged CPU budgets are restored on the same worker before releasing the transaction. Failed rollback is reported explicitly and never persisted as success. Window cancellation/timeout retires only replacement windows and restores the settings page; successful role/no-play changes retain the existing07 page/timer/blank semantics. A GPU upload failure itself remains08's staged/committed transaction: same pixel identity does not spin; new pixels/document reopen or an accepted changed budget can retry.

## Verification receipts

Final successful checks (all exit0 unless noted):

| Command | Result |
| --- | --- |
| `cargo check -p ferrostage-ui --features gpui,pdfium,test-support` | PASS |
| `cargo test -p ferrostage-ui --features gpui,pdfium,test-support --lib --quiet` with pinned FERROSTAGE_TEST_PDFIUM | 107 passed: 106 headless/pure GPUI tests plus real PDFium initial-load regression |
| `cargo test -p ferrostage-render --lib` | 17 passed |
| `cargo test -p ferrostage-pdf -p ferrostage-render -p ferrostage-document --features pdfium --all-targets --locked --quiet` | 37 passed across test binaries, including actual native PDFium pixel/cache/generation tests |
| `cargo clippy -p ferrostage-ui -p ferrostage-app -p ferrostage-render --features gpui,pdfium,test-support --all-targets --locked -- -D warnings` | PASS |
| `cargo clippy -p ferrostage-app -p ferrostage-ui -p ferrostage-render -p ferrostage-document -p ferrostage-pdf --features gpui,pdfium --all-targets --locked -- -D warnings` | PASS without test-support |
| `just ci` | PASS: fmt, strict default clippy, 162 tests, cargo deny; raw output in just-ci.log |
| `cargo build -p ferrostage-app --features gpui,pdfium --locked` after default CI | PASS; feature-build.log and build-identity.json identify binary |
| `git -c core.fsmonitor=false diff --check` | PASS |

Pinned native fixture library: `cache/pdfium-7881-win-x64/bin/pdfium.dll`. The normal, already existing vendor `harfrust::shape` dependency dead-code warning remains; workspace strict lint passes without a waiver. Initial unprivileged compilation could not write the configured shared mbx cache (exit1); the ordinary elevated Cargo invocation succeeded. Intermediate implementation iterations failed compilation or targeted tests while wiring controls; those are not claimed as PASS. Resolved cases included GPUI accessibility IDs, correct raw-key interception before mapped actions, slider mouse focus and persistent drag state, Idle checked-style validation, compact07 panel versus detailed09 layout, and test fixtures using actual display confirmation / elapsed-time freeze contracts.

New regression names:

- `settings_v09_edit_save_retry_readonly_and_runtime_consumers`
- `settings_v09_record_replaces_keymap_and_capture_precedes_actions`
- `settings_v09_numeric_editor_and_slider_use_real_keyboard_events`
- `settings_v09_all_categories_render_and_footer_stays_reachable`
- `settings_v09_dual_gpu_preflight_never_applies_half_a_transaction`
- `settings_v09_worker_ack_serializes_changes_and_failed_display_rolls_budget_back`
- `settings_v09_shared_display_apply_cancel_timeout_and_topology`
- `settings_v09_values_reach_workbench_timer_layout_pointer_clock_and_diagnostics`
- `field_edits_reject_fallbacks_and_keep_unrelated_values`
- `recent_limit_trims_and_coupled_timer_validation_is_atomic`
- `topology_diagram_preserves_negative_coordinates_and_physical_aspect`
- `acknowledged_budget_shrink_evicts_both_caches_without_reloading_active_document`

The existing native `native_golden_and_transaction` additionally verifies actual PDFium cache hit → acknowledged eviction → uncached byte-identical rendering on the same active generation. Existing07/08 regressions remain. Two07 tests adapt only initial focus traversal from first Tab to last Shift+Tab because the former one-button Settings placeholder now has complete categories: `v5_modal_close_button_has_priority_over_global_enter_and_space` and `v6_removed_modal_button_restores_keyboard_traversal`. Their keyboard Close, page/stage protection and focus-recovery assertions are retained. No assertion was relaxed.

## Review / remaining evidence

I-1–I-5 complete; I-7 implementer automatic gates complete. Independent reviewer owns its audit and any post-fix rebuild. I-6 and original composite native UI ACs remain deferred under the user instruction; headless GPUI input/layout checks do not claim native screenshots or physical matrix coverage. Main thread owns consolidated native work and progression into10/11. All08/09 source changes remain uncommitted and unarchived for review.

Final post-CI native executable: `D:\Documents\Code\Rust\Exp\ferrostage\target\debug\ferrostage.exe`, 41526784 bytes, UTC 2026-10-15T13:56:39.0122744Z, SHA-256 `91F8604B1B8F08B692009DEDFD7B235BFAE73980BB807ECB70932DF0FA2DDF03`. Target resolves through the existing mbx link recorded in build-identity.json. No process was launched.
