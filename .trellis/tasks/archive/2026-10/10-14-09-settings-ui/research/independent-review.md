# Task09 independent review — 2026-10-15

Decision: U-1–U-11 implementation and independent automatic gates PASS after the local transaction fix below. No unresolved implementation blocker found. Native desktop functional smoke remains NOT_RUN in this review, explicitly scheduled by the user for the main thread after09→10→11. Physical, exhaustive native/size/DPI/theme, latency and long performance/VRAM matrices remain DEFERRED_BY_USER / NOT_RUN. This decision does not mark the original composite native acceptance checkboxes passed.

Reviewed against09 PRD/design/implementation, parent acceptance-policy/execution-state/shared design, DESIGN.md and applicable frontend/backend specs. The latest policy supersedes historical Gate B blockers. Preserved08 and other agents' changes. All changes remain uncommitted and unarchived.

## Findings (fixed)

- `crates/ferrostage-ui/src/shell.rs`, `presenter/workbench_view.rs`: pending settings changes wait for a CPU-cache worker acknowledgement or confirmed display migration. Closing the Settings modal did not clear this transaction, but08's Split handler checked only WritePolicy. A user could close Settings and drag a divider, save the changed layout, then have it overwritten by the older pending Settings snapshot. Split now requires `!busy()` as well as Writable; the view also declines starting a drag while busy. Existing completion paths restore dragging automatically. This covers both forward application and acknowledged rollback without changing document/navigation behavior.
- `shell/preferences_tests.rs`: extended the worker-ack test to assert direct Split intents cannot change or save settings while pending, and work again after acknowledgement. Added `settings_v09_closed_panel_split_waits_for_apply_and_rollback_ack`: actual headless GPUI Escape closes Settings, mouse-down/move/outside-up attempts both before apply acknowledgement and while rollback is awaiting acknowledgement leave layout and exact saved file bytes unchanged; after completion the same mouse sequence changes and persists the ratio. The controller guard is also exercised directly during rollback.
- Root synchronized the runtime transaction contract in `.trellis/spec/frontend/state-management.md`; reviewer checked that the closed-modal and rollback serialization rules are recorded and the08 contracts retained.

## Requirement and boundary review

| Scope | Actual evidence / outcome |
| --- | --- |
| U-1 categories | `settings_panel.rs` has seven concrete categories and validated field definitions; `presenter/settings_view.rs` renders actual controls, category switching, fixed footer, feedback and scrollable content. Seven-category headless input/layout regression passes. |
| U-2 display details | Shared `DisplayDraft`/Identify/Apply/Cancel path is reused. `display_settings.rs` draws physical virtual-desktop geometry including negative origins/gaps, and shows actual identity/confidence/scale provenance and role-resolution warnings. Topology invalidates stale drafts; compact07 panel remains. |
| U-3 General | Language persists next-start catalog while live locale remains stable; existing theme/system appearance consumer updates; reopen/recent settings reach startup and recent list. Parser trims recent files on limit changes, Clear persists empty list. |
| U-4 Presenter | Ratios, notes scale, clock, thumbnails and independent grid columns/visible pages reach WorkbenchData and actual render geometry. Existing08 wheel/split/notes/privacy tests retained; pending settings writes now serialize divider changes too. |
| U-5 Timer | Mode/target/thresholds pass the existing parser's coupled validation; actual warning consumer changes without resetting elapsed core time. |
| U-6 Pointer | RRGGBBAA editing is converted to BGRA, live samples use actual values, and pen/alpha reach drawing consumers. Laser style is validated through checked core commands before settings commit and reaches Audience through its pure snapshot only. |
| U-7 Shortcuts | Raw key interception is restricted to active Presenter's settings editor/recorder, suppresses mapped actions while preserving the raw event, and validates canonical bindings. Full keymap clear/reinstall removes stale keys and restores modal/slider safety bindings. Conflict diagnostics disable both conflicting actions; reset restores defaults. Real keyboard regressions pass. |
| U-8 Advanced | CPU/L3 budgets use the same existing render worker, with immediate eviction and typed acknowledgement. L1 uses both-window admission checks. Debug has a real workbench consumer; settings-directory and reset actions honor writable policy. Hot reload is explicitly unavailable pending10; media placeholders are absent. No new09 dependencies/waivers. |
| U-9 transaction/read-only | Validate → runtime → SettingsSession edit → atomic save; failed save leaves effective values dirty with retry, future schema blocks mutations/reset. Asynchronous budget work commits only after acknowledgement; failures preserve settings and roll CPU budgets back before releasing busy state. |
| U-10 migration | Settings embeds the existing confirmed-window role transaction. Timeout/cancel leaves original windows and settings; successful migration preserves page/timer/blank. No-play releases Audience and freezes Ready timer. CPU acknowledgement plus later placement failure exercises acknowledged rollback. |
| U-11 design/accessibility | Existing palette/font/focus/button tokens are reused. Numeric typing and mouse/keyboard slider interaction, disabled/read-only/dirty feedback and reachable Close footer at wide/narrow headless sizes are tested. Native screenshots, DPI/theme matrix and physical monitor behavior are not claimed. |

GPU review: L1 preflight checks both retained Presenter/Audience images, rechecks after asynchronous acknowledgement, then applies budgets on the serialized UI thread before committing Settings. Invalid reductions preserve old effective limits and complete fallback frames. A retained Audience without an authorized visual rejects L1 changes; unrelated preferences still apply. Audience receives only `AudienceProjectionSnapshot`, including its approved scalar texture budget, and emits pure render status; no Settings/path/controller capability was introduced.08's candidate/commit-on-success, pending CPU fallback, overlay identity, Curtain/blank precedence, retirement accounting and same-pixel upload-failure suppression remain covered by the full retained suite.

The render budget protocol remains on the existing single PDFium worker. Native regression verifies cache hit → acknowledged eviction → uncached byte-identical raster on the same active generation. User PDFs remain read-only; settings persistence retains the existing atomic sibling-temp-file mechanism. No vendor/backend licensing policy relaxation, unsafe code, extra PDFium thread or notes leak was introduced by the review fix.

Inspected the two07 focus adjustments directly: `v5_modal_close_button_has_priority_over_global_enter_and_space` and `v6_removed_modal_button_restores_keyboard_traversal` use Shift+Tab to focus the final Close control in the expanded Settings panel. Close behavior, unchanged page/session, subsequent Tab traversal and action-availability assertions remain. No coverage weakening found.

## Verification

All final commands exited0; receipts are sibling `review-*.log` files.

| Command | Result / receipt |
| --- | --- |
| `cargo test -p ferrostage-ui --features gpui,pdfium,test-support --lib settings_v09_ --locked --offline` | 9 passed; review-focused.log |
| `cargo test -p ferrostage-ui --features gpui,pdfium,test-support --lib --locked --offline` | 108 passed, including actual PDFium initial-load regression; review-ui-tests.log |
| `cargo check -p ferrostage-app -p ferrostage-ui -p ferrostage-render --features gpui,pdfium,test-support --all-targets --locked --offline` | PASS; review-feature-check.log |
| `cargo clippy -p ferrostage-app -p ferrostage-ui -p ferrostage-render --features gpui,pdfium,test-support --all-targets --locked --offline -- -D warnings` | PASS; review-feature-clippy.log |
| `cargo clippy -p ferrostage-app -p ferrostage-ui -p ferrostage-render -p ferrostage-document -p ferrostage-pdf --features gpui,pdfium --all-targets --locked --offline -- -D warnings` | PASS production configuration; review-production-clippy.log |
| `cargo test -p ferrostage-pdf -p ferrostage-render -p ferrostage-document --features pdfium --all-targets --locked --offline` | 37 passed; review-native-tests.log |
| `just ci` | PASS fmt, strict default lint,162 tests and deny; review-just-ci.log |
| `cargo deny --all-features --locked --offline check` | PASS advisories/bans/licenses/sources; review-all-features-deny.log |
| `cargo build -p ferrostage-app --features gpui,pdfium --locked --offline` | PASS after default CI; review-feature-build.log |
| `git -c core.fsmonitor=false diff --check` | PASS |

Native DLL checks set `FERROSTAGE_TEST_PDFIUM` to the pinned `cache/pdfium-7881-win-x64/bin/pdfium.dll`. No desktop process was launched or native focus/input manipulated. Headless GPUI simulations are not desktop evidence. The preexisting dependency warning in vendor `harfrust::shape` remains; strict product lint passes without adding any waiver. An intermediate new-test compile attempt accessed a private rollback field and failed; it was corrected to assert observable pending/busy behavior before the final successful runs.

## Findings (not fixed) / handoff

No known in-scope implementation issue remains. Main thread owns the user-scheduled consolidated native functional smoke after11, including actual09 settings consumers, save/retry/read-only state, shared display migration and screenshot review. Deferred matrices remain separate and do not block10 implementation.

Final executable: `D:\Documents\Code\Rust\Exp\ferrostage\target\debug\ferrostage.exe`, **41,526,784 bytes**, UTC **2026-10-15T14:11:34.7739001Z**, SHA-256 **1556C464DD3DB2103FA1DEDC9126AE17928407BDC5DB7251FFCD95EBF548DED7**. This supersedes implementer's91F860… binary and was rebuilt after CI. `review-build-identity.json` records the exact command/identity; no launch performed. Source ownership released to root/next serial task. No commands remain running.
