# Implementation and verification

## Planning gate

- [x] Inspect existing commands, runtime staging, GUI subsystem/icon, docs, CI and actual local Cargo installation.
- [x] Create task and capture the explicit Setup request.
- [x] Perform authorized legacy uninstall and save `legacy-uninstall.json`.
- [x] Write converged PRD/design/checklist and research; curate implementation/check contexts.
- [x] Present final summary and receive explicit approval of that summary. User approved implementation and final local installation on 2026-10-18.
- [x] Only then run `python .trellis/scripts/task.py start .trellis/tasks/10-18-windows-desktop-distribution`.

## 1. Build and package

- [x] Load relevant specs; dispatch Trellis implementer with active task path and file ownership, preserving others' changes.
- [x] Acquire/pin a real stable NSIS compiler archive/hash and notices; use zlib. Do not accept WiX EULA.
- [x] Implement shared orchestration with Cargo-output/version resolution, build-before-stage ordering, incremental reuse, checked subprocess statuses and progress.
- [x] Build app graph with `--release --locked` and `gpui,pdfium`; create explicit verified payload/inventory and distinct licenses.
- [x] Update build/install recipes and add release/uninstall; preserve dev and default CI.
- [x] Generate versioned Setup/ZIP/checksums from that same payload and verify output before reporting success.

## 2. Installer lifecycle

- [x] Implement NSIS Modern UI setup/uninstaller, current-user scope, existing icon, Desktop/Programs shortcuts and quoted HKCU Installed apps entry.
- [x] Implement stable identity, same-version reinstall, forward update, downgrade refusal, file-lock/preflight handling and owned-file cleanup. Never force-kill or recursively remove unchecked paths.
- [x] Verify partial failure cannot falsely register a successful new version or leave a deceptively successful incomplete install.

## 3. Automated and native verification

- [x] Add meaningful checks for missing/corrupt payload, notices, target override, checksum tampering and subprocess failures; prefer lifecycle behavior over source-text assertions.
- [x] Exercise shared build/release pipeline and final actual `just install`; save artifact hashes, payload inventory and installed registration/shortcut metadata.
- [x] Exercise fresh install, same-version reinstall, forward upgrade and downgrade refusal with isolated test-version packages; restore real version afterward. Confirm single app entry.
- [x] Check running-app/file-lock failure; no process killing or false success.
- [x] Exercise native uninstall and `just uninstall`; preserve settings/PDF canaries and unrelated files, remove owned payload/shortcuts/keys, keep Cargo bin clear. Reinstall the final real package.
- [x] Run `just ci`, `just check-shell`, focused distribution checks and `git diff --check`; repeat broad gates only for new changes/failures.
- [ ] Load applicable computer-use skill before UI interaction. Verify actual Setup wizard, no elevation, shortcut icon/GUI launch without console, Windows Search discovery and normal-window real PDF open. Record any unavailable native checks as NOT_RUN, never PASS. No size/DPI matrix.

## 4. Integration and finish

- [x] Add Windows packaging CI/artifact upload through the shared pipeline; do not publish a release/tag.
- [x] Update English/Chinese READMEs, `docs/packaging.md`, and `AGENTS.md` commands; remove stale Cargo-bin/no-installer claims.
- [x] Inspect Windows/VC prerequisites and document signing/clean-machine limits accurately.
- [x] Dispatch Trellis checker on the full scoped diff and fix verified findings; update applicable distribution specs.
- [x] Create the approved work commit and hand off to the authorized archive/journal commands; retain receipts and exact validation limits. No public release or push is authorized.

## Rollback checkpoints

Fail on runtime/notice mismatch before packaging. Clean only validated owned generated paths. Preflight locks before install mutation, preserve settings/documents on all paths, and never remove unknown install-directory contents. Lifecycle tests must finish with the real application installed. Do not change GPUI/vendor families, Cargo license policy or unrelated UI.

## Final status

See `research/validation.md` and `research/final-installation.json`. Native desktop/PDF acceptance passed; actual Search UI remains NOT_RUN due to helper targeting limits. Final silent installation is complete. Code is reviewed and work commit `544df2046a5e56a6aa813b11b2cf2e66eeb07336` was created locally. The user authorized task archive/session recording; task status and developer journal record the bookkeeping result. No push is authorized. Native Search UI remains a recorded qualification limit rather than a claimed PASS.
