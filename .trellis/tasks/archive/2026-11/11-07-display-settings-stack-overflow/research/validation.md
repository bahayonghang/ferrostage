# Final validation

## Outcome

Resolved on current `dev` source. All functional acceptance criteria are met. The user confirmed the proposed local commit and Trellis archive, explicitly prohibiting push. Work commit, archive and journal identifiers are recorded in Git history and the session journal after this receipt. No push is authorized.

## Evidence

- Root cause and source-bound compiled stack reservations: `stack-diagnosis.md`.
- New Windows 1 MiB subprocess regression: observed RED before production edits, GREEN after repair. Final rerun after adding its Windows-only condition: 1 passed, 274 filtered out, 0.91 seconds.
- Full GPUI suite: 275 passed.
- Strict GPUI/app clippy with test-support and product `gpui,pdfium`: PASS, no warning suppressions.
- Default `just ci`: exit 0; fmt check, workspace strict clippy, 253 default-feature tests and cargo deny passed. Cargo deny retained nonfatal duplicate/unused-license-exception warnings; advisories, bans, licenses and sources all reported OK. No policy changes.
- Initial CI run hit Windows executable file-lock error 5 because the main session concurrently launched the app; rerun passed after normal native exit. This was a check-order error, not a source failure.
- Final normal-debug native `just dev`: both display hosts, real three-monitor topology, details, General return, Cancel and title-bar close PASS; process exit 0 and no late diagnostics. See `native-reproduction.md`.
- Independent source review: no actionable findings, `review.md`.
- Final diff whitespace check: PASS.

No physical hotplug, settings allocation changes, window/DPI matrix or performance qualification was performed. Other measured large builders have no established matching failure and remain unchanged, as documented in the sibling sweep.
