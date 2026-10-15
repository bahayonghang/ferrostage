# Task07 independent check — 2026-10-14

**The dependency blocker is independently confirmed. Task07 is not delivered and Gate B has not passed.** Both allowed isolated routes have successful native compilation evidence; the unchanged dependency policy rejects both and also rejects the smaller pre/platform-only graph. No product dependency, source, CI, pin, policy, theme or window implementation changed during this review.

The reviewer loaded the saved hook context, task artifacts, check manifest, applicable frontend/backend guidelines, RTK and the `trellis-check`/`trellis-before-dev` skills. The child session's `python .trellis/scripts/task.py current --json` returned `{"current_task":null,"source":"none","stale":false}`. Root separately confirmed its active task is `10-14-07-gpui-shell`; review used that explicit dispatch path without changing either session's pointer. Root owns parent/ADR/planning edits; this reviewer owns the task07 research supplement and implement checkpoint.

## Findings (fixed)

- **File: `research/gpui-display-api.md`. Issue:** the prior activation wording could be implemented as activate-then-prompt in one callback. Windows activation queues executor work, while the path chooser captures its owner immediately. **Fix:** documented the asynchronous boundary and actual public observation path: `Context::observe_window_activation`, cached `Window::is_window_active`, then a fresh `App::active_window` identity check immediately before prompting. Windows `active_window` calls `GetActiveWindow`. Unconfirmed activation, retired presenter handles and stale callbacks must not open UI. No working focus guard is claimed.
- **File: `research/gpui-display-api.md`. Issue:** opening a fullscreen-configured window and checking its display ID cannot alone prove fullscreen completion. `toggle_fullscreen` queues work; its internal flag is set before `SetWindowPos`, and that native error is only logged. **Fix:** recorded `observe_window_bounds`, `bounds` and `display` as event-updated observations, distinguished `is_fullscreen` from native success, and retained a bounded placement-confirmation protocol as an implementation prerequisite. Audience stays Curtain until confirmed; timeout/invalid topology must fail safely. No completion future or public native-success `Result` was found in the inspected path.
- **Evidence preservation:** saved the original display report and implement checkpoint as `independent-original-gpui-display-api.md` and `independent-original-implement.md` before edits. The original `evidence-manifest.json` is unchanged. New source hashes and precise code pointers are in `independent-source-review.json`; final hashes and expected document deltas are in `independent-hash-audit.json`.

Root coordinated the corresponding parent/07 transaction updates and removed the older geometry-fallback suggestion from the 07/09 planning clarification. The current planned UUID bridge has no geometry identity fallback. Observed geometry for post-create placement verification serves a different purpose. No `.trellis/spec/` implementation convention changed: these are pin-specific research constraints, and there is no new shell code to describe as delivered.

## Findings (not fixed)

1. **Dependency policy remains failing.** Selecting a new source, implementing a maintenance patch, adding a license or waiving an advisory is a policy/design decision outside this review. Do not integrate an audited failing route. The proposed ADR-008 preserves this distinction and is a reasonable bounded evaluation scope; it remains Proposed and does not authorize any pin/policy exception. Its latest-main observations are Root's research, not evidence that this reviewer exhaustively searched upstream branches.
2. **Native confirmation remains an implementation gate.** Safe public activation and bounds observation APIs exist, but neither an implemented focus transaction nor a proven fullscreen/placement confirmation protocol exists. A stale cached flag is insufficient. Design and native regression work must close this before Start publishes Live or a presenter modal opens. Replacement-window rollback and retired-handle callbacks are still planned, not tested.
3. **Product and physical acceptance remain incomplete.** GPUI feature wiring, shell/import/actions/theme, transaction tests, all real windows/screenshots and task07 native build/clippy are NOT_RUN. Physical unplug/port changes are NOT_RUN by the user's explicit constraint; actual single/zero-display acceptance is also NOT_RUN. Gate B continues to block08 real-window integration; Gate C and other prerequisites block10/11 full integration. No P2 scope was removed.

## Verified dependency evidence

The original checkpoint contains 99 path/hash records: 18 original/retained fixture records, two binaries, 52 source/license/advisory files, one policy file, one implement checkpoint and 25 evidence files. **All 99 matched before review edits.** All three retained manifests, locks and mains match their original ignored workspaces byte-for-byte. Both executable hashes match; neither executable was launched. Route C was intentionally not compiled.

| Route | Identity and result | Independently checked evidence |
| --- | --- | --- |
| A: tagged GPUI + platform | Zed `v1.23.3`, commit `96a1b1b5389efff473fc2306f7f8c690f2e1a417`; build exit0, 3m20s; final full deny exit5 | Lock Git source/commit, cached Git HEAD, retained source, binary, build tail and paired status exits; three license errors and three unmaintained advisories |
| B: kit/component | kit/component0.7.3, pre/platform0.3.9; published upstream metadata `93d745668805029476e6d7fbb19652116fed51de`; build exit0, 1m44s; full deny exit5 | Exact registry lock checksums and package metadata, retained source/binary, build tail/status exits; six license errors and five unmaintained advisories |
| C: minimal pre/platform | exact0.3.9, no kit/base/component; original full deny exit5, independent locked/offline full deny also exit5 | Fresh Windows normal/build tree, full-target inverse cbindgen tree and independent raw audit; two license errors and three unmaintained advisories |

Historical exits above are verified from retained raw output plus the contemporaneous `route-spike-status.txt`, not mislabeled as fresh builds. The initial A cache-write build failure (101), incorrect deny CLI placement (2), and initial missing spike license failure (5) remain intact. The authoritative A audit followed the fixture's MIT OR Apache-2.0 metadata correction and contains no spike `unlicensed` failure. B/C declared that metadata from creation. The product policy was never edited.

The tagged package's `gpui0.2.2` version is from the permitted Git source, not a crates.io substitution. B's direct dependency declarations disable defaults, but kit/base/component can re-enable GPUI features transitively; it is not the minimal graph. C is the separate minimal experiment and is the evidence against a component-only explanation.

Verified required paths in C are:

- pre → image0.25.10 with EXR → exr1.74.2 → pulp0.22.3 → paste1.0.15;
- pre → resvg0.46 with required `text` → usvg0.46 text → rustybuzz0.20.1;
- pre → direct nonoptional ttf-parser0.25.1, also referenced in `font_has_char`;
- pre/resvg/usvg → tiny-skia/tiny-skia-path → arrayref0.3.9;
- pre-platform → macOS backend → pre-apple → **build dependency** cbindgen0.28.0 in the full-target graph.

The first four appear in the fresh Windows normal/build tree. Cbindgen does not; its Apple cfg/build edge remains subject to the full unchanged audit. A's libbz2-rs-sys is an additional Windows dependency. B's enum-iterator/derive and option-ext are additional Windows/tooling findings, while dwrote and rustls-pemfile are absent from B's retained Windows tree and belong to its broader graph. A Windows-only tree diagnoses membership; it does not narrow policy coverage.

Arrayref's actual LICENSE is BSD-2-Clause and requires retained notices; cbindgen's LICENSE is MPL-2.0, with distinct covered-source/distribution terms. Both are outside the current explicit allowlist. Build-only attribution does not make the configured audit pass, and MPL is not classified as permissive. This review makes no license-policy change. See the [official MPL text](https://www.mozilla.org/en-US/MPL/2.0/).

Official RustSec pages independently confirm INFO Unmaintained with no patched versions for [paste](https://rustsec.org/advisories/RUSTSEC-2024-0436.html), [rustybuzz](https://rustsec.org/advisories/RUSTSEC-2026-0206.html) and [ttf-parser](https://rustsec.org/advisories/RUSTSEC-2026-0192.html). These are maintenance findings, not demonstrated exploits. Cached advisory database HEAD is `caee11f41fab07d676c0a4655a2a5dd1f94da97c`, matching the original audit. The [official resvg changelog](https://github.com/linebender/resvg/blob/main/CHANGELOG.md) confirms the0.48.0 text migration; the pinned0.46 constraints and GPUI's own ttf-parser call mean this is not a lockfile-only remedy. No compatible replacement/fork has been validated.

## Display/source review

The exact Windows source confirms the32-u16 zero-filled, big-endian UUIDv5/DNS identity algorithm; transient HMONITOR-based display IDs; private physical bounds/effective-DPI scale; filtered enumeration failures; and silent primary-display fallback on a failed requested display. The owned task03 GDI connection name supplies a plausible safe mapping input. Complete unique set/count matching and post-create verification remain necessary. No effective-DPI claim is inferred from task03's process-context GDI ratio.

Public APIs support bindings/actions, focus observations, file chooser/drop, tab/focus accessibility metadata and appearance observations. Uppercase single letters parse as Shift+letter, so the proposed settings canonicalization remains necessary. No general public window-position setter was found; replacement-window migration with preserved core stage/timer and rollback is a coherent plan. Runtime mapping, placement, fullscreen, dialog ownership, keyboard routing and appearance behavior are all unverified. The two asynchronous corrections above are the material additions from this review.

## Verification

Environment: Windows11 Professional10.0.26300 x64; rustc1.99.0 (`b940084d7`, LLVM23.1.1), cargo1.99.0 (`5f94df478`), cargo-deny0.20.2; host `x86_64-pc-windows-msvc`. Effective cargo/rustc: `C:/Users/lyh/.cargo/bin/`; Python3.14: `C:/Program Files/Python314/python.exe`. Commands ran at repository root.

| Exact command/check | Result | Report path under `research/` |
| --- | --- | --- |
| `python .trellis/tasks/10-14-07-gpui-shell/research/independent-hash-audit.py` | exit0; no unexpected mismatch | `independent-hash-before.json`, final `independent-hash-audit.json` |
| `cargo fmt --all -- --check` | exit0 | `independent-fmt.log` |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | exit0 after approved existing-cache retry | `independent-clippy.log` |
| `cargo check --workspace --all-targets --all-features --locked --offline` | exit0 after approved existing-cache retry | `independent-typecheck.log` |
| `cargo tree --manifest-path cache/gpui-pre-minimal-route/Cargo.toml --locked --offline --target x86_64-pc-windows-msvc --edges normal,build` | exit0 | `independent-minimal-windows-tree.log` |
| `cargo tree --manifest-path cache/gpui-pre-minimal-route/Cargo.toml --locked --offline --target all --edges normal,build -i cbindgen` | exit0 | `independent-cbindgen-all-tree.log` |
| `cargo deny --manifest-path cache/gpui-pre-minimal-route/Cargo.toml --config deny.toml --locked --offline check` | exit5 after approved advisory-cache retry; expected confirmed blocker | `independent-minimal-deny.log` |
| Task06 default/native tests | Prior PASS remains applicable:128 default /63 native, not rerun | Original task06 final manifest and raw closeout logs rehashed |
| `git diff --check -- .trellis/tasks/10-14-07-gpui-shell` and explicit trailing-whitespace scan of reviewer-owned Markdown/Python | exit0; no trailing whitespace; Python script parses | `independent-diff-check.log`, `independent-process-status.json` |

The initial lint/typecheck attempts exited101 at the existing target `.cargo-build-lock`, before compilation. The initial independent deny exited1 at the read-only advisory DB lock, before auditing. Their raw logs and exit JSON files are retained with `-initial` suffixes. Approved retries used the existing caches and returned the results above. No full CI, native tests, spike rebuild or benchmark was repeated without a changed product input. Baseline fmt/clippy/typecheck are not GPUI product checks because the product still has no GPUI feature.

Task06's final manifest remains SHA256 `6d00530903376138990099497bf797ee847aac6e6b233905a7da76dbfdcadf47`. All117 source/context inputs,43 evidence files and10 external inputs match, including PDFium runtime and font/fixture provenance. Current86 crate files exactly match the baseline set, with no additional/missing crate files. All92 classified compilation/product inputs are unchanged. The canonical117-file snapshot remains `e4bef9aaf6b86d859e6d79023b81ed13bf48d58de601af1bda4649dc49e7e46c`. Thus task06's128/63 evidence is applicable; it does not establish a new task07 product pass.

The task07 implementation manifest remains SHA256 `8f4c4fdb5a51979962ff683e85210aba067b02ce740515b5a74588003ca614c8`. Final hash audit records only the intentional display-report and implement-checkpoint revisions against it, each with a byte-identical retained original. The52 original source/license/advisory files, fixtures, logs, binaries and policy continue to match. Supplementary reviewed source files include the actual Context/App/Windows activation/event implementations.

Theme status remains `DESIGN.md` planning plus the prior51 contrast calculations; no theme code, font rendering, control-state, responsive or native screenshot acceptance exists. All commands launched by this reviewer completed. The17:43:45 UTC process snapshot found no cargo/rustc/cargo-deny/clippy-driver or spike executable process. No window was launched, no task switch, commit, push, archive or journal operation occurred. Root owns the parent integration checkpoint and the next scope decision.
