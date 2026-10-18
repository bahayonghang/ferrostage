# F14 — concrete AccessKit lifetime proposal (research only, not adopted)

**Product status: unresolved FAIL.** V15 fixes F13 only. No AccessKit or gpui_windows package has been added to the approved five-package maintained set, no product dependency/lockfile changed, and no accessibility behavior was disabled. The standalone candidate under `product-v15-accesskit-candidate/` is not linked to FerroStage. Independent review and an explicit maintenance decision are still required before adoption.

## Captured failure and reachable ownership chain

`native/v14-dark-matrix.stderr.log` captures an unnamed UIA thread panic at `accesskit_windows-0.34.0/src/util.rs:380:52`: `ClientToScreen(...).unwrap()` receives HRESULT `0x80070578` (invalid window handle), then the COM boundary cannot unwind. Process exit 0 and empty windows do not make this close a PASS. The captured stack does not identify which of four provider methods triggered the call; Root will use `RUST_BACKTRACE=full` for the next authorized native run.

Pinned AccessKit `PlatformNode` has `Weak<Context>`. `with_tree_and_context` upgrades it to `Arc<Context>`, locks the tree, then invokes the provider method. Context contains the HWND. A provider call that has already upgraded the Arc can outlive the adapter owner. It calls `ClientToScreen` directly; it does not pass through GPUI activation/hover callbacks or the GPUI action channel.

Pinned GPUI Windows revision `96a1b1b` keeps `WindowsWindowState.a11y: RefCell<Option<A11yState>>`, where A11yState owns AccessKit Adapter. `WindowsWindow::drop` clones its inner owner into a detached foreground task, waits for dialog idle, then calls RevokeDragDrop/DestroyWindow. Thus the inner owner and its adapter remain live across native HWND destruction. `events.rs::handle_destroy_msg` and the WM_NCDESTROY branch do not take a11y. `handle_getobject` can return the live adapter provider. AccessKit Adapter exposes no shutdown/drain/disconnect method.

Existing `vendor/gpui` has `a11y_init`, `a11y_tree_update` and `a11y_update_window_bounds`, but no supported per-window native teardown API. Cancelling its channel receivers alone would not affect these direct COM calls. Clearing child bounds would not fix root hit testing, which unconditionally converts coordinates. Reinitializing the adapter can let a subsequent WM_GETOBJECT create another context, and cannot revoke an already-upgraded context. None of those are claimed fixes.

## Compare the minimum approaches

| Approach | What it covers | Remaining race / cost | Decision |
| --- | --- | --- | --- |
| GPUI channel/task cancellation | Late GPUI work | Direct COM provider calls bypass it | Not an F14 fix |
| gpui_windows takes/drops Adapter before DestroyWindow and blocks new WM_GETOBJECT | Removes the owner's strong Context reference earlier | A COM call may already hold an Arc and continue after destruction. Moving the drop alone moves the race. | Insufficient as a complete repair |
| Full native retirement plus in-flight drain | Stops new provider publication and waits for existing native access | Needs an AccessKit API/protocol as well as gpui_windows. Must avoid deadlocking GUI SendMessage/COM reentrancy while draining. Larger two-package scope and substantially more synchronization testing. | Possible broader design, not the minimum demonstrated fix |
| AccessKit handles failure at each actual coordinate conversion | Already-upgraded/in-flight Context receives a normal provider error when ClientToScreen reports an invalid HWND | Does not establish full shutdown, handle-reuse protection, other native calls or all UIA methods | Smallest concrete candidate for the captured panic; proposed for review only |

[UiaDisconnectProvider](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationcoreapi/nf-uiautomationcoreapi-uiadisconnectprovider) can make later client calls report an unavailable element, but is subject to COM outbound-call restrictions while handling SendMessage and must not republish a disconnecting provider through reentrant WM_GETOBJECT. The [UiaReturnRawElementProvider WM_DESTROY cleanup](https://learn.microsoft.com/en-us/windows/win32/api/uiautomationcoreapi/nf-uiautomationcoreapi-uiareturnrawelementprovider) releases UIA's event-map references; neither instruction alone proves that a call already holding Context has drained.

## Reviewable patch

`product-v15-accesskit-proposal.patch` changes four upstream source files:

1. `src/util.rs::client_top_left`: return `windows::core::Result<Point>`. Test the result of the **actual** ClientToScreen call using BOOL.ok(), with no earlier IsWindow check. Convert only `ERROR_INVALID_WINDOW_HANDLE` to `UIA_E_ELEMENTNOTAVAILABLE`; return other native errors unchanged. Valid coordinates are unchanged. No panic catcher or global logging suppression enters production.
2. `src/context.rs::Context::client_top_left`: propagate Result without changing ownership.
3. `src/node.rs`: `IRawElementProviderFragment_Impl::BoundingRectangle`, `IRawElementProviderFragmentRoot_Impl::ElementProviderFromPoint`, and `ITextProvider_Impl::RangeFromPoint` use `?`. BoundingRectangle keeps its existing zero rectangle for a node with no bounding box.
4. `src/text.rs::ITextRangeProvider_Impl::GetBoundingRectangles`: propagate conversion failure, preserving the empty-range early return.

This checks the conversion where the failure actually occurs, so no check/use interval is added. It also permits a caller already holding an Arc to return a normal COM error. It does **not** solve HWND reuse: if a numeric handle were recycled to a new live window, ClientToScreen could succeed with another window's coordinates. A full ownership/generation protocol is a separate design question; this proposal must not be described as solving all provider lifetime issues.

## Executed isolated evidence

The research script copies the exact cached registry package to an independent `[workspace]` under research, removes GUI-test dev dependencies/examples from that standalone manifest, redirects the test module to the attached narrow harness, and uses its own Cargo.lock. Product Cargo.toml/Cargo.lock remain byte-identical to V14. No window is created, moved or closed by these tests. The valid control uses the existing desktop HWND read-only.

- `product-v15-a11y-red.json`: unmodified util.rs; live desktop positive control PASS, invalid HWND test FAIL with the exact native HRESULT and util.rs:380 panic. This is a real Windows API result, not a source-string test. The sentinel invalid HWND models the post-destruction handle state; it does not reproduce the timing of a real close.
- `product-v15-a11y-green.json`: preserved compile failure because the pinned ClientToScreen binding returns BOOL, not Result. No test result was inferred from that run.
- `product-v15-a11y-green-compiled.json`: corrected BOOL.ok() conversion; three tests PASS.
- `product-v15-a11y-green-providers.json`: four tests PASS. In addition to the leaf negative and live positive controls, `retained_context_com_queries_return_element_not_available` creates a real AccessKit Context/tree/provider, retains its strong Arc with an invalid HWND, calls actual in-process COM BoundingRectangle and ElementProviderFromPoint, and observes the expected UIA HRESULT without unwinding. This demonstrates the error path while Context is still live, rather than assuming owner drop cancels it.

The exact reusable harness is `product-v15-a11y-tests.rs`; the final candidate contains the same bytes. `product-v15-a11y-green-text.json`: six tests PASS. Both remaining text COM callers return the expected unavailable-element HRESULT with retained invalid-HWND Context. The live desktop control exercises all four COM methods, verifies exact rectangle coordinates and hit-test success, text `AB`, and all four SAFEARRAY rectangle values; the array is explicitly destroyed. All four production callers compile and execute in the standalone package.

`product-v15-a11y-hidden-compile.json`: the Root-only native harness compiles with `--no-run`; it has NOT been executed by the implementer. Command for Root, after the product handoff:

```powershell
python .trellis/tasks/10-14-07-gpui-shell/research/product-v15-a11y-candidate.py root-hidden
```

This separate feature performs 16 owned hidden STATIC-window cycles (no WS_VISIBLE, ShowWindow or activation). A worker retains Context/providers before the owning thread drops its Context and calls actual DestroyWindow. After synchronized destruction, all four COM methods must return unavailable-element errors. Each channel wait has a five-second timeout and the owner destroys only its own HWND. This deterministically tests native destruction with retained context on another thread; it does not claim randomized simultaneous conversion/destruction or HWND reuse coverage. Root owns execution and the resulting `product-v15-a11y-root-hidden.*` receipt. Original upstream GUI tests, actual external UIA clients and full app shutdown remain NOT_RUN for this candidate. The candidate is not a native product PASS and must not be counted as F14 repaired.

## Identity, policy and maintenance cost

Source: registry `accesskit_windows 0.34.0`, registry checksum `106c2b961215864d1c2e703ee63269c25c4e80a577ffb2c1017b9c17dcdf83a1`, upstream VCS `c88605b96d04431f9c3c792464a0f2f253480e94`, `platforms/windows`. Its declared license is MIT OR Apache-2.0, already allowed; no license exception is proposed. `product-v15-a11y-upstream.json` hashes the exact original four files and manifest. Original copyright/license headers remain in the research copy.

Adoption would nevertheless add a **sixth maintained package**, outside current ADR-008 authorization. The minimal patch has no public API change and no dependency update, but it needs an upstream tracking/removal condition, exact reconstruction provenance, rebase checks for all four callers, test maintenance and full all-feature product deny/lint/build. A gpui_windows retirement/drain implementation would add another maintenance surface and synchronization obligations; it is not silently substituted here.

Before adoption: independent review of this patch and captured native evidence; explicit boundary decision; Root execution of the compiled real-DestroyWindow harness and review of its bounded coverage; confirm native live accessibility still works. After adoption: exact-hash full product checks, Root's real titlebar-close/Presenting teardown with full stdout/stderr/backtrace, and repeated controlled UIA queries during retirement. Current V15 native runs use the unpatched approved dependency graph and remain useful to isolate F13 and collect F14 evidence. Any clean run is time-scoped evidence, not proof that the race is gone.
