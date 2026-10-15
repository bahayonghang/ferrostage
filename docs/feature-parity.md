# Feature parity

> Current2026-10-15 user revision: Gate B implementation PASS on verified V16 evidence; original extended hardware/native qualification DEFERRED_BY_USER.03/07 are eligible for implementation-scope archive,08 is unblocked. See `.trellis/tasks/10-14-presenter-v0.3/acceptance-policy.md`; older incomplete/blocking statements below are historical under the previous policy. No old test outcome is changed.

The behavior target is a dual-screen PDF / Beamer presenter. Priorities match [docs/PROJECT_PLAN.md](PROJECT_PLAN.md) §1 and §3. Nothing below is a port of another project's source. The table records source and scoped evidence through V16 on 2026-10-15, not completed parent acceptance. Task07 has verified core native behavior; its full native acceptance and Gate B remain incomplete.

| Feature | Priority | Current source |
| --- | --- | --- |
| Open a PDF (CLI, dialog, drag-and-drop, recent files) | P0 | Task07 has CLI/dialog/drop/recent routing. CLI, native valid/error recent imports and V6 successful native chooser import observed; PDF bytes unchanged. Drag/drop remains unverified |
| Session lifecycle (Ready → Start) | P0 | Core Idle/Ready/Presenting, audience availability guard, injected-time timer, and Curtain projection verified in task 04. Native window transaction remains task 07 |
| Presenter and audience windows | P0 | GPUI links through the optional feature; task07 source and automated checks are independently reviewed. [ADR-008](adr/ADR-008-gpui-dependency-gate.md) records the original five maintained packages, separately approved four-file AccessKit repair as the sixth package, and unchanged three exact license exceptions. V16 product checks and bounded native placement, Start/Stop/restart, modal routing and owner/Audience close pass. Gate B remains incomplete for physical topology and the remaining native matrix |
| Monitor roles / three-screen | P0 | Core IDs/roles, pure resolution, complete Windows enumeration and cancellable polling are implemented; static three-screen probe passed. GPUI A/B/C placement observed in07; port-change/hotplug timing and09 role UI remain pending |
| PDF raster and render scheduling | P0 | Task05 single PDFium worker, owned BGRA, same-density crop goldens, candidate transactions and bounded CPU caches independently verified. Local 4K CPU P95 including retained frames is 12.35–13.96ms on the recorded fixture/environment; GPU upload remains task08 |
| Next / previous page | P0 | Implemented in `ferrostage-core` |
| Speaker notes | P0 | Task06 owns ordered per-content notes mappings, exact label matching and orphan diagnostics; notes remain outside the snapshot. Presenter rendering remains task08 |
| Beamer overlay steps | P0 | Task06 content-only SlideMap groups contiguous nonempty original labels, with physical-pages override. Real Beamer fixtures tested; repeated producer labels remain an explicit heuristic limitation. Independent review passed |
| Beamer notes crop and manual override | P0 | Task06 implements five layouts, ratio candidates with warnings, geometry checks and document-local overrides; actual Beamer left/right crops tested. Runtime viewing controls remain task08 |
| Blank black / white | P0 | Core state and07 audience black/white output implemented; native black on A and white on C observed |
| Screen swap | P0 | Core routes the intent to the app coordinator without changing page/crop. Task07 implements confirmed replacement-window migration and settings persistence. Native page/blank continuity, actual locked-file failure, single unsaved feedback, successful Retry/read-back and saved-role relaunch observed |
| Settings system + UI | P0/P1 | Task02 provides the v1 eight-section schema, `LoadOutcome`, `SettingsStore`, startup persistence and read-only `check-config`. Task07's minimal display panel applies and persists roles with scoped native evidence. Task09's complete settings categories and workbench consumers remain unimplemented |
| Jump preview | P0 | Core preview navigation is isolated from the committed audience page until Commit; regression tests pass. Task07 minimal native jump editing/commit/cancel observed; full workbench controls remain task08 |
| Claude visual theme | P0/P1 | Root `DESIGN.md` and task 07/08/09 plans define shared light/dark tokens and control states. Task07 shared theme/locale implemented; native light/dark/system initial appearance, Chinese fallback and en-US captured. Disappearing-control focus repair passed V6 native retest; full logical-client size/state matrix and live OS theme event remain pending |
| Laser pointer | P1 | Position is stored; no window draws it. Session memory only |
| Pen strokes | P1 | Points are stored; no window draws them. Session memory only |
| 1–16 page thumbnail grid | P1 | Task05 implements fixed-height180px thumbnail requests and bounded L3 cache. No grid UI |
| Hot reload | P2 | Policy constants and generation check; watcher loop not started. External PDF replacement only |
| Video / GIF | P2 | Task06 reads a documented Movie/Screen/RichMedia metadata subset with diagnostics; no playback. Shared-frame backend and runtime integration remain task11 |


## Display-selection implementation (2026-10-15)

Task07 implements explicit display assignment: detected count/number list, matching Identify, application/PDF destination selectors and Apply/Cancel. V16 native evidence verifies all three numbers, actual placement, conflict/Cancel, migration preserving page/white/running timer, no-play timer freeze and saved-role relaunch. Task08 will reuse this surface; task09 will integrate complete settings/topology. Physical single/zero-display and hotplug acceptance remain NOT_RUN; the size matrix is user-deferred. Gate B and all P2 requirements remain unchanged. See task07 `research/native-v16-core-results.md` and `research/native-v16-independent-review.md`; historical Swap/configuration receipts alone do not prove the new workflow.
