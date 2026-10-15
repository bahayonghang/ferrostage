# Independent V6 native closeout

**Conclusion: the retained evidence supports the scoped V6 F7/F8 native passes, successful chooser import, saved-role relaunch and normal shutdown. The three checked task07 criteria have evidence with the version boundaries below. Task07 and Gate B remain incomplete; this is not permission to start08–11 or to mark parent AC/M1/M2 complete.**

This review inspected saved screenshots, accessibility snapshots, logs, launch/exit receipts, config read-back, current audience types and status documents. It did not control the desktop, launch an application, rebuild, run Cargo, change product/task documents, or rewrite any prior freeze/index. Only new independent closeout artifacts were written. Root remained the sole native operator.

## Findings (fixed by Root during review)

- Parent `research/gate-b.md` named GPUI0.7.3 as the adopted environment. It now correctly records the approved maintained Zed v1.23.3 source, package version0.2.2; kit/component0.7.3 was an evaluated alternate. Startup logs still print both pin constants; those log fields are not evidence that both packages are linked.
- Task07 PRD's initial footnote cited only the v5 checkpoint for A/B/C placement. Root added the actual v4 A/B log/screenshot and v5 C log, together with the v5/v6 independent applicability reviews. The checked items no longer imply all placements were repeated in one V6 run.
- Root also corrected current parent I-2/status text and local links, while preserving historical failures. Current parent PRD/traceability/execution-state and Gate B retain pending physical/native requirements and unchecked parent acceptance criteria. No product repair was required by this closeout review.

## Native assertions checked

| Assertion | Evidence and bounded conclusion |
|---|---|
| F7 Settings Close, Stop and Jump Commit/Cancel leave usable traversal | Saved `v6-close-modal-tab-open-pass`, `v6-stop-tab-open-pass`, `v6-jump-commit-tab-pass`, `v6-jump-cancel-tab-pass` AX records identify Open after Tab. Stop screenshot remains Ready1/9. These supplement the previously reviewed real-GPUI tests. |
| Focused Start activates, then Tab works | `v6-start-tab-settings-pass.json` identifies Settings focus; screenshot shows Presenting1/9 and Stop. The preceding log confirms actual fullscreen Audience and Presenting commit. **This Start→Tab route is native evidence only, not automated red→green coverage.** |
| F8 has one save-feedback owner and Retry clears it | `v6-save-failure-single-message-0.jpg` shows one unsaved warning/Retry at Ready2/9. The locked Retry receipt retains failure. `v6-retry-success-clears-error-0.jpg` shows warning/button gone with page2 unchanged. This is visual feedback evidence in addition to automated SettingsStore tests. |
| Disappearing Retry returns usable focus | `v6-retry-removed-tab-open-pass-0.jpg` shows Open's focus ring. That first JSON still reports application root7, so it alone does not prove AX focus. The later `v6-retry-tab-readback.json` explicitly reports Open8 and resolves the stale snapshot. |
| Runtime roles survive failed persistence, then save/relaunch | Launch config SHA `41798cb34079e64d46078604e4c382ec1698d1ad925977870caf8c71e66fefe3` is the locked original. Swap confirms PresenterB/display65731 while the save fails. Read-back and post-run config SHA `b96b8708fcb7207f7673c322be62d1f2f68d1c9ff382fe3607fbf7d0d7249ab9` agree on A Audience/B Presenter/C None. Relaunch uses those same bytes and confirms PresenterB, Idle, audience=None. |
| Native chooser import | Root's recorded folder-navigation operation plus `v6-chooser-import-pass` AX/screenshot establish completion: previously Ready2 becomes Ready1/9 with Open focused. The screenshot shows the resulting UI, not each intermediate chooser interaction. PDF fixture hash is unchanged. The older failed text-injection attempt remains a tool limitation, not a product failure. |
| Close and process ordering | PID29092 exits0 at02:45:52.793Z; PID175992 is launched only at02:46:10.350Z and exits0 at02:46:51.099Z. Final saved Sky inventory at02:52:10.180Z contains no FerroStage windows. The lock helper writes its unlock receipt after disposing its file handle; helper command exit0 is Root's checkpoint observation, not a fabricated exit-code field in that receipt. |

All timestamps above are UTC on2026-10-15. Saved window screenshots were viewed directly from disk. The small rendered page numeral and Presenter page placeholder are task07 output; no full PDF/workbench rendering acceptance is inferred.

## Three checked PRD criteria

1. **A Audience / B Presenter / C None:** retained v4 `a-cli` launch/source identity and stdout show Presenter65731 confirmed windowed, Ready with audience=None, then Audience131073 confirmed fullscreen with matching display bounds. `v4-a-presenting` and `v4-a-audience-page1` captures corroborate B Presenter/A Audience; the PoC checkpoint and `v4-escape-ready-page2` retain audience dismissal/Ready. C None means no FerroStage audience was created there, not a pixel-by-pixel assertion about unrelated desktop content. V6's chooser case establishes actual chooser completion separately. This is the original v4 native placement observation, not a claim of a fresh V6 A/B placement run.
2. **C Audience:** v5 `c-vertical` starts Ready/audience=None on PresenterB, then confirms actual display65735 with fullscreen bounds `[1706.6666,-761.3333,1066.6666,1706.6666]`. The portrait audience screenshot is at physical origin(2560,-1142), and stopped-Ready capture/normal exit are retained. This is v5 native evidence. V5→V6 changes were limited to focus recovery and save-feedback ownership, independently reviewed against the frozen sources; no role-mapping/fullscreen-placement rewrite is hidden in V6.
3. **Only-snapshot audience:** current `audience.rs` has one field, `Entity<AudienceProjectionSnapshot>`. The current snapshot and its nested visual/media types carry projection values/pixels and no LoadedDocument, SlideNotes, speaker notes, path or controller. Direct source reading and targeted `rg` corroborate the earlier review. This establishes the field boundary, not complete parent AC-5/workbench privacy acceptance.

The remaining combined PRD items correctly stay unchecked. Native PageDown/B/Escape observations do not establish the unobserved audience-cursor clause. CI passing does not complete the PoC's physical requirements.

## Evidence integrity and applicable checks

`python .trellis/tasks/10-14-07-gpui-shell/research/native-v6-independent-audit.py` returned **0**. `native-v6-independent-audit.json` records environment, exact command, per-manifest results, config/run ordering and identities. This report and the retained evidence are indexed by the separate `native-v6-independent-manifest.json` and `.sha256`; no older index is changed. Independent recomputation matched:

| Manifest | Records | SHA256 |
|---|---:|---|
| `native-v6-evidence-manifest.json` |54/54|`6c7551fa9c38890252c0500951c90597500b2f7a690d1e52aefe89986a6ac1a0`|
| `native-v5-evidence-manifest.json` |108/108|`ddfec0154c3dfe6731eb08b2c5dcf4b9b27c2a49ccfd54589a05f2ed295d73d0`|
| `product-v6-evidence-manifest.json` |26/26|`dac5512c8887bbb3278e0b9c68fc8a622731fbc40eea187aa7dfef090a40912c`|
| `product-independent-v6-manifest.json` |141/141|`252a22e5ab5173b7c9a03da0e6a15768edd2d904b6cfd5404c42c9f72c101fb5`|

The preserved V6 freeze SHA is `b7df3a62ff2b1b7212924e47f79d72468ff86356265a1aaf650c0694adb6d7ea`; its602-entry aggregate recomputes to `b71eb54d7da71f9b30ac5ab3bdb58fc1ada637bf2991adde10041f0a172170be`. Before native,602/602 matched in the independent V6 audit and Root identity receipt. At this closeout601 match; only DESIGN.md line151 changed after native shutdown. Retained `native-v6-design-before-closeout.md` exactly matches frozen DESIGN SHA `1b13bda5cc6460a8f320a637e047c836813f817231315533d200ed873d31f4d6`; current DESIGN SHA is `88a7a31303e10d3adfc0be373bf326d16751123b54a2b211984ba9bfd6fe99c9`. All94 current crate files remain covered and unchanged.

Executable `d4f2e653ee793176b384034797ad99e1ad665aa0da6535a454ba88a36b055074`, PDFium DLL `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`, Cargo.lock `2557e10b37fa5cfe63e78ce19965b8bbbd83438bdd540eed50405fbbc3dbc23e`, and fixture `25a7932f46ee8fcd4866011921c574b4f5b343af05b793bc397b7e93ed0df013` independently match. Screenshot hashes referenced by V6 JSON also match. The three prior manifests remain unmodified; their counts overlap and are not a distinct-test total.

- **Lint: PASS, reused.** Previously reviewed strict all-feature/all-target consumer clippy and default CI remain applicable. The six disclosed direct upstream HarfRust lint failures remain separate; no waiver or suppression was added.
- **TypeCheck/build: PASS, reused.** Same frozen compiled sources, dependency graph and native executable. A documentation status-line correction does not require rebuilding.
- **Tests: PASS, reused.** Actual GPUI49 and default145 receipts still match. Targeted native F7/F8 above now pass. Unchanged full deny evidence and only the approved three exact exceptions remain applicable.
- One exploratory read incorrectly treated a manifest's dictionary as a list and returned a Python TypeError. The corrected schema read and completed audit above supersede that inspection error; no product command or validation was counted as passing because of it.

## Findings (not fixed / remaining limits)

- Physical unplug/reconnect, port change/identity stability and ≤1s topology response remain **NOT_RUN** by user availability. Actual single/zero-display hardware and windowed single-screen audience remain unverified. Simulated events and a no-Audience-role config do not replace these tests.
- Full exact logical-client size/DPI/state matrix, live OS theme switching, audience cursor behavior and direct Curtain-transition capture remain pending. Existing English resize attempts did not reach requested dimensions. Settings remains task07's placeholder, and08–11 remain blocked.
- Generic `window not found` and invalid-window-handle errors persist around retirement/normal closure. Exit0 and successful recovery do not locate their cause or prove them harmless. This review did not suppress them or claim resolution.
- Current task07/Gate B, all combined parent AC and M1/M2 remain incomplete. Current approval covers the adopted maintenance/source/license scope; it does not waive native gates or authorize delivery side effects.

The reviewer has no active command, build, desktop action or owned native process. Root's two controlled V6 app runs have explicit exit0 receipts, its lock helper release/exit is recorded, and the final inventory is empty. No new execution remains pending from this review.
