# Independent V14 source and evidence review

Status: **PASS for the bounded source repair and automated checks; native V14 remains pending.** No product fix was required by this review. V13 F12 remains a recorded native FAIL until Root completes the new frozen-build retest. Gate B is INCOMPLETE; this report does not authorize progression to08.

The reviewer explicitly read the V14 implementation, source delta, corrected red/green receipts, latest implement tail, Root's break-loop analysis and native plan, the changed source/contracts, and the existing native evidence applicability record. Source/spec/config/build files were not modified. Only new independent research artifacts were created; Root retains desktop and progress-document ownership.

## Findings (fixed)

None. Independent strict lint, explicit typecheck and the related real GPUI regressions pass without source edits.

## Findings (not fixed)

No blocking source defect was found in the dispatched V14 delta. The following acceptance boundaries remain open and are not silently converted to PASS:

- Actual light320x640 scrolled Displays/titlebar Close must be repeated on V14, with full post-exit logs and final window inventory. V13's exit0 and empty windows did not prevent its late hover error. Other callback-dependent owner/nonowner/panel closures remain part of Root's planned runs.
- Accessibility's Send producers and detached foreground channel receivers are a distinct scheduling boundary. The pinned Windows adapter retains activation/action handlers; receiver loops still update a window handle. TestWindow's default a11y_init is a no-op, so the17-callback tests do not cover this mechanism. This audit does not establish that these receivers cannot outlive an owner or prove their shutdown safety. No equivalent executable failing case or actual F12 attribution was established, so speculative accessibility redesign was not added to this bounded correction. If an accessibility retirement symptom occurs, establish producer/receiver reachability and an executable regression before changing ownership.
- The exact dark1440x900, dark800x600 and dark390x844 observations remain feasible pending work, as do narrow Presenting/Stop, panel keyboard Cancel/returned focus and a stable hover observation. These are not external blockers. Physical/true single-zero topology, live OS theme/drag tooling limits and transient/cursor proof retain their separately recorded NOT_RUN or UNVERIFIED states.

## Lifetime and regression assessment

Window owns one Rc token immediately before platform_window in field order. There is no custom Window Drop implementation that changes this order. Rust field teardown releases the token before invoking the native platform owner's destructor. Pinned WindowsWindow::drop can retain its inner Rc while native destruction waits for dialog retirement, explaining why a retained platform callback is not proof that its GPUI Window still exists.

All17 reviewed registrations use Weak references to that same token: frame, viewport, insets, resize, moved, appearance, button layout, active, visibility, hover, input, hit-test and five tab operations. No upgraded strong owner is retained across an update. They return before accessing a retired handle. Appearance additionally rechecks inside its scheduled body, covering enqueue-while-live followed by owner removal. Input and hit-test preserve the existing DispatchEventResult::default()/None fallback; the frame callback does not issue a new wake when entered after owner removal. Normal live update/logging bodies are retained.

The on_close path remains deliberate idempotent teardown and system-tab cleanup, rather than an ordinary live-window event. First-party should-close deferral, current/pending/retired ownership, Shell poll/read/chooser cancellation and transaction semantics are unchanged. This adds no second display-role resolver, teardown policy, settings owner, dependency family or logging filter.

The new tests retain the actual TestWindow, remove the actual GPUI Window, then invoke registered platform callbacks. They are not source-string checks. Corrected red-final records4 failing groups and1 passing live control with19 removed-window errors; green runs the same5 tests successfully. Early visibility compile failures and the mistaken input-propagation assertion are preserved separately and are not represented as product red evidence.

Live visibility, client geometry and appearance are read back after callbacks. A live Window leased out of the App slot deliberately triggers a visible update error, proving that the token guard does not erase live errors. The lease helper releases the App RefCell borrow before callback dispatch and restores the slot on the normal tested path; it is test-support-only. Scoped tracing capture remains alive through scheduled test execution and replaces the shared global capture, preserving the V8 assertion without cross-test buffers.

## Verification

| Check | Evidence and result |
| --- | --- |
| Independent consumer lint | app/UI, all features/all targets, locked/offline, -D warnings: exit0 |
| Independent typecheck | app/UI, all features/all targets, locked/offline: exit0 |
| Independent related regressions | retirement_v filter:15 passed,0 failed/ignored; V8/V12/V13/V14 all retained |
| Independent whitespace check | git diff --check: exit0; existing Git CRLF notices remain visible |
| Current V14 full UI | verified receipt/log:82 passed =59 actual GPUI +23 pure,0 ignored; count matched source annotations and executed test names |
| Current formatting | verified workspace fmt and explicit rustfmt of the3 changed vendor files: exit0 |
| Current policy/native build | verified full all-feature locked/offline deny and gpui,pdfium build: exit0 |
| Default CI | V12's150 reused explicitly after101 default source/manifest/toolchain/CI/policy inputs matched; not a new V14 just-ci run |

The existing HarfRust dependency shape dead-code warning remains visible. No lint or advisory waiver was introduced. Unchanged full checks were not needlessly repeated after the narrower independent checks.

## Historical native evidence applicability

Source delta is7 changed/609,602 identical to V13. The sole runtime change is vendor/gpui/src/window.rs. Shell changes are test-only; the other code changes expose/retain actual TestPlatform callbacks, and Root's three contract/ADR changes describe this boundary. Independent comparison matched the recorded delta against both frozen source dictionaries. Settings, role resolution, layout/paint, Identify mapping, transaction confirmation, timer, save ownership and Shell async-task ownership have no new runtime delta here.

| Historical evidence | Reuse through V14 and limit |
| --- | --- |
| V9 Identify/list and draft controls | Matching actual three-screen numbers, Cancel/window identity and conflict evidence remain scoped functional observations. New callback retirement is not covered by V9. |
| V11 new UI functional runs | Valid migration/page/blank/running timer, no-play Ready/page/frozen elapsed, actual read-lock failure/single Retry, future-schema readonly and persisted role relaunch remain applicable. They are not re-labelled V14 runs. V11 F11 stays failed historical evidence. |
| V13 placement and dimensions | Immediate C full-titlebar/strict placement, exact four light sizes and dark320 geometry remain applicable. V13 light320 closing remains FAIL despite usable geometry. Three missing exact dark cells remain pending. |
| V13 chooser and close flows | Actual successful raster.pdf chooser import and Cancel retain functional evidence with unchanged fixture. Four clean closures retain their individual historical outcomes; they do not override the fifth F12 failure or replace V14 retesting. Small-file success does not prove pending-read shutdown timing. |

The first two checked new-UI PRD items remain supported by these explicitly versioned observations. Complete PoC/design/close combination items must stay unchecked while their gates are unmet. N11 requires individually grounded state observations and the explicit both-theme size matrix; it does not create an exhaustive state-by-size-by-theme Cartesian requirement.07 placeholder audience views remain distinct from08 real PDF rendering.

Identity verification read back every file in the V14 implementation manifest34, V13 native manifest92, V13 independent manifest2, V11 native manifest115 and V9 native manifest37. All hashes and recorded sizes matched. The native V13 configuration/close distinctions from its independent closeout remain unchanged, including en CRLF-to-LF parsed identity and the light-small hover error.

## Frozen handoff and process closeout

Final identity read-back at2026-10-15T07:10:51Z has no mismatch:

- Source609 aggregate: `9764bfa0d94764c314fd559d8df2b55e056fc8f6ef77882cdde89662fe5d4413`.
- EXE: `4be903dc34b4bb47163816cc88a67caf8c9f9806772f931b209bf27a9d3dbcd1`.
- Freeze SHA256: `3f00b403a9668d3a841e0ba15564d28ad804934f8b740c3d4f1803b82da41a1c`.
- Implementation manifest34 SHA256: `54d47ceaab215335cb7345788826df9d316715a4a08b3c061907c21302cca00e`.

Source review is complete and no further source/config/build mutation is planned. All reviewer check commands returned. Successful Get-Process enumeration at07:11:35Z found no cargo, rustc or ferrostage process in this tool's visible process scope; sandbox visibility is not proof of the complete host inventory. Root separately owns the host-permission inventory and launcher's existing-process refusal. Root may begin the planned native V14 verification after this handoff.

Two audit-helper errors are preserved and corrected: the first test-count scan used Windows' GBK default, then succeeded with explicit UTF-8; the initial CIM process query was denied, so product-independent-v14-process-closeout.json's EMPTY value is invalid evidence. The separate process-closeout-correction.json explicitly invalidates that receipt and records the successful Get-Process enumeration. Neither helper error changed product source or native evidence.

The reviewer performed no desktop operation, native launch, commit, push, archive or add_session. Gate B remains INCOMPLETE, C/D NOT_RUN,08–11 gated and P2 unchanged.
