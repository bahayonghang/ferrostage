# Workbench implementation checkpoint

Date: 2026-10-18. Branch: `codex/workbench-settings-design`. Uncommitted sequential child after settings/font implementation. Independent review and integrated native acceptance remain coordinator-owned.

## Implemented

- Root DESIGN now replaces pill drawing tools and the separate session footer with one utility strip. Existing provenance, home composition, palette and settings/font rules remain.
- Header anchors the filename beside a quiet passive stage label. Utilities have quiet faces. Countdown remains the stronger 24px scaled mono role; wall clock becomes a muted 14px mono role. Progress no longer competes with filename in the header.
- A 56px session face with local 26px geometric play/stop icon and explicit label shares the navigation group with Previous/Next and compact page progress. Start/Stop retain existing IDs, Intents and shared deferred activation/accessibility/disabled/focus routing. A scaled minimum width keeps ordinary Start/Stop/preparing faces stable. Start checks both existing reason and busy state; local reason remains visible. The composition is present for document-open states even when workbench steps are empty.
- Compact localized page/step text appears beside an explicit Jump label; full logical/step/physical progress is the accessible name and pointer tooltip, plus keyboard-focus detail above the strip. Detail is absolutely positioned to avoid moving a target during activation. Explicit progress focus is a Tab stop. Typing still edits only; confirmation commits the existing jump.
- Blanking uses small bordered black/white swatches plus label and actual-state check. Drawing modes share a soft group with regular-radius checked selected faces. Clear remains semantic danger and separate from tool selection; Grid uses a secondary checked tool face, and Document retains its labeled utility. Group wrapping preserves control targets and keeps Start/Previous/Next together.
- No split ratios, image/raster/notes logic, core state machine, timers, media/ink semantics, audience projection ownership, dependencies or platform/vendor sources changed. Earlier settings/font changes are preserved.

## Checks

| Check | Result |
| --- | --- |
| `cargo test -p ferrostage-ui --locked` | PASS: 35 unit tests and doc tests, including shared palette contrast and locale parity. |
| `cargo test -p ferrostage-ui --features test-support --lib workbench --locked` | PASS: 12 tests. Real progress click, partial input does not navigate, Enter submits direct jump, private notes pages stay private, Tab reaches full progress detail without moving the activation target, busy Start cannot create a transaction or run the timer, document-open/no-step Start and in-place Stop remain available. Existing pointer, split/save, thumbnail scheduling/privacy and settings-to-workbench checks retained. |
| `cargo test -p ferrostage-ui --features test-support --lib presenter_ --locked` | PASS: 11 tests, including updated header/navigation co-location and notes priority, unchanged Idle home visibility, creation under controller lease, focus/chooser and presenter shutdown regressions. The existing header test retains its historical secondary size sample; no size/DPI matrix was added. |
| `cargo clippy -p ferrostage-ui --features test-support,pdfium --all-targets --locked -- -D warnings` | PASS on final source. Native GPUI/test production compilation, no waiver. |
| `cargo fmt --all` and `git diff --check` | PASS. |

Initial sandboxed Cargo access failed against the managed target cache; authorized cache access succeeded. The first native compilation identified one missing active-face ID and obsolete local, fixed before passing. Focused tests caught an explicit focus handle overriding Tab-stop eligibility and a detail-row reflow risk; the handle now opts into Tab and detail is an overlay. A test originally omitted the existing presenter activation-confirmation poll before checking Jump; it now follows the real coordination boundary. No runtime guard was weakened.

The final Grid-only styling change was type/lint checked after these tests; it changes its face from primary to checked secondary without changing dispatch. Integrated full suites remain coordinator-owned.

## Acceptance and remaining gaps

W1-W4 have source and focused input/layout evidence. W5 is pending native screenshot/interaction acceptance. Baseline source/native captures are coordinator-owned at parent `research/native/before-*.jpg`. This child did not launch or automate the desktop, build a new application executable, or claim screenshots/physical behavior verified. Coordinator must bind final screenshots and Start/Stop, navigation/jump, blank/tool, Ready/Presenting and Home checks to the rebuilt executable. Relevant light/dark and Chinese/English checks use an ordinary window only. No dimension, DPI or performance expansion.

`just ci`, `just check-shell`, final app build and independent Trellis review are intentionally reserved for the integrated checkpoint. No commit/push/archive or task-completed status performed. Displays can now consume this shared Presenter/theme foundation.

## Source checkpoint

SHA-256 values below identify this handoff's shared working-tree files, including preceding settings changes; later display/integration edits can change them. There is no new executable identity from this child.

| File | SHA-256 |
| --- | --- |
| `DESIGN.md` | `760d7fa68e5454ed048bfe79cb074ed4d38162cc1c9ec506fcda200ea2704dcc` |
| `crates/ferrostage-ui/src/presenter.rs` | `d7045acfa4dfc43f0977e0e15b72a8c38e63ad6df339e15e146dccfbfb8daada` |
| `crates/ferrostage-ui/src/presenter/workbench_view.rs` | `c6dd33cd197bf76a5f8819004d82c0f7d5372388e8574156a1214bd224f92f26` |
| `crates/ferrostage-ui/src/theme.rs` | `31ec48f36395cbbde2f4bc8d6a342f92e6b148b6998ad515c1e12bbdc69eefa9` |
| `crates/ferrostage-ui/src/shell.rs` | `d7624484ceccfe46ba8f7dc85d61720c9dbf0574f86e4828340b09e8882ab314` |
| `assets/locales/en-US/app.toml` | `02d62b2e52e130a298825ea766b227db68b9a50b8d412a18985e5083a377730d` |
| `assets/locales/zh-CN/app.toml` | `de70cc012acb8f2919ce03421d3e0b0d08700d55c2c2f2af2101597d1487a4c1` |


## Coordinated final acceptance

The implementation checkpoint above is historical. Independent review and actual native core/visual verification subsequently completed. See the parent [final verification](../10-18-workbench-settings-design/research/verification.md) for AC mapping, final source/executable identity, screenshots, 194 default and 136 GPUI checks, and exact limits. Font restart/default/fallback, full recent paths, workbench Start/Stop and compact display allocation/movement were observed. Native clipboard paste remains inconclusive, not PASS. No commit/push/archive.

