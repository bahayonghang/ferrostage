# Native home verification

## Setup and isolation

Used the documented Computer Use `@oai/sky` API against returned FerroStage windows, with the exact HWND independently matched to the launched executable/PID. The helper inherited the parent AppUserModelID; this was not automation of Codex. Both baseline and rebuilt application used isolated settings and repository-owned PDF copies under `%TEMP%/ferrostage-home-01a1521e`. The user's actual settings, history and PDFs were not modified.

Copied only the actual UI preference values into the fixture: zh-CN, system theme, WenYuan Serif SC, size 18. The fixture history contained three long paths, two synthetic known event times and one unknown legacy entry. Synthetic times were test data, not reconstructed user history. Both windows were ordinary 1103x792 outer windows, corresponding to a 1100x760 client. No size/DPI matrix was introduced.

## Observed results

- Baseline: installed executable, PID 70148/HWND 2169370. Screenshot showed filename/parent pairs starting around x174, spanning approximately 754px to x928, no timestamp header/column. Filename tops were approximately y319/374/430 in outer-window screenshot coordinates.
- Rebuilt debug: PID 165580/HWND 1118164, source-bound debug product build PASS. Startup argument opened a deterministic truncated PDF. Screenshot showed a flat header/separator table with filename/path left, shared right-aligned dates, and `暂无记录` for the legacy entry. Dates stayed complete and long paths truncated with their existing full-path disclosure retained.
- The actual requested serif font rendered in the rebuilt window. Header and three dates shared the same right edge at approximately x928. The visible path content ended around x734, keeping the date area separate. Independent headless bounds at this font size measured filename x174..739 (565px), header/time x755..926 (171px), with a 16px gap. Native screenshots agree visually; screenshot ruler values remain approximate.
- The truncated PDF produced Chinese conditional compile/export wait, reopen and re-export guidance, with no raw parser English on screen. The corresponding `lopdf could not parse the buffer` diagnostic remained in tracing. This does not establish that an external compiler was running.
- Clicked the legacy row in the real native window. It loaded into Ready and rendered the fixture PDF. Returning home showed that entry first, with a newly recorded local minute-resolution time. Independent settings readback confirmed three paths/three timestamps and the newly opened legacy path first with persisted metadata. The earlier failed truncated-PDF import had kept its fixture history/times intact.
- Native Alt+F4 closed both task-launched windows. The rebuilt PID exited, stderr was empty and stdout contained only the expected partial-PDF warning, without late dispatcher/invalid-handle diagnostics.

Window captures initially showed occluding foreground content. A fresh returned-window selection and one activation retry produced the actual target screenshot; only unobscured FerroStage captures were accepted as evidence. No coordinates from occluded captures were used.

## Consumer accounting and limits

History consumers: checked 2 / changed 1 / left 1. The home history table changed; the settings history list retained its existing presentation and passed the full GPUI regression suite. Native workbench load/return also passed. Full-path keyboard disclosure and Enter/Space are established by actual GPUI input regressions; native mouse row activation is separately observed. Empty-state and preserved settings/workbench behavior are covered by the full GPUI suite, not claimed as additional native screenshots.

Installation, signing, clean-machine qualification and publishing were not performed. Release packaging and final CI/policy results are recorded separately.
