# Chapter outline implementation review

## Scope and baseline

Reviewed against this task's approved PRD, design, implementation plan and curated check context. The working tree already contained separate defense-experience, transition, playback and presenter changes. Those existing changes are not attributed to this task and were not reverted. No commit was made.

Backend scope: `ferrostage-pdf/src/structure/outline.rs` and its synthetic tests, outline integration in `DocumentStructure`, `ferrostage-document/src/outline.rs`, builder mapping/current-section selection and their tests. Frontend review covers the directory's data snapshot, guarded Shell activation, visible view state, input/focus routing, retirement and scheduling of thumbnail work, locale copy and audience separation.

## Findings (fixed)

- File: `crates/ferrostage-ui/src/presenter.rs`
- Issue: renewing the browser identity during a directory tab switch overwrote the actual invoker with `progress_focus`, violating AC3 for a browser opened from the separate Page browser button.
- Fix: reported to the frontend owner during exclusive ownership. The owner now preserves `return_focus` for same-open browser identity changes, retaining the existing progress fallback only when entering the browser from another contextual surface. Reviewed the fix and real-input `outline_reveals_current_wrapped_title_and_restores_original_invoker` regression, which starts from the separate `grid` invoker, reveals the final current row and verifies Escape returns to the saved invoker.

No backend defect found. No speculative cleanup was applied.

- File: `crates/ferrostage-ui/src/presenter/page_browser.rs` and `presenter/number_input.rs`
- Issue: the newly added real Tab regression exposed that intercepted raw contextual keys did not reach the previous bubble-stage handlers reliably.
- Fix: the frontend owner moved directory Tab/Shift+Tab traversal into capture-stage `outline_key`, retained number-editor capture handling and dispatches the existing `ActivateButton` action for non-row Enter/Space. Removed the temporary per-button raw handler. Reviewed the final source: row activation is handled and consumed once in capture; number-editor keys retain their own route; other buttons retain shared enabled/deferred activation.

## Findings (not fixed)

No remaining concrete source defect found. No design/public-interface or out-of-scope finding was left unresolved. The user manually verified the feature and asked for no further optional validation. The coordinator owns mandatory broad gates and any authorized commit/finish work; this reviewer stopped additional testing accordingly.

## Reviewed contracts

- PDF tree traversal preserves source order and duplicate targets, descends only to structural depth two, tracks indirect identities and applies 4096-node/row and 32-hop ceilings. Invalid title/target metadata retains readable children and never makes outline failure a page-loading error.
- Local destinations resolve page references through actual page-tree identities. Unsupported/external actions remain disabled. No action execution or action-chain traversal is introduced.
- Content captions use the actual `SlideMap` steps. Binary search is sound because `SlideMap::new` enforces strictly increasing steps. Recognized notes retain their original target and cannot be redirected to their owner. Option changes and reload rebuild mapping.
- Current-section selection is based on committed physical page, greatest eligible target, then greatest depth, then earliest source id. Focus and numeric draft state do not feed this calculation.
- Shell activation resolves a current entry id only after matching browser id/generation and Outline view, then delegates to the existing content-membership-checked submission path.
- View changes renew browser identity, retire thumbnail frames/targets/pending work, and disable grid requests. The scheduler retains previous requested keys until its next comparison so queued work receives an epoch cancellation. Presenter texture retirement follows renewed browser identity.

## Verification

- Lint: PDF/document strict all-target Clippy passed per backend implementer. Workspace/native broad gate results belong to the coordinator's final receipt; no duplicate Cargo run was launched by reviewer.
- TypeCheck: backend targeted tests/Clippy passed. Native application build completed successfully in `research/native-build.log`; final post-fix workspace/native gates belong to the coordinator.
- Tests: source-reviewed parser and mapping regression coverage. Independently compared the Rust probe's 39 TSV rows against the original Python PDF probe: every authored title, depth and zero-based physical destination matches, and every parsed target issue is `None`. The backend implementer reports all 36 PDF/document tests and strict all-target Clippy passed; broad execution receipts remain pending coordinator.
- Focused UI tests: frontend owner reports `cargo test -p ferrostage-ui --features test-support --lib --locked outline_ -- --nocapture` passed, 5 passed, 205 filtered, 1.00 s. Source-reviewed coverage includes Ready/Presenting activation, keyboard traversal/numbering, disabled/cancelled/stale input, numeric notes guards, thumbnail retirement/restored budget, wrapped current-row reveal and original Grid-invoker focus restoration.
- Native acceptance: user manually verified and waived further optional validation. Do not infer a full native/theme/locale or dual-display matrix from this or from the headless tests. Final native evidence remains the coordinator's responsibility.

Review is complete based on the final implementation source and available evidence. No additional tests/builds were run after the user's stop-validation instruction. Mandatory broad gate results should be read from the coordinator's final receipt.
