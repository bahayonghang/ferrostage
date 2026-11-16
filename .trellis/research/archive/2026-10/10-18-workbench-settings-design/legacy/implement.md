# Implementation sequencing and final acceptance

## Entry and ownership

- [x] User explicitly approved the latest plan and requested sequential Trellis sub-agent implementation on 2026-10-18, followed by independent review and native core/visual verification without size/DPI expansion.
- [x] Rechecked current source/diff and existing home work at `7c25e46`; only four untracked planning directories existed. Preserve unrelated user changes.
- [x] Implement selected children sequentially: settings, workbench, displays. Use Trellis implement/check sub-agents as required by the project when execution is authorized; every dispatch begins `Active task: <selected child path>` and assigns owned files. All three children were subsequently dispatched sequentially after user approval.
- [x] Parent stays an integration/planning container; activate the child actually being implemented. Curated context is spec/research only, not a source-code manifest.

## Child gates

1. Settings: schema/default/parser/merge tests, live typography and fallback, recent rows, all-category shared composition, save/error truthfulness; update root DESIGN before code.
2. Workbench: update root workbench DESIGN contracts, unify Start/Stop/navigation strip, concise progress, tool selection; retain notes/manual navigation and stage/busy guards.
3. Displays: sample actual locations, publish Presenter-only projection, compact shared body, remove nested scroll, show actual/configured/draft and transactional feedback; retain policy and lifecycle.

Do not run these concurrently against theme/Presenter/locales/shell. A child's `evidence.md` must identify its local ACs and the parent's ACs it satisfies. Expand the plan first if source research discovers a materially different scope, dependency change or product behavior.

## Verification budget

During each child run focused tests named in its plan. On final integrated source run:

```powershell
just ci
just check-shell
git diff --check
```

`check-shell` includes native app build, strict native clippy and GPUI tests; avoid immediately rerunning identical commands without an invalidating change/failure. Run focused settings tests when schema changes. Existing license checks remain in default CI; wider optional-dependency failures are reported separately without waivers or migration work.

Normal-window native checks use one representative ordinary working window and current three-screen arrangement, dark and light with representative Chinese/English text. Compare before/after workbench, General, Displays. Observe font selection/size/default/fallback and persistence, recent clear, page jump/tool/blank controls, Start/Stop, pending allocation/Cancel/Apply and actual window move. Use existing isolated task-owned config and PDF fixtures. Record source/executable identity, theme/locale, state and screenshot paths. Do not change the user's PDF or personal settings for a probe.

Do not add a minimum-window, breakpoint, mixed-DPI, physical unplug/replug or performance qualification matrix. Exercise stale/missing/ambiguous topology through existing fixtures; physical exhaustive tests remain deferred. Keep basic keyboard/focus/contrast correctness in the implementation. If native observation/capture is unavailable, make one bounded attempt, record the exact missing view/action, and do not label it PASS. Browser mockups do not count.

## Integration gate

- [x] Parent AC1–AC12 map to actual child evidence, not only a plan or constant assertion.
- [x] Home remains visually consistent and usable; font setting intentionally changes UI text but not the wordmark role. Settings/Displays entry/return still work.
- [x] Ready has no audience; Start creates audience only in Presenting; Stop/Home closes it; no notes leak or audience theme/settings controller.
- [x] One shared theme/type scale and coherent native controls across children; no stale separate Start footer, nested display scroll or contradictory Saved text.
- [x] Review complete diff, root DESIGN and applicable spec updates; keep unrelated files intact. Roll back only the failing child's cohesive diff if necessary, retaining prior valid checkpoints.
- [x] Report completed native/core checks and explicit gaps. Do not commit/push/archive unless subsequently authorized and acceptance is complete.

## Planning completion receipt

Planning validation checks task links/status, presence of all three documents, requirement mapping, manifest paths, screenshot availability and whitespace. This validates planning artifacts only; product tests and native visual acceptance have not run in this turn.


## Coordinated final receipt

Implementation, independent review and requested core/visual acceptance completed on 2026-10-18. The parent [verification](../10-18-workbench-settings-design/research/verification.md) is authoritative for final source/executable hashes, 194 default / 136 GPUI checks, actual screenshots and exact native limits. Native clipboard paste remains inconclusive and is not counted as passed. No commit, push or archive was performed; lifecycle status remains in progress pending that separately authorized step.
