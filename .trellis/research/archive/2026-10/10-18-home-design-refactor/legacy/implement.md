# Implementation and acceptance plan

## Entry gate

- [x] User approved the final plan and explicitly requested Trellis implementation/check and native visual verification on 2026-10-18.
- [x] Confirmed current task/diff/source; loaded pre-development guidance and dispatched the Trellis implement sub-agent with explicit ownership.
- [x] Ran task.py start after user approval; task is in_progress on codex/home-design-refactor.
- [ ] Follow repository Trellis implementation/check sub-agent workflow. Every dispatch begins with current `Active task:` path, assigns file ownership, and preserves other workers' edits.

## 1. Comparable baseline and design source

- [ ] Build before staging PDFium/launch through existing `just dev`; do not manually repair the cache-managed target. Use current native launch/test conventions and isolated task-owned settings/fixtures.
- [ ] Capture unchanged Idle with 0/3 recents at 1100x760 and 1440x900 logical client sizes. Record revision/dirty diff, executable path/hash, client rectangle, scale, theme, locale and fixture settings. User screenshot is supplementary because its DPI/client metadata is unknown.
- [ ] Measure Open face/outer target, `recent-0`, content/brand/utilities via actual GPUI debug bounds: width, height, left edges, row gap and column width. Separate focus reservation from visual face.
- [ ] Update root DESIGN to the reviewed contract, retain provenance, then implement only necessary semantic tokens in `theme.rs`.

Verify: source/geometry evidence, no parallel theme/global density change. AC1-4, AC7, AC10.

## 2. Coherent home implementation

- [ ] Add dedicated brand and bounded composition, using `presenter/home_view.rs` if it improves readability.
- [ ] Keep one `welcome-open`; recent rows use basename/parent, original import paths, stable IDs and full-path discovery.
- [ ] Apply copy role table in both locales, including empty history. No placeholder data in production.
- [ ] Make title/chrome/footer stage aware. Idle has no times/status/duplicate Open/Start footer; document screens retain controls/timer/clock.
- [ ] Preserve feedback, save Retry, chooser/drop, activation, busy guards and focus recovery. Extract only a small shared activation wrapper if needed; inventory and review every affected shared consumer.

Verify: empty/default home, cancellation, loading, missing file, topology warning, save error/Retry, Settings/Displays open/close and Home return. AC1-6, AC8-9.

## 3. Focused regressions and required gates

Use existing GPUI fixtures in `shell.rs`:

- [ ] Actual Idle tree has one Open, dedicated brand, natural primary width, bounded left-aligned rows, no timer/start/footer chrome.
- [ ] Check 0/1/3 recents, same-basename parents, long Chinese/English path at narrow width, full-path discovery and target bounds.
- [ ] Actual Tab/Shift+Tab, Enter/Space, accessible click, disabled/busy, modal return focus, import failure and Home return paths. Exactly one invocation, no hidden focus stop.
- [ ] Preserve `presenter_bar_shares_one_row_and_notes_fill_the_column`, `idle_presenter_has_no_home_button`, `home_returns_to_welcome_and_keeps_recent_files`, `home_from_presentation_closes_the_audience`; extend home assertions without dropping unrelated checks.
- [ ] Existing chooser/focus/guards and Ready/Presenting timer/clock, manual navigation, notes priority, Start/Stop remain correct.
- [ ] Reuse contrast and locale parity tests; extend only new pairings. Do not test mere constants equal themselves.

```powershell
cargo test -p ferrostage-ui --locked
just ci
just check-shell
cargo deny --all-features check
git diff --check
```

For focused diagnostics use `cargo test -p ferrostage-ui --features test-support --lib --locked <filter>` before the full gates as needed. `just check-shell` already performs native-feature app build, strict clippy and GPUI tests; do not immediately rerun the identical gate without a change/failure. Default/headless success is separate from native visual acceptance. Record environment/cache failures separately. AC5-10.

## 4. Native visual and interaction matrix

Render each geometry below in light/dark and zh-CN/en-US with 0/3 recents. Use pairwise coverage for other states rather than an uncontrolled Cartesian product. All sizes are logical client sizes.

| Probe | Purpose |
| --- | --- |
| 1440x900, 1100x760 | Large and launch-size hierarchy/comparable baseline |
| 799x600, 800x600 | Both sides of home gutter boundary and compact desktop |
| 599x600, 600x600 | Both sides of utility wrapping |
| 320x760 | Minimum width alone |
| 1100x240 | Minimum height alone |
| 320x240 | Combined actual minimum, vertical scroll and focus reachability |
| 390x844, 320x640 | Existing root DESIGN narrow-window probes |

- [ ] Additional states: 1 recent, long CJK/Latin path, same-basename files, hover/pressed/focused, loading/busy, missing/invalid recent, chooser cancellation, topology warning, unsaved/save failure/Retry, modal open/close.
- [ ] Repeat representative normal/narrow probes at 100/150/200% OS scaling; distinguish bitmap/logical sizes. Check increased text scaling where supported; record unsupported cases.
- [ ] Targets >=44x44; no horizontal overflow/clipped ring/essential label; all controls reachable at 320x240. Default-scale Open face 120-200px, recent targets 48-56px, row gaps <=8px, shared left edges within 2px.
- [ ] Ready/Presenting regression screenshots: unchanged notes/workbench, timer/clock and Start/Stop, audience only while Presenting, Home returns to new Idle.
- [ ] Store screenshots plus `research/verification.md` with matching-size before/after geometry and final source/executable identity. Report `checked N / changed M / left K` across the actual affected shared consumers.
- [ ] Aesthetic review: one FerroStage anchor, distinct brand/UI type, useful whitespace, file-like recents, no generic cards/gradients, duplicated labels or fake data.

If native capture is unavailable, mark exact size/state `NOT_RUN`; do not replace it with compilation/browser mockups. Make one bounded capture attempt after launch rather than repeatedly fighting focus. AC1-10.

## Review and rollback

- [ ] Check sub-agent reviews complete diff against PRD/design/specs, requirement-to-evidence mapping and preserved boundaries. Fix issues; rerun affected/invalidated gates.
- [ ] Bind final evidence to last source/executable; each AC has PASS evidence or an explicit gap. A gap cannot silently become PASS.
- [ ] Keep implementation within planned view/theme/locale/tests/design files. Wider core/settings/platform changes require returning to planning.
- [ ] Roll back cohesive view/token/copy/test/DESIGN change if shared interactions/workbench break; no migration rollback.
- [ ] Leave product changes uncommitted for user review; do not push or archive without completing acceptance and the applicable commit workflow.

## User scope correction, 2026-10-18

The user explicitly directed: do not validate dimensions or similar non-core behavior, and persist this preference globally. Stop further native size/breakpoint/minimum-window/DPI matrices. Their unexecuted cases are no longer blockers for this task. Existing passing regression evidence is retained without more matrix expansion. Continue only core behavior, normal-window visual smoke and necessary build/lint/relevant regressions. No exhaustive accessibility or physical-display qualification is required for this home polish task. The unrelated existing all-features dependency-policy failure remains truthfully reported separately; do not add waivers or expand this UI task into dependency migration.

## Execution outcome

- Trellis implement and independent check completed; root DESIGN and frontend executable guidance updated.
- Core normal-window native smoke completed with final source/executable binding; settings, chooser cancellation, recent import and Home return observed.
- Default CI/native build/strict clippy/187 workspace tests/126 GPUI tests pass.
- All-features deny fails on unchanged optional Linux unmaintained dependencies, preserved without waivers.
- Per latest user instruction, stop non-core dimension/DPI matrices. No additional matrix completion gate is pending.
- Full receipt and remaining dependency item: research/verification.md. Changes remain uncommitted for review.
