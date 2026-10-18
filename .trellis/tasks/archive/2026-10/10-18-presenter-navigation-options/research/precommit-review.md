# Final full-diff precommit review, 2026-10-18

## Scope and verdict

**READY for the coordinator's authorized Chinese commits, three-task archive and push to dev. No unresolved code or validation blocker was found.** This is a full-scope final Trellis check of the parent and both children, not a review of only the last test change. No production/test source was edited during this final pass. The only file authored by this review is this report.

Loaded all three `check.jsonl` manifests, parent and child PRDs/designs/implementation plans, frontend/backend indexes and applicable quality contracts, updated frontend component/type/privacy/persistence rules, prior independent implementation/check receipts and final coordinator verification/native provenance. Reviewed the entire tracked product diff and explicitly inspected all six currently untracked Rust modules:

- `crates/ferrostage-ui/src/page_browser.rs`
- `crates/ferrostage-ui/src/presenter/number_input.rs`
- `crates/ferrostage-ui/src/presenter/page_browser.rs`
- `crates/ferrostage-ui/src/presenter/layout_navigation.rs`
- `crates/ferrostage-ui/src/shell/page_browser.rs`
- `crates/ferrostage-ui/src/shell/layout_navigation.rs`

Scope also includes core getter/tests, all changed UI integration/tests, both locale files, root DESIGN, frontend executable specs, task artifacts and the coordinator's narrowly scoped `.gitignore` change. The user selected keeping the screenshot-embedded planning `interaction-preview.html` local-only; `git check-ignore` confirmed both its current task path and archive path match. Runtime source, screenshots and verification receipts remain eligible for the authorized commit scope.

## Findings (fixed)

No new defect required repair during this final pass. Earlier checker repairs remain present and verified: numeric-editor focus on modal transitions/repeated Jump, plus isolation of that new regression with the existing temporary SaveFixture. The test isolation changes only three lines inside cfg(test); production behavior was not changed after native acceptance.

## Findings (not fixed)

No remaining concrete correctness, security, dataflow or regression issue was identified within the approved task scope. Native rich AccessKit text-run/selection-action and screen-reader qualification remain explicitly documented limitations; they are not represented as tested. No speculative expansion, extra approval flow or size/DPI matrix was introduced.

The coordinator owns archive metadata/path rewrites and will update each curated manifest's source-evidence path after moving all three tasks. Their plan to archive children before parent preserves the explicit dependency/tree bookkeeping. This review did not commit, archive, push or modify task lifecycle metadata.

## Behavior and boundary review

- One contextual host preserves the workbench beneath the browser/menu, with distinct measured anchors, upward placement, viewport clamping and input occlusion. Focus is tracked across surface identities without stealing focus on ordinary repaint. Escape/outside dismissal is consumed before Stop, drawing or navigation.
- Numeric input uses ASCII numeric text with caret/selection/edit/paste and explicit physical-page validation. Edits and density changes do not navigate; submit/tile activation is explicit, deferred and guarded by browser identity/generation/content membership. Independent notes pages cannot become projection targets while excluded.
- Browser density writes the existing paired preference leaves atomically, retains legacy custom settings until explicit edit, preserves unknown TOML leaves and respects read-only/failed-save/Retry behavior. There is no schema migration or document-override persistence.
- Six checked layout choices and two independent checkbox choices reflect effective requested overrides. Auto separately describes the detected result. Menu keys cannot fall through to presentation shortcuts. Successful changes close, identical complete overrides skip document build/core reconfiguration and preserve frames/annotations, and errors retain original effective state/checks with a localized local failure.
- The new all-page annotation-presence query is pure/read-only, includes live laser and nonempty off-page strokes, excludes empty stroke/container artifacts and adds no UI/IO dependency to core. Original core clear-on-reconfiguration policy is unchanged.
- Presenter render uses captured data, preserving synchronous first-frame borrow safety. Deferred input dispatch, Stop/Home/Swap/import/replacement dismissal and audience snapshot-only notes boundary remain intact. Physical navigation changes overlay grouping while notes exclusion remains independent.
- On-demand thumbnail rendering retains one worker and the existing bounded request/cache identity. Closed browser has no retained UI thumbnail surfaces; failures match current exact request pixels rather than stale generation/crop/group data. No dependency, lockfile, vendor source or deny-policy change exists.
- Real warnings remain localized ordinary feedback; booleans and internal Debug layout dumps were removed. Root DESIGN/specs match the actual implementation, including browser identity-based focus and the exact core getter semantics.

## Evidence and source freshness

Recomputed **all 22 entries** in `native/integration-final-reviewed-identities.json`: all hashes and lengths match, with **zero mismatches**. These cover every changed/new production/test source and locale, both real PDF fixtures, current EXE and staged PDFium DLL. The accidental `crates/ferrostage-ui/unused-headless-settings.toml` is absent. No ferrostage process is running.

Current EXE `target/debug/ferrostage.exe`: **43,604,992 bytes**, SHA256 `FBBE35BA06CE8E47BF16003CB6418330236197E2BA99AFF639C0868DDFCA8B53`. Native screenshots used the prior `35D8C54E...` production build. The only subsequent source delta was the documented three-line cfg(test) fixture isolation; the build relinked the EXE, and native provenance correctly preserves both identities instead of relabeling older captures.

Because the checked sources/inputs are unchanged, existing successful gates are reused instead of rerunning identical builds/tests. The latest strict native Clippy and full GPUI suite include the fixture isolation. Default CI and PDFium retention predate only that cfg(test)-only isolation; their exercised production code is unchanged. Re-ran `git diff --check` during this final review: exit 0. Git's existing LF-to-CRLF informational warnings are not whitespace failures.

## Verification

| Gate | Result and receipt |
| --- | --- |
| Lint / format | PASS: default workspace strict Clippy/fmt in `integration-ci-final.log`; final native strict app/UI Clippy in child `check-shell-isolated.log`; post-isolation fmt PASS |
| Default TypeCheck/tests/policy | PASS: `just ci`, **198 tests**, advisories/bans/licenses/sources OK, `integration-ci-final.log` |
| Native TypeCheck/build/GPUI | PASS: `just check-shell`, **159 tests**, 0 failed, child `check-shell-isolated.log` |
| Combined UI strict lint | PASS: gpui/pdfium/test-support all-target Clippy `-D warnings`, `integration-ui-clippy-final.log`; latest fixture-only delta additionally passed native Clippy |
| Focus regression after isolation | PASS: **1 test**, child `check-focus-isolated.log` |
| Real PDFium retention | PASS: **1 test**, verified runtime, `integration-pdfium-final.log`; thumbnail counts `[4,0,16,0]` and bounded retained-frame ledger |
| Full-feature dependency policy | PASS: `integration-deny-all-features.log`; current Cargo/lock/vendor/deny graph unchanged |
| Native functional/visual smoke | PASS within the named normal-window scope, coordinator `verification.md` and `native/integration-*.jpg`; historical browser checkpoints are labeled accurately |
| Final identities/artifact/process/diff checks | PASS: 22/22 identities match, generated settings absent, 0 running app processes, diff whitespace exit 0 |

No CPU/FPS/RSS, exhaustive platform/hotplug, screen-reader, size/DPI or all-theme/locale-matrix result is inferred from these receipts. The verified ordinary-window dark/zh and light/en behavior and automated privacy/input/transaction regressions support the approved implementation acceptance.

## Handoff

Coordinator staging audit: all110 related files are included; the user-excluded HTML is ignored at both active and archived paths. Repository `.gitattributes` deliberately preserves research receipt bytes (`-text`). Raw Cargo logs contain Windows CRLF, diagnostic-art padding and terminal blank lines, so they are retained byte-for-byte rather than formatted as authored source. Staged code and authored documents pass `git -c core.whitespace=cr-at-eol diff --cached --check -- . ':(exclude)**/*.log'`; raw log whitespace is excluded from this formatting-only check, not from review or commit. No product check or warning policy was waived.

Only this report was added in this final review. No source fix or additional build/native run is needed. Coordinator may proceed with the already authorized related-change commits, task archives/path bookkeeping and dev push, then verify the staged/final Git state and remote result. The intentionally local-only planning HTML must remain excluded.
