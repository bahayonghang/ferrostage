# Independent review, stage 1

2026-10-19. Scope requested by parent: completed fullscreen/layout and original Notes work only. Timing integration and transition implementation were still being edited by their owners. No shared production files were changed by this reviewer.

## Findings (fixed)

None. No concrete product blocker was found within this stage's inspected source paths.

## Findings (not fixed)

1. **Specification synchronization remains required before closeout.** `.trellis/spec/frontend/state-management.md:33` still places current Notes above Next; the approved implementation deliberately puts Next above Notes. `component-guidelines.md` retains the previous fixed session geometry wording, and root `DESIGN.md` needs the matching final hierarchy/lifecycle review. The layout worker already records this handoff. Parent owns final integrated spec updates, so this reviewer did not create concurrent edits.
2. **Task documents contain obsolete approval status.** The fullscreen child PRD says implementation is not approved, and the Notes child says execution awaits final approval. The parent PRD and explicit user instruction establish that approval has already occurred. These are documentation defects, not missing user permission. Parent should normalize them during task closeout.
3. **Native acceptance remains a separate pending gate.** Source and headless assertions cannot prove Windows normal/maximized restore, assigned dual-display output, final Notes readability while standing, or scroll interaction in the rebuilt program. Parent is performing native smoke. Do not mark these passed based on this review.

## Inspected behavior

- `shell/presenter_fullscreen.rs` captures the current window identity, role-bound monitor/display, bounds and native mode before requesting fullscreen in place. Start's existing transaction additionally waits for `start_presenter_confirmed`; requesting a toggle is insufficient.
- Cancellation and Stop initiate one restore; the busy guard blocks a new Start during restoration. A queued Windows entry is observed before issuing one exit, preventing early Stop from accepting the still-windowed pre-entry state. The maintained Windows implementation queues the toggle and retains style/outer bounds, consistent with this mechanism. Real maximized restoration still needs native evidence.
- Confirmed presenter replacement discards the obsolete restore identity; unsafe missing-display recovery uses the existing role policy. No notes were added to the public audience input by the layout work.
- Fresh right-column ratio is 0.40; existing persisted values remain intact. Vertical dragging preserves the meaning of `notes_ratio` and accounts for divider/gap height. Notes images use their raster aspect ratio at available width with no flex shrinking; one existing vertical scroll owns overflow.
- More/Drawing use the existing contextual host, deferred actions, focus handles, disabled guards and Escape precedence. No new Notes mode/font controls were introduced.
- External thesis changes are restricted to the four recorded files: durable sync template, generated driver, focused regression test and slide workflow instructions. The template's 16pt/20pt typography is locally grouped, expands only note margins, and retains the label hook after definition. Note payloads and audience sources are not modified.
- Reviewed the local analysis receipt and independently viewed the dense page 156 Poppler image. It contains the complete ending with bottom clearance and readable paragraph separation. The receipt distinguishes two stale pre-existing PDF notes from the unchanged current source payloads, and reports zero current-source text differences and zero audience raster differences. This reviewer did not claim a new independent all-95-page visual sweep.

## Verification

- Lint / type-check: PASS for independently run `cargo clippy -p ferrostage-settings --all-targets --locked --offline -- -D warnings`.
- Whitespace: PASS for independently run `git diff --check` in both repositories.
- UI full lint/type-check: deferred until active animation integration settles; not a stage-1 PASS. Layout implementation receipt records its earlier successful type-check and 186-test UI checkpoint, plus a subsequent 15-test fullscreen rerun; shared transition lint was incomplete then.
- Tests: existing layout/settings/Notes receipts reviewed, not redundantly rerun because no product finding required a regression and shared source was actively changing. Final integrated tests remain mandatory.
- Native: parent-owned and pending for this report. No native success inferred from headless geometry or PDF render measurements.

Next review: integrated transition/scheduler event identity, first paint versus settlement timing, interruption/retirement, budgets, Notes exclusion, final strict gates and version/distribution consistency.
