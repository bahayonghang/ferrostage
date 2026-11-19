# Windows acceptance for settings layout and font search

Verified on 2026-11-19 using the native presenter, an isolated writable settings file, Windows UI Automation and client-only window captures. The app ran at 125% display scaling. The release build used the GPUI and PDFium features. Its SHA-256 was `62c4903290eb814339c3689260d7b73722ab7a7200324174c36bebf6136f7c8d`.

## Layout

| Native client | Left margin | Right margin |
| --- | ---: | ---: |
| General before, 2398 × 1303 physical pixels | 30 px | 928 px |
| General after, same client and settings | 479 px | 479 px |
| Displays after, same client | 379 px | 379 px |

Margins run from the client edges to the category rail and the right edge of the form. The old Close button was at the window edge. It now lines up with the form. GPUI geometry tests also check the title and feedback alignment and the 960 / 1120 logical-pixel body widths.

[Before](before-general.webp) · [After](after-general.webp) · [Wide Displays](wide-displays.webp)

At an 870 × 1003 physical pixel client and UI font size 20, navigation wrapped into two lines. Scrolling to the recent-file limit and clicking + saved 11. Switching to Displays made Refresh immediately accessible. Refresh and Close both worked. The GPUI regression also exercised a 390 × 640 logical-pixel window at font size 24.

[Narrow General](narrow-general.webp) · [Narrow Displays](narrow-displays.webp)

## Font search

The field opens focused and exposes its localized name and value to Windows UI Automation. Typing `arial` and pressing Enter persisted Arial. Enter on an empty result kept the field open. Switching to Light with a live query retained the text. Clicking a different visible match persisted Arial Rounded MT. The automated settings input tests cover selection replacement, paste, scrolling to the last candidate, Tab and Shift+Tab, Escape, explicit default restoration, persisted configuration and presentation-key isolation.

[Old picker](before-picker.webp) · [New picker](after-picker.webp) · [Dark query](dark-query.webp) · [Light query](light-query.webp) · [No results](dark-empty.webp)

| Actual input pixels | Text / field | Focus border / field |
| --- | ---: | ---: |
| Light | 14.82∶1 | 5.13∶1 |
| Dark | 14.57∶1 | 8.27∶1 |

The Windows Chinese IME composed `ni` and committed 你. The query field stayed at the same client position while the match list emptied. During a separate `yu` composition, pressing Enter kept the picker open. UI Automation supplied the query value and the saved configuration confirmed that entering Chinese text preserved the existing font.

[IME composition](ime-preedit.webp) · [Committed text](ime-commit.webp)

## Checks

- Release build and Clippy with GPUI and PDFium passed. All 284 GPUI-enabled UI tests passed.
- `just ci` passed formatting, workspace Clippy and all 253 default-feature tests. The final `cargo deny` advisory check failed on `chrono 0.4.45` through `lopdf 0.45.0` (RUSTSEC-2026-0543). Running the dependency check in a clean worktree at the original commit `d1c9b63` reported the same advisory. Bans, licenses and sources passed on both runs.
