# Final implementation and acceptance, 2026-10-18

## Outcome

Both approved children were implemented sequentially through `trellis-implement`, independently reviewed/self-fixed through `trellis-check`, and checked in ordinary native windows by the coordinator. Product changes are complete and verified within the requested scope. No commit, push or automatic task archive was requested/performed.

Subsequent user instruction authorizes final review, Chinese commits, task archival and push. Final full-scope precommit check is READY with no new defects; see `precommit-review.md`. The user explicitly requested that the screenshot-embedded `interaction-preview.html` remain local via `.gitignore`, including its archived location; all other related task records remain included.

Page browser now has a real focused PDF-number editor, target logical/step context, one real-thumbnail selection path and independent1-4 column/row preferences. Layout & navigation is an upward contextual menu with exclusive/independent check semantics, immediate apply/close, no-op preservation and local failure recovery. Both retain the workbench and share safe focus/dismissal and stale-session rules.

## Final build and automatic checks

- Current EXE: `target/debug/ferrostage.exe`, 43,604,992 bytes, SHA256 `FBBE35BA06CE8E47BF16003CB6418330236197E2BA99AFF639C0868DDFCA8B53`. Native captures below used the preceding `35D8C54E...` build; the subsequent delta is confined to a test fixture, as documented below.
- `just ci` after the final product repair: PASS,198 default tests; formatting, strict Clippy and default cargo-deny all pass.
- Final `just check-shell` after test isolation: PASS, native GPUI/PDFium build, strict lint and159 GPUI tests. Receipt: layout-menu child `research/check-shell-isolated.log`.
- Additional strict UI Clippy with `gpui,pdfium,test-support`: PASS.
- Verified-DLL `native_perf100_grid_retention`: PASS,1 test, visible-thumbnail sequence `[4,0,16,0]`; closed/open/closed retained-frame ledger stays bounded. This is not CPU/FPS/RSS evidence.
- `cargo deny --all-features --locked --offline check`: PASS; `integration-deny-all-features.log`. Cargo manifests/lockfile, deny policy and maintained vendor source unchanged, so this receipt remains valid after UI/core-only repairs.
- Native source/runtime/PDF identities are recorded in `native/integration-final-identities.json`; closing the test processes and final rehash establish no source/PDF drift during native acceptance. Current reviewed identities are in `native/integration-final-reviewed-identities.json`. All22 entries match except the three-line test-fixture delta in `shell.rs` and the relinked EXE; reconstructing the prior test body exactly matches its native-snapshot source hash. No product/runtime/PDF input changed.

Detailed independent reviews: `../../10-18-page-browser/research/check-review.md`, `../../10-18-layout-navigation-popover/research/check-review.md`, and `integration-check.md`.

## Native acceptance on final combined build

Computer Use used the actual Windows application, real repository PDF fixtures and isolated settings under this task, on EXE SHA256 `35D8C54E16509067FBDA1AA2E3FA75D670583C85276D9B4280304C47A4E738E0`. Normal window only; no size/DPI matrix. Final dark/zh PID184500 and light/en PID103192 both closed through the app's normal titlebar route, with empty stderr and no remaining owned process.

| Capture under `native/` | Verified outcome |
| --- | --- |
| `integration-dark-layout-menu.jpg` | Upward menu above its trigger, live workbench retained, Auto and independent-notes checks, effective Right-layout helper, all rows visible |
| `integration-dark-unsplit-applied.jpg` | Selecting Unsplit closes immediately; real combined PDF preview appears and current-notes area becomes No notes |
| `integration-dark-unsplit-checked.jpg` | Enter reopens from restored trigger focus; Unsplit alone is selected |
| `integration-dark-physical-checked.jpg` | Physical navigation applied/closed, logical count changed50->100, reopened menu retains its independent check; viewport clamping works after toolbar wrapping |
| `integration-dark-auto-restored.jpg` | Selecting Auto closes and restores real right-half notes/crop without another confirmation screen |
| `integration-light-private-notes-rejected.jpg` | PDF3 is an independent notes page: local English refusal, disabled Go, Enter leaves panel/current physical1 unchanged, and thumbnails omit PDF3 |
| `integration-light-layout-menu.jpg` | English/light menu and wrapped helper text fit; new process restores session-default overrides rather than persisting prior dark-session choices |
| `integration-light-keyboard-toggle.jpg` | End focuses physical navigation; Space applies/closes, count3->8 and current red physical1 remains unchanged, proving no shortcut fallthrough |

Earlier page-browser acceptance is in `../../10-18-page-browser/research/native-acceptance.md`. Its final B7D001... checkpoint shows all16 thumbnails, instant2x3 selection with paired settings readback, outside dismissal, focus reopening and direct physical3 tile jump. Earlier ADC9... checkpoint separately proves edit69 without navigation and Enter to physical69/logical35. Those checkpoints are not relabeled as final-source runs. Final combined159-test suite retains those behaviors and the final native light browser confirms shared-host integration/notes refusal.

## Requirement acceptance map

| Parent AC | Evidence |
| --- | --- |
| A1 | Real retained workbench/menu/browser captures; mutual-exclusion GPUI regression |
| A2 | Real editor input/caret/selection/paste tests; physical69 target checkpoint; final native independent-note refusal |
| A3 | Normal-window16-tile fit regression/capture; actual2x3 change and2/6 config readback |
| A4 | Real input submit/tile/Enter/Space/hover/cancel tests; native explicit Enter/tile jumps and inert edits |
| A5 | Actual SettingsStore paired readback, unknown-leaf, legacy13/tall, save failure/Retry/read-only tests |
| A6 | Six-layout/toggle pointer+keyboard regressions; final live native menu apply/close/reopen/check evidence |
| A7 | Off-page annotation getter/no-op/exact-state preservation and induced failure recovery regressions |
| A8 | Presenting Escape/no-Stop, outside-click/drawing isolation, lifecycle and modal/repeated-Jump focus regressions; native outside/Escape/focus/Space observations |
| A9 | Real PDFium request/retention test plus stale exact-pixel failure/frame retirement regressions |
| A10 | Final default/native/lint/policy gates and named normal-window dark/zh + light/en native observations |

## Repairs caught by checks

Review repaired thumbnail image/caption overlap, truthful failed placeholders, duplicate selected checks, normal4x4 clipping/anchor bounds, and modal/repeated-Jump editor focus. Failing checkpoints and before/after test receipts remain in the owning child's research folder. One CI attempt was interrupted by the Windows executable lock while the coordinator's test app was running; the failed receipt is preserved and the complete gate subsequently passed after closure.

Final artifact review also caught a test-isolation defect: the new focus regression changed density through a working-directory dummy settings store. The checker added exactly three lines inside that cfg(test) regression to use the existing temporary SaveFixture, then removed the known generated `unused-headless-settings.toml`. Targeted regression1 PASS, full GPUI159 PASS, strict lint/build and fmt/diff PASS; the file remained absent after both test runs. No production path changed. The required build relinked the EXE, so the current hash is recorded separately; native smoke was not repeated for this test-only delta. The accidental artifact was removed from task relatedFiles and is not treated as a product identity input.

## Scope and limitations

No user PDF was written. No new renderer, UI framework, settings schema, dependency, native patch, override persistence or core clearing-policy change. The only core addition is the pure read-only all-page annotation-presence query. Root DESIGN and frontend executable contracts were synchronized.

Native UIA exposed no useful text tree, so rich screen-reader/AccessKit text-run/selection-action qualification is not claimed. Ordinary numeric editing and keyboard menu behavior have real-input regressions and the named native observations. Physical audience projection/hotplug, exhaustive cross-theme/locale permutations, size/DPI/performance and long-soak matrices were not run and are not implied by these results. Snapshot notes privacy, Presenting menu/Escape and projection data-flow remain covered by automated tests.

View-family review: checked3 / changed2 / left1. Changed page browser and layout menu; retained the base presenter composition. No placeholder product content is shipped; thumbnails and previews use the existing PDFium path. The HTML planning mockup remains historical illustration, not runtime acceptance evidence.
