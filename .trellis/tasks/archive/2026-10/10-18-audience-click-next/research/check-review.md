# Independent implementation review

Reviewed 2026-10-18 against approved PRD/design/implementation plan, curated check manifest, full scoped diff and both new spec sections. No product behavior defect found. No core, vendor, dependency, public projection or task metadata changes made by this reviewer.

## Findings fixed

- `crates/ferrostage-ui/src/shell.rs`: the existing new presenter-isolation assertion clicked client corner `(2,2)`, which establishes chrome isolation rather than the required real slide-preview isolation. Added `audience_click_does_not_bind_the_presenter_slide_preview`: a completed click at the rendered `surface-Current` center in Presenting leaves the page unchanged; clicking that same Shell's Audience advances to physical content page 2, skipping the notes page. Test-only change; product executable did not need rebuilding.

## Findings not fixed / limits

- Full-feature dependency audit FAIL: existing unmaintained advisories `RUSTSEC-2026-0371` (`zbus_names` 4.3.4), `RUSTSEC-2026-0370` (`zvariant` 5.15.0), `RUSTSEC-2026-0372` (`zvariant_derive` 5.15.0). All originate in the existing Linux/AccessKit/GPUI dependency graph. Cargo.lock, manifests, vendor sources and deny policy are unchanged. Dependency migration is outside this input-feature task; no waiver or dependency edit was introduced. Bans/licenses/sources pass. See `check-deny-all-features.log`.
- Reload mouse suppression has source-path evidence (`Shell::busy` calls `reload_busy`, which tests `reload.pending.is_some`) rather than a new reload-specific input fixture. Actual mouse tests exercise loading, asynchronous preferences and window transitions; reload internals are private and its existing integration module requires PDFium. No extra accessor, privacy relaxation or heavyweight duplicate fixture added for this one-line established busy contract. Do not describe this as an executed reload-specific mouse test.
- Native activation/fullscreen/windowed interaction verification is parent-owned and was NOT_RUN by this reviewer. Headless evidence does not establish Windows first-click activation or placement.

## Data-flow review

- Optional schema-v1 bool defaults true; parser diagnoses the full leaf; known-key merge preserves unknown display/monitor metadata. Explicit false, future-version read-only, save failure and retry have executable coverage.
- Displays field uses the existing checked control, localized label/help and keyboard/pointer behavior. The comparison copy normalizes only the new non-placement leaf and the prior startup-only leaf. Real display edits still enter the transaction, with a combined-edit regression.
- Audience keeps only snapshot input and private GPU resources. Full-size stable root accepts completed mouse-left clicks and emits a payload-free event. Framework hit-testing cancels outside release; no synthetic keyboard binding was added.
- Shell defers consumption beyond the source-window lease, then rechecks committed handle, Presenting stage, effective settings, all modals, established busy state, Live projection, published blanking and core blanking. Accepted input calls existing `dispatch(Intent::Next)` once, preserving NextStep, media/annotation, timer and matching-frame publication paths. Pending/retired/stopped windows are rejected. Presenter pen/laser tools remain independent.
- Both new spec sections match current code and make no native acceptance claim.

## Verification

| Check | Result | Evidence |
| --- | --- | --- |
| `just ci` | PASS | `check-ci.log`; fmt, strict workspace clippy, all workspace tests including 44 settings tests, default deny |
| GPUI/PDFium product build | PASS | `check-product.log`; build completed in 8.65s |
| Product all-target strict clippy/type checking | PASS | `check-product.log`, rerun after test addition in `check-lint-final.log` |
| UI test-support suite | PASS | Initial 142 tests in `check-product.log`; final 143 tests, 0 ignored/failing in `check-ui-final.log` |
| Final rustfmt and `git diff --check` | PASS | Executed after review test addition; Git only reports normal LF-to-CRLF conversion notices |
| Full-feature offline deny | FAIL, existing dependency policy | Three exact advisories above; `check-deny-all-features.log` |
| Native smoke | NOT_RUN by reviewer | Parent owns this separate gate |

Initial sandboxed CI failed before compiling because mbx could not write its managed cache/session paths. Authorized elevated retry passed; this was an environment permission issue, not a product failure. No global execution policy was changed.

Product executable freeze: `target/debug/ferrostage.exe`, SHA256 `0881EFAF1F00F9F54B8AC13EDBC265A2A451D2BC564DEA04270EF338A97E3060`. Parent was notified when all executable-building work finished; later changes affect tests only. No commit, push, archive or app launch performed.
