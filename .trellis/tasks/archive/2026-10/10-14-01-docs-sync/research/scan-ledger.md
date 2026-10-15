# Scan ledger — 10-14-01-docs-sync

Historical snapshot from the initial task 01 implementation, before task 02 product edits. The line numbers, empty crate diff, and absence of write APIs below describe that snapshot only. The current review and classifications are in [review-2026-10-14.md](review-2026-10-14.md); do not use this ledger as current workspace evidence. Its fixed `settings.toml.tmp` wording was corrected to a unique sibling temporary file during the follow-up review.

Recorded 2026-10-14 after the docs edit, and checked again after the review fixes. Exit code 1 from `rg` would mean no matches; these scans matched, so they exited 0. A hit is not by itself a failure. Historical mentions and explicit prohibitions stay. No current positive LaTeX-build or annotation write-back capability was left in the scanned docs. The review did not move any classified line below.

## Commands

| Command | Exit |
| --- | --- |
| `rg -n -i "latexmk\|xelatex\|FreeText\|annotation edit\|edit annotations" AGENTS.md docs README.md README.zh-CN.md crates` | 0 |
| `rg -n "OpenOptions\|File::create\|write\|SHADOW_TMP_SUFFIX\|ferrostage.tmp" crates AGENTS.md docs .trellis/spec` | 0 |
| `rg -n "ferrostage.tmp" AGENTS.md docs` | 0 |
| `git diff --check` | 0 |
| `git diff --stat` | 0 |
| `git diff --stat -- crates Cargo.toml Cargo.lock justfile rust-toolchain.toml` | 0 (empty) |

`git diff --check` also printed "LF will be replaced by CRLF" warnings. Those are autocrlf notices, not whitespace errors.

`OpenOptions` and `File::create` had no hits. `crates/` had no scan-1 hits.

## Scan 1 — forbidden capability phrases

| Path | Line | Classification | Disposition |
| --- | --- | --- | --- |
| `README.md` | 22 | excluded | "Not in v1.0" lists PDF annotation editing and write-back as out of scope, and keeps read-only media-structure discovery. Prohibition, not a capability. Kept. |
| `docs/PROJECT_PLAN.md` | 12 | historical | v0.2 revision row names `latexmk` / `xelatex` as the deleted toolchain. The same cell says that positive capability was removed and hot reload is generic PDF replacement. Kept. |
| `docs/PROJECT_PLAN.md` | 13 | historical | v0.2 revision row names `/FreeText` and annotation write-back as deleted. Same line as the `ferrostage.tmp` hit below. Kept. |
| `docs/PROJECT_PLAN.md` | 235 | excluded | States that v0.2 PoC-5 (`xelatex` recompiles) was deleted. The replacement hot-reload check writes only a fixture in an independent temp directory. Kept. |

`README.zh-CN.md`, `AGENTS.md`, and `crates/` produced no scan-1 hits. `docs/feature-parity.md` has no annotation-editing row.

## Scan 2 — write APIs and temp suffixes

| Path | Line | Classification | Disposition |
| --- | --- | --- | --- |
| `AGENTS.md` | 20 | allowed-readonly-structure | "never writes them" forbids writing the user's PDF. No write target on this line. Settings `.tmp` + rename is the next bullet and is not this hit. |
| `docs/architecture.md` | 17 | allowed-readonly-structure | Matched "never writes". The same sentence names the allowed rename target: sibling `settings.toml.tmp` (a settings file) and FerroStage's own artifact temps. Neither is the open PDF. |
| `docs/PROJECT_PLAN.md` | 13 | historical | Matched `.ferrostage.tmp` inside the deleted v0.2 user-PDF replace path. The disposition cell says that token only records the old path. Current atomic rename is `settings.toml.tmp` and FerroStage's own artifacts (ADR-004). |
| `docs/adr/ADR-004-lock-free-reload.md` | 5 | allowed-readonly-structure | "does not write the user's PDF". Prohibition. |
| `docs/adr/ADR-004-lock-free-reload.md` | 12 | allowed-readonly-structure | "does not truncate, rewrite, or atomically replace the user's PDF". Prohibition. The PDF is an external replacement opened read-only. |
| `docs/adr/ADR-004-lock-free-reload.md` | 14 | write-target | Authorizes a sibling `settings.toml.tmp` renamed onto `settings.toml`. Same sentence: that file is never the user's PDF. FerroStage's own artifacts may use separate sibling temps. |
| `docs/adr/ADR-004-lock-free-reload.md` | 20 | allowed-readonly-structure | "`StageShadowDocument` … is not a write path." The bytes are a read of the external replacement. |
| `.trellis/spec/backend/database-guidelines.md` | 9 | allowed-readonly-structure | "FerroStage never writes that path" for the open PDF. |
| `.trellis/spec/backend/database-guidelines.md` | 29 | allowed-readonly-structure | Forbidden-list entry. "Writing the user's PDF" and "truncate-and-write" are prohibited. No write is authorized. |
| `.trellis/spec/backend/database-guidelines.md` | 30 | write-target | Forbids `SHADOW_TMP_SUFFIX` (`.ferrostage.tmp`) as a user-PDF path. The authorized target is sibling `settings.toml.tmp` in the config directory, plus separate temps for FerroStage's own artifacts. `from_toml` has no write side effect. |
| `.trellis/spec/guides/code-reuse-thinking-guide.md` | 11 | excluded | Trellis guide word "rewrite". Not a FerroStage file write. Unchanged. |
| `.trellis/spec/guides/cross-layer-thinking-guide.md` | 151 | excluded | Trellis template "writer". Not a PDF or settings write. Unchanged. |
| `.trellis/spec/guides/cross-layer-thinking-guide.md` | 154 | excluded | Trellis template "writes" for an init/update scenario. Unchanged. |
| `.trellis/spec/guides/cross-layer-thinking-guide.md` | 248 | excluded | Duplicate of line 151. Unchanged. |
| `.trellis/spec/guides/cross-layer-thinking-guide.md` | 251 | excluded | Duplicate of line 154. Unchanged. |
| `.trellis/spec/guides/cross-layer-thinking-guide.md` | 307 | excluded | Trellis "event writer". Unchanged. |
| `.trellis/spec/guides/cross-layer-thinking-guide.md` | 316 | excluded | Trellis "event writer". Unchanged. |
| `crates/ferrostage-platform/src/lib.rs` | 16 | current-conflict | Comment still says the suffix is for when FerroStage itself writes a PDF. No caller performs that write. Not edited: product code is out of scope. Task 10 deletes the constant and this comment. |
| `crates/ferrostage-platform/src/lib.rs` | 19 | current-conflict | Same comment block tells the reader not to truncate a destination and to rename a sibling over it. In context the destination is a PDF. Not edited. Task 10. |
| `crates/ferrostage-platform/src/lib.rs` | 20 | current-conflict | `SHADOW_TMP_SUFFIX` is still `".ferrostage.tmp"`, defined as that PDF temp suffix. Not edited. Task 10. |
| `crates/ferrostage-platform/src/lib.rs` | 22 | allowed-readonly-structure | "until a write has produced this" is the external replacement's `%%EOF`, which reload waits for. FerroStage does not write it. |
| `crates/ferrostage-platform/src/lib.rs` | 35 | allowed-readonly-structure | "finished PDF write" describes detecting that external EOF, not a FerroStage write. |
| `crates/ferrostage-platform/src/lib.rs` | 54 | current-conflict | Test locks `SHADOW_TMP_SUFFIX == ".ferrostage.tmp"`. Not edited. Task 10 removes it with the constant. |

## Scan 3 — `ferrostage.tmp` in AGENTS.md and docs

| Path | Line | Classification | Disposition |
| --- | --- | --- | --- |
| `docs/PROJECT_PLAN.md` | 13 | historical | Only hit. The token names the deleted user-PDF replace path. The same line limits the current atomic rename to settings file `settings.toml.tmp` and FerroStage's own artifacts. It does not authorize writing the user's PDF. |

`AGENTS.md` has no `ferrostage.tmp` hit. Settings there use `settings.toml.tmp`.

## Outside these commands

Not scan hits. Left unchanged because they are product code or outside the docs roots. They are still current conflicts with the new contract:

| Path | Line | Classification | Disposition |
| --- | --- | --- | --- |
| `crates/ferrostage-render/src/lib.rs` | 4 | current-conflict | Comment still says P1 neighbor / P2 presenter / P3 prefetch. `docs/architecture.md` now follows `PROJECT_PLAN.md` §4.6 (P1 presenter current/next/notes, P2 audience ±1, P3 jump-preview hover, P4 thumbnails). Task 05 owns the comment. |
| `tests/hot_reload/README.md` | 3 | allowed-readonly-structure | Names `SHADOW_TMP_SUFFIX` only to say it is not a user-PDF write path and that task 10 removes the constant. Outside the required scan roots. The constant in `crates/` is unchanged. |

The skeleton still has `screens_swapped` and `Option<u32>` monitor indexes. ADR-003, ADR-007, and the frontend state spec now call those non-authoritative. Removing them is task 04 / 02 / 03, not this docs change.

`docs/packaging.md` does not state a LaTeX toolchain or annotation-edit capability, so it was not edited.
