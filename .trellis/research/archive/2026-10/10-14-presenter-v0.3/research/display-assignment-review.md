# Independent display-assignment planning review

2026-10-14 America/New_York.

**Disposition: planning consistency PASS after three corrections by Root.** No remaining blocking inconsistency was found in the reviewed screen-assignment scope. This approves the coherence of the revised plan; the new UI is PLANNED and has no implementation or native acceptance claim.

The reviewer owns only this report. Root owns the planning edits, task records and native evidence. No product source, existing report, task state, UI, build, test, commit or archive was changed or executed by the reviewer during this review.

## Findings (fixed)

### P1 — Reusable prompt retained the superseded Gate B rule

- File: `implementation-prompts.md`, introductory routing and Prompt A dependency/priority clauses.
- Issue: the reusable instructions still permitted configuration-file-only roles for Gate B and deferred the role UI to09, contradicting07 G-13/I-8.
- Fix: Root made the current continuation/canonical plan authoritative and updated the clauses to require07's count/number/Identify/selectors/Apply before Gate B.09 integrates the shared panel later. The reviewer reread the changes and scanned current task steps; no remaining active instruction defers Identify to09 or accepts only configuration-file roles for the new UI requirement. Historical receipts remain historical.

### P2 — Single-screen no-play behavior was ambiguous

- Files: `research/display-assignment-plan.md`, parent `design.md`,07 `prd.md`/`design.md`,09 `prd.md`/`design.md`.
- Issue: leaving Audience unset does not disable Start when the existing single-screen `windowed_audience` mode is enabled. The actual source at `crates/ferrostage-ui/src/policy.rs:98` selects the Presenter as a windowed target in that case. Two role selectors alone could not encode the promised no-play choice consistently.
- Fix: the shared draft now commits roles and the existing `single_monitor_mode` together. No-play clears Audience and sets `presenter_only`; explicit single-screen window playback sets `windowed_audience` with only a Presenter physical binding; a selected physical PDF display sets Audience and `presenter_only`. No duplicate MonitorId binding or new schema is introduced.09 reuses these choices instead of providing a conflicting second mode control. This also prevents an inherited windowed preference from authorizing an unselected target after topology loss.

### P3 — No-play during Presenting conflicted with unconditional stage preservation

- Files: canonical plan, parent `design.md`,07 G-13/design/acceptance,09 U-10/design.
- Issue: the generic Apply rule preserved stage/timer, while09 already required safe Stop when there is no Audience. Selecting no-play needed an explicit exception.
- Fix: valid-target migration preserves page/stage/running timer. Explicit no-play while Presenting follows Curtain → Stop → audience close, retains the page, freezes the timer and returns Ready. The new unchecked acceptance includes this branch and single-screen Start/restart consistency.

## Findings (not fixed)

None within the requested planning review. Runtime implementation details, regression results and hardware acceptance remain future work; their absence is not recorded as a planning PASS for the product. The separately attributed V7 retirement/close errors remain unfixed and retain their own evidence.

## Verified consistency

- 03 owns complete enumeration, identity/confidence/source data and topology events. It does not own numbering persistence or UI. Successful zero-display enumeration is distinct from Unavailable.
- 07 G-13/I-8 owns the minimal presenter-side numbered list, application/PDF choices, draft Apply/Cancel and matching 3s Identify overlays before Gate B. 08 reuses that surface with real PDF content; 09 embeds it in the complete settings/topology page. 07 has no 09 prerequisite, and 08 still requires Gate B.
- Number labels are transient, stable for the same complete topology and shared with Identify. They are not asserted to equal Windows Settings numbers. Persistent authority remains MonitorId plus existing role/mode settings. Topology events invalidate stale drafts, and Apply revalidates current targets.
- Opening the panel does not migrate windows. Identify contains only number/name and is disabled while Presenting. Presenter activation confirmation, snapshot/notes isolation, actual placement confirmation, rollback and one SettingsStore save-error/Retry owner remain required.
- The primary Swap button is replaced. Existing configured Swap shortcuts remain compatible through the same coordinator; no second resolver or authoritative swap flag is introduced.
- Parent PRD/design/implement/traceability,03/07/08/09 plans, DESIGN.md, PROJECT_PLAN.md and the continuation prompt agree on the ownership and gates. The new07 UI acceptance checkboxes are unchecked; old checked configuration-based A/B/C observations explicitly do not satisfy them.

## Verification

- Read-only document/diff/source review: PASS. Parent and03/07/08/09 implement/check JSONL files parse, their referenced paths exist, and all ten include the canonical plan. This is independent context inspection, not a duplicate product test run.
- Historical evidence integrity: the V7 implementation manifest22 entries and previous native-independent213, native-V6 54, native-V5 108 and implementation-V6 26 entries all match stored bytes/SHA256. The V7 executable still matches its freeze; among its602 recorded inputs, only DESIGN.md differs because of this explicit planning revision. No old freeze was rewritten to conceal that delta.
- Task boundaries:03/07 remain `in_progress`;08/09/10/11 remain `planning`. Archived01/02/04/05/06 remain `completed`, with no Git diff under their archive directories. No task was recreated or reopened.
- Lint: NOT_RUN (planning-only scope; no product changes).
- TypeCheck: NOT_RUN (planning-only scope).
- Tests/native UI: NOT_RUN. Existing executable checks remain version-scoped evidence, not verification of the new flow.
- Physical unplug/reconnect/connector/latency and actual single/zero-display cases remain NOT_RUN. Gate B remains INCOMPLETE; Gate C/D, M1/M2 and final parent acceptance remain incomplete. Tasks10/11 retain P2 and their dependency gates. No submission, push or archive is implied by this review.
