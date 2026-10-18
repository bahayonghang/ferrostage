# Execution plan

## Activation

- [x] Obtain task creation consent and clarify direct navigation.
- [x] Inspect screenshot/log, design source, code and relevant specs.
- [x] Persist requirements, technical design, execution plan and context manifests.
- [x] User approved final artifacts and implementation on 2026-10-17; task activated.
- [x] Dispatched trellis-implement with explicit product/test ownership and child-side context fallback. Child implements directly without recursively spawning implement/check agents.

## Ordered work packages

1. [ ] Baseline real native build with repository 100-page Beamer fixture. Record revision, copied settings, runtime/fixture identity, viewport, import stage, page/timer behavior; investigate unsolicited advance.
2. [x] Implement direct core navigation and migrate actions/shell/grid/page input. Remove obsolete preview state/UI. Test Ready/Presenting, invalid/notes-only/boundary pages, annotations, Escape/focus and exactly-once submit.
3. [x] Update DESIGN.md before theme/button/layout/locales. Move notes above next; fix drag geometry and legacy ratios; remove strip and workbench recent files. Adjust related settings UI and compatibility tests without touching user settings.
4. [x] Remove hidden thumbnail scheduling and test real PDFium closed/open/closed-grid work/retention. Retain notification cadence because deep frame equality would introduce new work; no CPU/FPS claim.
5. [x] Default logging to warn with diagnostic override and visible errors. Add isolated filter behavior tests and update logging spec.
6. [x] Run focused and integrated gates and independent full-diff trellis-check; task-local findings repaired. Default CI, final native build/lint/123 GPUI tests and 40 PDFium tests pass. Full-feature policy remains blocked by three unchanged dependency advisories; see research/verification.md.
7. [x] Record automated/native resource evidence and update relevant specs. User accepted the result and waived further visual checks on 2026-10-17; screenshot cases remain NOT_RUN. Full-feature advisory failure remains explicit and prevents claiming all gates passed.

## Closeout

- Native visual acceptance: user accepted; no more screenshot work requested.
- Commit: user approved the exact commit plan on 2026-10-17; included in the local work commit.
- Existing full-feature policy failure: retained without dependency changes or waivers; this task is not a full-policy release qualification.
- Archive: not performed while the required policy gate remains unresolved.

## Expected ownership

- Implement agent: affected core command/state/tests; UI actions/shell/transactions/presenter/workbench/theme/settings panel and locales; settings default/compatibility tests; app logging/tests. Read design and specs before edits.
- Coordinator: root design decisions, task artifacts, cross-layer review, spec reconciliation and evidence handoff. Avoid concurrent edits to agent-owned files.
- Check agent: integrated review and focused self-fixes only after implementation completes.

## Quality commands

```text
cargo test -p ferrostage-core -p ferrostage-settings -p ferrostage-ui --lib --locked
cargo test -p ferrostage-ui --features test-support --lib --locked
just ci
just check-shell
cargo deny --all-features --locked --offline check
just test-native
```

Resolve the existing verified PDFium library and set FERROSTAGE_TEST_PDFIUM for native tests. Never download an arbitrary DLL or silently skip runtime tests. Run focused checks after relevant changes and full gates after integration; repeat only for new changes/failures. Preserve exit codes and concise logs.

## Validation matrix

| Area | Cases |
| --- | --- |
| Manual presentation | Import Ready/no audience/no timer; ticks before Start and beyond target after Start do not navigate; explicit Next/Previous do. |
| Jump | Grid click, valid submit, partial/invalid text, notes-only page, boundaries, Escape, Ready/Presenting; no hidden preview or second confirmation. |
| Notes | Real split-page and separate notes, no notes, freshness after jump, audience isolation. |
| Layout | Notes above next with dominant initial area, persisted divider, no strip, recent only in Idle, on-demand grid. |
| Controls | Both themes; semantic roles, selected/disabled/hover/pressed/focused, keyboard and long zh/en labels. |
| Render | Zero closed-grid thumbnails despite legacy true; bounded open-grid work; closing retires frames; unchanged scheduling submits nothing new. |
| Publication if changed | Equal snapshot quiet; PDF/media/stroke/blank/staging/error changes visible, timer updates at displayed-second boundaries. |
| Logs | Default routine silence, genuine errors, RUST_LOG override and native shutdown diagnostics. |

Native sizes: 1440x900, minimum supported width, minimum supported height and their combination. Probe DESIGN.md's 800x600, 390x844 and 320x640 only where native shell permits; record actual logical client size and notes/next bounds. Make a bounded capture attempt and report unavailable automation precisely.

Performance comparison uses the same fixture/settings/viewport before and after: requested/submitted raster counts and retained bytes, plus idle CPU/paint and navigation latency when measurable. Native raster latency claims require project benchmark methodology (20 warmups, 200 samples and fixture/runtime identities). Counters alone are not a speed claim.
