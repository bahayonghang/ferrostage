# Final implementation and core native verification

## Scope and outcome

The approved home redesign is implemented using Trellis implement/check sub-agents. The user subsequently directed that window-size and similar non-core validation must stop and that this preference be recorded globally. That instruction supersedes the original exhaustive native geometry/DPI matrix. Unexecuted matrix cases are not completion blockers for this UI task; do not resume them without an explicit request.

Core outcome: one dedicated FerroStage wordmark, concise sans interface copy, one natural-width Open PDF action, flat recent-file rows with real parent paths, and no Idle document-status/timer/disabled Start chrome. Existing document-stage controls, feedback and import authority remain intact.

## Source binding

- Branch: `codex/home-design-refactor`.
- Baseline HEAD: `595910cc41d6259a3e87766c2aceb2c505d28359`; evidence captured before the user-authorized local commit.
- Final executable: `target/debug/ferrostage.exe`, SHA256 `2C109A1CC8CC944DE2AB3F11F24FC0C605E1921B6E6480E404A2C8E739DB3E9A`.
- Independent source/build receipt: `check-source-sha256.txt`.
- Native process/config/source/DLL hashes: `native/zh-cn-dark-3-launch.json`. Task-local settings were used; normal import/settings persistence can normalize that fixture config after launch.
- Final run exit: 0 in `native/zh-cn-dark-3-exit.json`. Its stdout/stderr logs contain no product diagnostics; no post-exit invalid-handle diagnostic appeared in the completed runner output.
- No production source changes occurred after this final build; subsequent edits only record scope and local-commit authorization. The checker added one test before the final quality gates; no production correction was required by independent review.

## Core native observations

Real GPUI Windows window, normal launch size, Chinese/dark theme on the current host. These are actual native captures, not a browser recreation or headless test render.

| Action / observation | Result | Evidence |
| --- | --- | --- |
| Home first settled view | One distinct wordmark, quiet Settings/Displays, compact Open, left-aligned filenames and actual parents; no duplicate header, times or Start footer | `native/zh-cn-dark-3-1100x760.jpg`, `native/final-home.jpg` |
| Tab then Enter on Settings | Settings opens through real keyboard dispatch | `native/core-settings-open.jpg` |
| Escape from Settings | Home restored and Settings focus ring restored | Observed native screenshot after Escape |
| Tab to Displays then Open | Focus follows visible utility/action order | Observed native focus rings |
| Enter on Open | Native PDF file chooser opens | `native/core-open-chooser.jpg` |
| Escape from chooser | Home remains usable; Open retains focus | Observed native screenshot after cancellation |
| Tab to first recent | File row gains visible focus and complete path appears below the list | `native/core-recent-focused.jpg` |
| Enter on recent perf-100.pdf | Actual PDF and notes/next-slide workbench load; Ready header/timer/Start return | `native/core-recent-opened-ready.jpg` |
| Tab to Home, Enter | Home returns, native title is FerroStage, recent entries remain, document-stage chrome disappears | `native/final-home.jpg` |
| Alt+F4 | Task process exits normally | `native/zh-cn-dark-3-exit.json`, empty stderr |

The first immediate capture after launch preceded the settled drawing and showed a blank surface. It was reobserved and not used as final evidence. No first-paint latency or transition performance claim is made.

Baseline visual reference: `native/baseline-zh-dark.jpg`. Its executable was pre-existing and correspondence to source was not independently proven. It is not a fully source-bound before/after pair. The normal-window change is visually clear: the original almost-full-width outlined commands became a content-sized primary action and flat file rows. The already-completed automated measurements record a 120x44 Open face, 128x52 target, and 52px recent targets with 4px gaps; those dimensions are descriptive evidence already gathered, not a new validation requirement.

## Quality gates

| Gate | Result |
| --- | --- |
| `just ci`: formatting, strict default clippy, workspace tests, default deny | PASS, 187 tests |
| `just check-shell`: native-feature build, strict clippy, GPUI regressions | PASS, 126 tests |
| Focused long-path regression | PASS, no production change needed |
| `git diff --check` | PASS |
| `cargo deny --all-features check` | FAIL on three existing unmaintained optional Linux dependency advisories |

The independent check is `check-review.md`. Its baseline proof demonstrates unchanged manifests, lockfile and deny policy. No dependency/policy waiver was added. Official notices confirm these are unmaintained-package notices, not a claim of a demonstrated vulnerability in the Windows home:

- [zvariant, RUSTSEC-2026-0370](https://rustsec.org/advisories/RUSTSEC-2026-0370.html)
- [zbus_names, RUSTSEC-2026-0371](https://rustsec.org/advisories/RUSTSEC-2026-0371.html)
- [zvariant_derive, RUSTSEC-2026-0372](https://rustsec.org/advisories/RUSTSEC-2026-0372.html)

Maintained dependency migration is outside this home redesign. Default quality gates and the Windows app build pass; the optional all-features policy gate is still FAIL and must not be reported green. Task remains `in_progress` for this outstanding quality item/user review, rather than being archived as fully accepted.

## Scope limits after the user's correction

- No further size, breakpoint, minimum-window or DPI/scaling validation. Earlier automated checks are retained. Native resize attempts did not establish changed dimensions and are not claimed as successful probes.
- No exhaustive native theme/locale/state, physical projection or screen-reader qualification. Both shipped locale catalogs and themes are covered by automated regressions/contrast checks; only the named native observations above are claimed.
- Computer Use exposed no useful UIA tree. Native keyboard invocation was observed; OS accessibility Click invocation is not claimed.
- Do not turn these excluded non-core qualification items into completion blockers. Preserve meaningful core import/input/feedback/lifecycle regressions.

## Aesthetic and consumer review

Warm editorial utility remains the design direction: Georgia distinguishes FerroStage, while the system sans stack keeps controls and filenames readable. The clay accent is limited to the primary action; file rows use alignment and typography rather than card borders or decorative effects. No placeholder product content, logos, external fonts or invented metadata were added.

View families: **checked 5 / changed 1 / left 4**. Idle changed; existing document composition, workbench, settings and displays retained. Consumer sites: **checked 49 / changed 3 / left 46**. Details and automated evidence are in the implement/check reports.

## Global preference receipt

User-requested memory note was written and reread at `C:\Users\lyh\.codex\memories\extensions\ad_hoc\notes\2026-10-18T08-19-45-604Z-focus-core-validation.md`. It records that core behavior and normal-window visual smoke take priority, and non-core size/DPI matrices require explicit user request. Existing MEMORY registry files were not edited.
