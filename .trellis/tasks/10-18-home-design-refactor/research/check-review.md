# Independent check review

Reviewed 2026-10-18 against task PRD/design/implementation plan, curated frontend specifications and the complete working-tree change, including the new untracked home view. Baseline HEAD: `595910cc41d6259a3e87766c2aceb2c505d28359`.

## Findings (fixed)

- File: `crates/ferrostage-ui/src/shell.rs:4892`.
  Issue: the implementation's geometry matrix checked truncated rows, and its keyboard test checked a short focused path, leaving a long unbroken focused path's horizontal containment untested.
  Fix: added `home_focused_long_path_wraps_inside_minimum_width`, exercising actual Tab dispatch with three recents and a long mixed Latin/CJK path at 320x240. It verifies the full-path block stays inside client width and wraps to multiple lines. The focused check and full 126-test GPUI gate pass. No production correction was required during independent checking.

The implementation agent had already fixed the reproduced minimum-height failure where keyboard focus reached a recent row at y=294..346 outside the 240px client. Final code retains home-only focus handles and GPUI scroll anchors. Its regression asserts rendered focused-row bounds after the scheduled scroll frame. The checker observed this pre-freeze failure while reading `implement-keyboard-test.log`; that file was subsequently overwritten by the passing focused rerun. The observed failing excerpt is retained below, not presented as an intact original log:

```text
thread 'shell::tests::home_keyboard_reaches_recents_and_restores_modal_focus_at_minimum_size' (59060) panicked at crates\ferrostage-ui\src\shell.rs:4879:9:
focused row must scroll into view: Bounds { origin: Point { x: 16px, y: 294px }, size: Size { 288px × 52px } }
test shell::tests::home_keyboard_reaches_recents_and_restores_modal_focus_at_minimum_size ... FAILED
```

## Findings (not fixed)

1. **Required all-features policy gate fails on existing dependencies.** `cargo deny --all-features check` returns 1 with three **unmaintained** advisories. This report does not classify them as demonstrated vulnerabilities:

   | Baseline package | Advisory | Baseline lock source location |
   | --- | --- | --- |
   | `zbus_names 4.3.4` | RUSTSEC-2026-0371 | `Cargo.lock:7621` |
   | `zvariant 5.15.0` | RUSTSEC-2026-0370 | `Cargo.lock:7837` |
   | `zvariant_derive 5.15.0` | RUSTSEC-2026-0372 | `Cargo.lock:7853` |

   Gate output traces the optional Linux graph through `gpui_platform 0.1.0 -> gpui_linux 0.1.0 -> accesskit_unix 0.22.1 -> accesskit_atspi_common 0.19.1 / atspi-common 0.13.0 -> zbus_names / zvariant`; `zvariant_derive` is used by `zvariant`. The tagged GPUI packages remain at Zed `v1.23.3`, revision `96a1b1b5389efff473fc2306f7f8c690f2e1a417`. These are all-target dependency-policy results, not proof those Linux paths execute in the Windows home.

   Baseline proof: `git diff --exit-code HEAD -- Cargo.toml Cargo.lock deny.toml ':(glob)**/Cargo.toml'` returned 0, including all tracked package manifests. `git show HEAD:Cargo.lock` independently confirms the versions above, plus `accesskit_unix 0.22.1` and `accesskit_atspi_common 0.19.1`. See `check-baseline-dependencies.txt` and exact diagnostic chains in `check-deny-all-features.log:5051`, `:5098`, `:5154`. This establishes unchanged dependency inputs, not that a historical advisory database necessarily returned the same result.

   Left unchanged because maintained dependency migration and policy changes are outside the home task's approved boundary. Do not add an advisory/lint waiver or substitute the GPUI family. Recommendation: treat this as a separate maintained-dependency repair decision. Keep this gate FAIL and AC10 incomplete until resolved.

2. **Native acceptance remains separate and partly unexecuted.** Main session owns final screenshots/input receipts and the source-bound native matrix in `verification.md`. OS accessibility Click invocation and unexercised 150%/200% OS scale/increased-text states remain NOT_RUN. Source callback registration and headless geometry do not establish these states. The initial baseline executable's correspondence to source was not independently established, as recorded in `native/baseline-session-notes.md`; do not describe its screenshot as a fully source-bound before/after pair.

No other confirmed product-code, specification-compliance, scope or architecture issue was found in the reviewed diff. Task remains `in_progress`; this review does not claim complete native acceptance or an entirely green required-gate set.

## Verification

| Gate | Result | Receipt |
| --- | --- | --- |
| Lint / formatting | PASS, default and native strict clippy with `-D warnings`; rustfmt | `check-ci-final.log`, `check-shell.log` |
| TypeCheck / native build | PASS, `gpui,pdfium` app build | `check-shell.log` |
| Default workspace tests | PASS, 187 tests across workspace targets | `check-ci-final.log` |
| GPUI regressions | PASS, 126 tests including input/lifecycle/workbench and new home regressions | `check-shell.log` |
| Focused long-path test | PASS | `check-focused-path.log` |
| Default dependency policy | PASS, advisories/bans/licenses/sources | `check-ci-final.log` |
| All-features dependency policy | FAIL, three unmaintained advisories; bans/licenses/sources PASS | `check-deny-all-features.log` |
| Whitespace | PASS, `git diff --check` | `check-diff.log` |

The first CI attempt failed on sandbox mbx-cache/home permissions and PowerShell profile policy (`check-ci.log`). An escalated attempt then collided with the native QA process's executable lock (`check-ci-elevated.log`). After coordinating window closure, the uninterrupted final CI passed. These environmental failures were not treated as product failures. Process-scoped `PSExecutionPolicyPreference=Bypass` was used; no global policy was changed.

Final native app SHA256 after `just check-shell`: `2C109A1CC8CC944DE2AB3F11F24FC0C605E1921B6E6480E404A2C8E739DB3E9A`. Production sources, locale catalogs, DESIGN and test hashes are recorded in `check-source-sha256.txt`. Main must rebind native receipts if any production source changes afterward.

## Shared consumer and scope review

Source-site inventory: **checked 49 / changed 3 / left 46**. Counts refer to visual consumer call sites, not dynamically repeated controls. The three new home activation faces are Open, recent row and utility builders. The existing 46 generic-button call sites retain their visual geometry: presenter 10, workbench 15, settings 14, displays 7. The shared extraction retains stable IDs, disabled/tab guards and deferred mouse/Enter/Space/accessibility dispatch. Home focus anchors apply only on Idle; document-stage event routing and faces remain unchanged.

Confirmed source boundaries: only UI view/theme/tests, two locale catalogs, root DESIGN and task/spec documentation changed. No core/settings schema/PDF/media/audience/dependency changes. Recents retain `.take(3)`, order and original `Intent::Import` paths without filesystem reads. `can_open` is the common guard. Common error/topology/save feedback and Retry remain inside home scrolling. Ready/Presenting keep their header/timer/footer and workbench composition.

The existing contrast tests already cover home ink/body/muted across canvas/soft/card, focus boundaries and primary action states. Locale catalog key/interpolation parity passes. The specification's new home/focus guidance agrees with the final implementation.

## Acceptance mapping

| Criteria | Evidence and limit |
| --- | --- |
| AC1-2 | Idle-only native title/chrome branch, dedicated brand, locale copy and actual Idle-tree absence assertions. Provisional normal Chinese/dark native screenshot independently viewed; final locale/theme receipts belong to main. |
| AC3-4 | Actual GPUI layout matrix across all 11 planned geometries, both locales/themes and 0/1/3/4 supplied recents. Open face 120x44, three-entry cap, bounded rows and <=8px gap; focused full path wraps at minimum width. Native geometry/font acceptance remains separate. |
| AC5 | Count/order/path/guard source review and existing import transaction regressions pass. Native valid/missing recent acceptance belongs to main; no fabricated metadata is rendered. |
| AC6 | Actual Tab/Shift+Tab, Enter/Space, disabled controls, mouse dispatch and modal recovery regressions pass. OS accessibility invocation NOT_RUN. |
| AC7 | Default-scale geometry/keyboard/contrast checks pass, including focused minimum-height scrolling. Native full matrix and higher OS/text scaling cannot be inferred from those checks. PARTIAL until main's explicit receipts/gaps. |
| AC8 | Feedback retained; existing chooser owner/cancellation, busy/import authority, persistence failure/Retry and focus recovery tests pass. Physical/native error and topology cases retain their separate evidence limits. |
| AC9 | Existing header/notes geometry, timer settings, direct navigation, Home/recents and audience closure regressions pass; no corresponding production logic changed. Native document-state screenshots remain main-owned. |
| AC10 | Build/lint/tests PASS; all-features advisory gate FAIL, native evidence partly unexecuted. Overall AC10 is not PASS. |

Visual review of `native/zh-cn-dark-3-1100x760.jpg` supports the warm editorial utility direction at that one provisional state: a single distinct wordmark, quiet utilities, natural-width clay Open and flat left-aligned recent files. No generic cards, fake data, Idle clock/footer or visible normal-width clipping appeared. This single screenshot cannot establish narrow, light-theme, English, focus or DPI acceptance.

## Subsequent user scope correction and final native receipt

The user explicitly stopped size/DPI and similar non-core verification and requested a global memory preference. Those originally planned native matrices are no longer required or blockers. Do not resume them without an explicit request. Main subsequently verified the final source-bound normal-window Chinese/dark home, real Settings open/close with focus restoration, Open/chooser cancellation, focused recent full path, actual recent-PDF import into Ready, and return Home. See `verification.md` and its native captures. The all-features dependency-policy failure remains separate and unchanged; no claim of an entirely green gate set is made.
