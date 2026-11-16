# Execution Plan, Approved 2026-10-19

## Before Implementation

- [x] Obtain explicit approval of the latest proposal in a subsequent turn. Do not equate task creation with implementation approval.
- [ ] Re-read current diffs/specs. Preserve existing defense-experience edits and refresh drifting source anchors.
- [ ] Review probe, evidence and design; run task validation.
- [x] Activate only after approval, then use project Trellis implement/check roles with active-task path and curated context. No implementation agents were dispatched during planning.

## 1. PDF Metadata (AC1, AC4)

- [ ] Add bounded two-level outline extraction, owned entries, target issues and partial status. No new dependency.
- [ ] Generate redistributable synthetic PDF tests for hierarchy/order, unnumbered/Unicode titles, duplicates, direct/local GoTo/named destinations, invalid parent/valid children, missing/short arrays, references, cycles, bounds and external actions.
- [ ] Update DocumentStructure literals with empty outline defaults while retaining ongoing presentation fields. Do not copy the private 72 MB PDF into fixtures or CI.
- [ ] Run `cargo test -p ferrostage-pdf --locked`.

## 2. Document Mapping (AC2, AC4, AC6)

- [ ] Map through actual SlideMap steps after building the map. Test notes, overlay, identity, options and reload.
- [ ] Include nonalternating notes to catch divide-by-two shortcuts; note-owner mappings cannot silently redirect.
- [ ] Test current-row tie rules, duplicates and out-of-order destinations.
- [ ] Run `cargo test -p ferrostage-document --locked`.

## 3. Browser State/Lifecycle (AC3, AC6)

- [ ] Add session-local view and entry-id activation; reuse guarded submit/JumpTo.
- [ ] Renew browser id on view change; validate current entry and generation on commit. Cover old id/view/generation, replacement and reconfiguration.
- [ ] Separate contextual-open from thumbnail visibility. Retire hidden thumbnail frames/targets/textures and queued work; preserve speaker/next/notes/audience scheduling.
- [ ] Test zero commands on tabs/focus/hover/cancelled press, one activation, inert hidden controls and thumbnail request budget.
- [ ] Coordinate ongoing transition/autoplay changes by using existing committed JumpTo behavior.

## 4. Native Frontend (AC1, AC3, AC5)

- [ ] Amend root DESIGN.md's page-browser contract before implementing this branch, preserving unrelated edits.
- [ ] Render tabs, fully expanded directory, page columns, current badge, reveal, full titles and disabled/empty/partial states.
- [ ] Reuse GPUI tokens/contextual host/typography/press/focus helpers. Keep common editor and hide thumbnail density/footer only in directory view.
- [ ] Localize UI/accessibility labels in both locale files, leaving authored titles untouched.
- [ ] Route list keys contextually; preserve number-editor keys. Add first-draw, focus restoration and selection-separation tests alongside existing Shell/presenter-time coverage.

## 5. Gates and Native Evidence (AC1-AC7)

Run targeted checks first, then the broad gates once:

```powershell
cargo test -p ferrostage-pdf -p ferrostage-document --locked
just ci
just check-shell
cargo deny --all-features --locked --offline check
```

- [ ] When available, read the private sample and compare all 39 targets with the probe. CI uses synthetic fixtures, never a private absolute path.
- [ ] Native normal-window smoke: Ready/Presenting, both views/numbering modes, click/keyboard jump, current reveal, Escape/outside focus, long/empty/disabled states and relevant themes/locales.
- [ ] Confirm browsing leaves audience unchanged; committed activation shows the correct content without notes. Preserve ordinary ink/notes/overlay/timing navigation behavior.
- [ ] Capture native evidence; missing checks remain NOT_RUN. HTML previews do not establish GPUI acceptance. No extra min-size/DPI/performance matrix is required.
- [ ] Update relevant specs and evidence before finish work. No commit, push, install or publish is authorized by this planning request.

## Rollback

Keep old numeric/thumbnail navigation usable at every stage. Undo only this task's hunks if outline failures block loading, notes become projectable, stale callbacks navigate or hidden thumbnails keep scheduling. Never reset whole overlapping files from the dirty baseline.
