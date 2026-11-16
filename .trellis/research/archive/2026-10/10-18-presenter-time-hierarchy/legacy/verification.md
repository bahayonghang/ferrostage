# Implementation verification

Date: 2026-10-18. User approved the combined plan and D1 notes rejection. Implementation was validated before the user requested local Chinese commits and Trellis closeout. No push is requested.

## Product checks before independent cache correction

| Command | Result | Evidence |
| --- | --- | --- |
| `just ci` | PASS, exit 0, 16.9s | Implement agent: format, strict default lint, workspace tests and cargo-deny |
| `just check-shell` | PASS, exit 0, 19.7s | Actual GPUI/PDFium app build, strict feature lint, 165 GPUI tests |
| `just test-native` with pinned PDFium DLL | PASS, exit 0, 7.3s | Native document/render suites, compiled Beamer public/private crop checks |
| Isolated `native_perf100_grid_retention` UI test | PASS, 1/1, 18.6s including build | Real PDFium thumbnail ownership/retention in both numbering modes |
| Isolated `real_initial_load_notes_projection_and_failed_candidate_preserve_active` | PASS, 1/1, 10.5s including rebuild | Fresh No split assertion followed by explicit Automatic crop checks |

The attempted concurrent `pdfium,test-support` full UI run was FAILED (169/171). Two simultaneous real runtime owners hit the process-global PDFium binding constraint, and the initial-load test still expected automatic cropped notes from default import. The default assertion/explicit-Auto setup was corrected without weakening pixel assertions. The two affected real targets passed in separate processes. Native hot-reload child passed, checking preserved PDF numbering and explicit Automatic. This attempted combined run is not reported as PASS.

## First normal-window native smoke

Launched the actual feature executable with an isolated task config and `fixtures/beamer/independent.pdf`; no ordinary user settings were changed. Computer Use captured the actual window at 1102 x 790 screenshot pixels, default UI scale, dark theme and Chinese labels. The synthetic fixture intentionally uses colored content and irregular labeled notes; it is a mapping test input, not an aesthetic replacement for the user's slides.

- Quiet local clock appeared before filename at the upper-left; restrained labeled 28px timing remained on the right.
- Standalone Navigate was absent; the four annotation tools remained.
- Content mode opened by default with 8 content positions, not the 7 logical groups or 14 physical positions. Captions mapped to actual PDF 2,5,7,9,10,12,13,14.
- Switching to PDF original produced 14 positions and marked notes placeholders, with no rendered note previews.
- Input 1 (an orphan notes page) showed exactly 备注页不可投影; Jump was disabled. Enter preserved the panel, target rejection and current physical page 2, without redirection.
- Layout & navigation contained exactly Automatic, No split (checked) and Identify independent notes pages (checked). The four directional choices and old physical-navigation checkbox were absent.
- Closing the isolated presenter returned exit 0, with empty stderr. PDF SHA256 before and after was `C5F6204DA4C5A7C2C7775C3484BB2327652C15509261E2ADD95D40AE8AFC2568`.

First executable SHA256: `89854EA7F8AC38CAC0402BAEAAE7054B3FB092178ED94DDEAD6501472004D79F`. Launch/exit receipts are under `research/native/`. These initial observations precede the independent cache correction and are not the final-binary freshness receipt.

## Independent correction and final verification

Independent full-scope review found a real AC16 gap: clearing Workbench thumbnail frames did not purge Presenter TextureCache fallback pixels. A local identity-transition retirement using the existing cache synchronization boundary was added, preserving current/next/notes surfaces. A seeded actual-cache regression requires no old raster before the remapped slot receives a replacement. The cache regression was confirmed RED without retirement and GREEN after restoration. Independent review found no other concrete product-code issue; see `research/check-review.md`.

Final gates after all product/test changes:

- `just ci`: PASS, exit 0, including default strict lint, tests, format and cargo-deny.
- `just check-shell`: PASS, exit 0. Actual GPUI/PDFium application build 9.69s, strict feature Clippy 2.87s, all 166 GPUI tests passed (0 failed, test runtime 1.19s).
- `git diff --check`: PASS. No warning/advisory waivers or dependency changes.

### Refreshed native executable

Final executable SHA256: `CFDB8F498420E191B29FDBACBDE6D2461A4B23973FB4AA914C9B7985738A69D0`. This is the rebuilt binary after the cache retirement correction. `research/native/final-launch.json` and `final-exit.json` bind the run to that executable. It exited 0 with empty stderr, and the fixture PDF hash remained unchanged.

Observed in Chinese/dark at the normal 1102 x 790 outer window:

- Ready and Presenting retain quiet leading wall clock and unboxed labeled timing; the extra pause/reset controls fit without header growth.
- Starting presentation changed stage and countdown proceeded normally.
- Paused countdown remained 44:51 while wall clock advanced from 11:59:07 to 11:59:36. Resume continued to 44:48; Reset restarted near 45:00 (44:57 after capture latency).
- Content input 2 revealed PDF 5 without committing. Enter committed exactly that page, closed the panel and changed the progress to step 2/2, physical 5/14.
- Reopening and selecting PDF original converted editor value 2 to 5 and moved the current indicator to the original PDF 5 slot. Content denominator 8 and PDF denominator 14 remained coherent.
- Input 1 showed 备注页不可投影 and disabled Jump. Enter kept the panel open and committed page at physical 5/14, with no redirection.
- Escape dismissed the browser without stopping the presentation. Reopen retained PDF mode and initialized the editor to current physical page 5.
- Native title-bar close ended the isolated presenter cleanly.

Representative English/light native run used the same final executable and separate `smoke-en-light.toml`. Ready, Presenting and Content-mode panel were inspected: clock/timing labels, Pause/Reset, action labels, numbering choices, input/range and grid captions fit and remained readable. It exited 0 with empty stderr; receipts use `en-light-*` names. This is representative language/theme coverage, not a Cartesian matrix.

One capture was occluded by unrelated desktop activity; it was discarded and one bounded reactivation succeeded. That occluded image is not used as acceptance evidence. The actual-cache stale-image case is established by the seeded RED/GREEN regression, rather than claiming the uniform-color native fixture can visually distinguish every physical raster.

Surface accounting: checked 4 / changed 4 / left 0 (time header, tool strip, layout menu, page browser). No requested product feature remains unimplemented.

## Acceptance boundary

Automated behavior and native screenshots are separate evidence. No CPU/FPS/performance improvement, exhaustive dimensions/DPI matrix, or complete physical-monitor qualification is claimed. Native warning-threshold/overtime edge colors and clock-disabled state were not individually driven in the final physical window; their calculations, labels, geometry and visibility were checked by focused GPUI regressions and shared palette/catalog checks. No exhaustive edge-state or dimension matrix is claimed. Chinese/dark and English/light normal-window observations are explicitly recorded above.
