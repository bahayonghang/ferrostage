# Native-feedback repairs

These are scoped follow-ups to `integration-review.md`, requested by the coordinator after inspecting the actual running ordinary-window UI. The reviewer edits source and runs library tests only while the first-checkpoint app remains running. The coordinator owns screenshots/input and will send an app-closed signal before application relinking. No size/DPI/performance or physical-hotplug test matrix is added.

## F3: duplicate progress detail on focus plus hover

Native observation: after Jump confirmation, focus returns to progress while the pointer remains over it. The absolute focus detail and hover tooltip repeat identical text.

Fix: attach the progress hover tooltip only while progress is not focused. Keyboard focus keeps the existing absolute detail, and pointer-only hover keeps the existing tooltip. The activation target and full accessible name are unchanged. Added a debug selector to the shared tooltip solely so the real rendered tooltip can be observed in the existing input regression.

Proof: `workbench_controls_jump_directly_and_keep_note_pages_private` now shows the actual hover-only tooltip, reaches progress by Tab, moves the pointer over the focused control, advances the real GPUI tooltip timer, and requires exactly the focus-detail surface with unchanged target bounds. It then retains the original Jump/edit/confirm and private-note assertions. Before fix: failed `focused progress already presents full detail; hovering must not duplicate it`. After fix: PASS, 1 test.

## F4: settings helper text crossed the row hairline

Native observation: Language and Reopen startup helper lines were clipped at their row bottom, despite later content having space.

Cause and fix: the settings content flex column shrank the natural-height setting rows toward their 56px minimum. Add `flex_shrink_0()` to the shared setting row so its actual label/help/control content determines height; the existing single content scroll owns overflow. Do not increase a fixed height or reduce type size. Stable row/help debug selectors let a normal-window test compare the rendered helper and owning row.

Proof: new `settings_general_help_is_contained_by_its_content_driven_row` uses the existing ordinary-window fixture and verifies Language/Reopen helper bounds are inside their row. Before fix: Language help origin y217, height19.5, bottom236.5 while its row origin y175, height56, bottom231, a real 5.5px overflow. After fix: PASS, 1 test. No extra window-size case was introduced.

## F5: font-filter paste became a literal v

Native observation: text-entry automation using the standard clipboard/paste path produced filter `v` instead of the supplied font name. The capture layer forwarded Ctrl+V plus its printable fallback, and the new filter treated all text fallbacks as typing.

Fix: only the font-picker key branch handles Ctrl+V/Cmd+V/Shift+Insert with GPUI clipboard text. Pasted control characters are rejected, insertion observes the existing 128-byte bound, and other Control/Alt/platform/function combinations do not append their letters. Ordinary typing, Shift-modified text, filter navigation and Escape/default/commit remain. Keybinding recording and other numeric editors are unchanged. Both locale help lines mention paste.

Proof: extended real-keyboard font regression writes `Arial` to the test platform clipboard and sends Ctrl+V. Before fix: actual `v`, expected `Arial`. After fix: PASS. It also rejects a multiline paste, verifies Ctrl+C/Alt+X/Ctrl+Z do not add text, then continues the original typed filtering, font selection, saved readback, live size/default and Escape/history checks. All 16 settings-filtered library regressions pass, including real keybinding recording and numeric editing.

### Subsequent native paste timing correction

The first repaired executable `d8982568...` still showed an empty filter when native text-entry automation temporarily supplied clipboard text and restored it. The initial library test kept the clipboard installed through deferred execution, so it proved ordinary paste handling but missed the lifetime boundary. Native paste was therefore not accepted.

A stronger actual Window key-event regression installs `Arial`, dispatches Ctrl+V, then restores the clipboard before deferred Shell consumption. Before correction it inserted `restored clipboard` instead of `Arial`, confirming the race. The Presenter now snapshots clipboard text at key capture only for an open font picker and a paste chord, carrying it in existing `SettingsKey.text`. The Shell font branch consumes that immutable event payload later; it no longer reads mutable clipboard state after deferral. No synchronous Shell/window update was introduced and recording/numeric-editor handling is unchanged. All16 settings library regressions pass after the correction. Final native paste still requires observation on the subsequently rebuilt executable.

## F6: allocation actions were below the ordinary settings fold

Native observation: all three screen rows and the diagram were present, but Apply/Cancel fell below the existing settings content viewport at the normal launch window. The coordinator supplied `native/native-displays-actions-below-fold-before-repair.jpg`; reviewer inspection confirms the duplicated diagram/callout/row role text and the separate allocation-controls line consumed the core action space.

Fix: each display row puts role markers and allocation controls into one aligned wrapping line. Callouts are produced only for diagram boxes too small for their role labels, matching the approved design instead of repeating every already-labeled box. The two existing numbering/Identify explanations share one naturally wrapping paragraph. All 44px control faces, fonts, actual/configured/pending markers, warnings, Details, host scrolls and transaction logic remain.

Proof: replace the existing two-host ordinary sample of 1400x900 with the observed launch sample of 1100x760, rather than adding dimensions. Before fix, embedded Apply bottom was799.5, outside the760px client. Strengthened assertions compare all three rows, summary and actions against the actual owning content-scroll bounds; standalone actions are correctly checked against their separate footer/client. An intermediate row-only repair still put Apply bottom729 below the content fold, so redundant callouts and explanatory-line spacing were corrected too. Final focused test passes both hosts, including repeated embedded action containment after a draft change adds pending markers/feedback and unchanged scoped Cancel/save behavior.

## Verification and build checkpoint

- Focused library regressions for all four repairs passed; red-before/green-after receipts are recorded above from actual terminal output.
- `cargo fmt --all` and `git diff --check`: PASS after source repairs.
- The coordinator confirmed the app exited normally with code0 before final application build/link. Native first-round observations are bound to the initial checkpoint, not these repaired sources.
- `just ci`: PASS, 194 default workspace tests plus format/strict lint/dependency policy. Full log: `integration-native-repair-ci.log`.
- `just check-shell`: PASS, `gpui,pdfium` application build, strict native clippy and all136 GPUI library tests. Full log: `integration-native-repair-shell.log`.
- Whitespace check: PASS, `integration-native-repair-diff-check.log` (empty successful receipt).
- No remaining concrete source findings from these four repairs. Native resmoke remains coordinator-owned and must use the new identity below. Passing library/layout tests do not establish native pixels or paste behavior.

## First repaired source/executable handoff

- Base HEAD `7c25e46d1868f5520b8c2225c2ab89acfe9cfbed`, branch `codex/workbench-settings-design`, uncommitted.
- Manifest `integration-native-repair-source-manifest.json`, recorded UTC `2026-10-18T10:05:46.0954512Z`, includes changed product source/locales/root DESIGN hashes.
- Manifest SHA-256 `dff986a0b4c812448e4aeabe27840ba45b30df3838b38d37d4c9fa8a5258547e`.
- `target/debug/ferrostage.exe`,43,302,912 bytes, SHA-256 `d89825682523478701e620dc4fb5c809f93fcacb924411652af838f9519a1c64`.
- Staged PDFium remains the pinned SHA-256 `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`.
- Coordinator notified that this executable was ready, then native paste timing feedback reopened the specific correction described above. No desktop launch, commit, push or archive by reviewer.
- Remaining native actions: confirm helper lines no longer clip; paste into font filter and retain ordinary key/selection/default/size/restart behavior; confirm one progress-detail presentation in combined hover/focus; inspect compact ordinary embedded/standalone Displays and pending Apply/Cancel, then Start/Stop/Home lifecycle and clean close. Coordinator owns all final screenshot/input/exit receipts.

## Final event-time-paste source/executable handoff

The coordinator confirmed PID65268 exited normally with code0 before this final link. Comparing product hashes against the first repaired checkpoint shows only `presenter.rs`, `shell/preferences.rs` and `shell/preferences_tests.rs` changed for the paste timing correction. Display, session, pixel and lifecycle production paths are unchanged from that checkpoint. The coordinator reports successful native allocation/Cancel/Apply, actual PDF on screen3, black audience cover, standalone actual-role labels, Stop and Home on the prior executable; those empirical results remain explicitly bound to that prior executable rather than being silently relabeled final-source observations.

| Final check | Result / receipt |
| --- | --- |
| `just ci` | PASS: format, strict default lint,194 workspace tests, dependency policy; `integration-final-ci.log` |
| `just check-shell` | PASS: native application build, strict native lint,136 GPUI tests; `integration-final-shell.log` |
| `git diff --check` | PASS; `integration-final-diff-check.log` |
| Transient clipboard/library regression | PASS: event-time `Arial` retained despite restoring clipboard before deferred consumption; numeric/keybinding regressions retained |
| Final native paste/restart/fallback/light observation | Pending coordinator resmoke; no native PASS inferred from these tests |

- Base HEAD remains `7c25e46d1868f5520b8c2225c2ab89acfe9cfbed`; branch `codex/workbench-settings-design`, uncommitted.
- Final manifest: `integration-final-source-manifest.json`, UTC `2026-10-18T10:20:56.4505648Z`, SHA-256 `bd55dc71e4fe364eee2cfbc1f565fc33597071218bc78c1eafd74f91c1030f8f`.
- Final executable: `target/debug/ferrostage.exe`,43,302,912 bytes, SHA-256 `8e8046914806af6bea6f42e26b8c82d3b89a26a954b6264779fd8e3f3e3e9a47`.
- Staged PDFium retains pinned SHA-256 `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`.
- Removed only generated `crates/ferrostage-ui/unused-headless-settings.toml`. Its09:44 creation and recent limit11 are consistent with the initial obsolete Shift+Tab Close tests activating the last increment control; the path is the headless fixture's dummy store, not user settings. It did not recur in the final successful suite. No product or test-fixture restructuring was needed for cleanup.
- Coordinator notified final executable ready; reviewer product sources frozen after this checkpoint. No desktop launch, commit, push or archive by reviewer.

## Subsequent native diagnosis and F7 recent-row repair

On executable `8e804691...`, coordinator native restart retained Consolas18 and ordinary key filtering worked. Automated `sky.type_text` still left the filter empty. This observation alone does not prove the application's paste handler fails for a stable clipboard, nor establish that the automation restores its clipboard after Windows dispatches the key. No further paste code, retry/delay or platform/vendor patch was added on that assumption.

Pinned source review: `gpui_windows/src/platform.rs:897` delegates to its synchronous `clipboard.rs:92`; that opens Windows clipboard once and reads CF_UNICODETEXT. Failed OpenClipboard logs an error; task native logs contain no such diagnostic. The pinned GPUI example `vendor/gpui/examples/input.rs:147` implements Paste using the same `cx.read_from_clipboard().and_then(|item| item.text())`. Its text-input handler is a different ordinary/IME text path, not additional clipboard synchronization. Coordinator will use native filename-edit Copy followed by native Ctrl+V in the picker as a stable-clipboard control. Paste remains unverified until that native result; source/library tests do not substitute for it.

Separately, `native/final-source-recent-path-clipped.jpg` establishes a real UI clipping issue at the persisted Consolas18 setting. The ordinary General containment regression was extended, without adding dimensions, to use Consolas18 and three real repository fixture paths, then scroll to recent history. Before repair the first parent path origin was y572.5 with height22 (bottom594.5), while its owning row was y529.5 with height56 (bottom585.5):9px overflow. This is the same flex-column shrink boundary as F4, on a separate informational row renderer.

F7 fix: add `flex_shrink_0()` to settings recent rows so filename/parent content establishes row height and the existing settings scroll owns overflow. Debug selectors permit assertions that every parent has at least its selected font height, remains within its row, and is fully inside the scrolled visible content. No font reduction, metadata changes, history action or scroll redesign. The actual containment assertion failed before repair; all16 settings library regressions passed after it. Only `presenter/settings_view.rs` and the existing General containment test changed for this repair. Coordinator confirmed PID59356 exited0 before rebuilding.

## Latest source checkpoint after F7

- `just ci`: PASS,194 default workspace tests, format, strict clippy and dependency policy. Receipt: `integration-recent-final-ci.log`.
- `just check-shell`: PASS, native application build, strict native clippy, all136 GPUI tests. Receipt: `integration-recent-final-shell.log`.
- `git diff --check`: PASS, `integration-recent-final-diff-check.log`. Generated dummy headless store did not recur.
- Final manifest `integration-recent-final-source-manifest.json`, UTC `2026-10-18T10:28:04.0023089Z`, SHA-256 `e9cfdb9cd245b793c7389385e0a39de1bf929f219a96da505eb3fdbe875ae174`.
- Final executable `target/debug/ferrostage.exe`,43,303,424 bytes, SHA-256 `0ecb9c1cd6bab70f0a5a9273e037edc269a5b77ef72ddb129884b063216c7df5`.
- Product source frozen and ready signal sent to coordinator. No further paste changes, delay/retry, vendor changes, added dimension cases or dependency changes. No reviewer desktop launch/commit/push/archive.
- Coordinator plans final native Consolas18 recent-row and stable clipboard Ctrl+V observation, then missing-font fallback/light core smoke. Those results remain pending and coordinator-owned at this handoff.
