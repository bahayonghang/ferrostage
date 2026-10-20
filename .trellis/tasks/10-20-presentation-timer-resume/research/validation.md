# Validation: presentation timer resume and header timer redesign

Date: 2026-10-20. Toolchain 1.99.0. Headless results only; native acceptance is a separate item.

## Commands

| Command | Result |
| --- | --- |
| `cargo test -p ferrostage-core --locked` | PASS: 34 passed, 0 failed |
| `cargo test -p ferrostage-ui --features test-support --lib --locked` | PASS: 230 passed, 0 failed (after trellis-check added the narrow split regression) |
| `cargo clippy -p ferrostage-ui --features test-support --all-targets --locked -- -D warnings` | PASS (GPUI code paths, which default-feature clippy does not compile) |
| `just ci` → `fmt-check` | PASS |
| `just ci` → `clippy` (`--workspace --all-targets -D warnings`) | PASS |
| `just ci` → `test` (`--workspace --all-targets`) | PASS |
| `just ci` → `deny` (`cargo deny check`) | NOT_RUN: `cargo-deny` is not installed on this machine (`error: no such command: deny`). No dependency, `Cargo.lock`, `Cargo.toml` or `deny.toml` change in this task. |

## Acceptance items (prd.md)

| Item | Status | Evidence |
| --- | --- | --- |
| Core: Stop makes a session resumable; Resume continues from the frozen value; Start/Restart zeroes | PASS | `state::lifecycle_tests::stopped_session_resumes_from_frozen_time_or_restarts_at_zero` |
| Core: Load/LoadMapped/Close clear; hot reload (`CommitDocument`, `commit_reload`) keeps; <1 s not resumable (1 s boundary is) | PASS | `state::lifecycle_tests::short_sessions_and_document_boundaries_clear_resumable_time` |
| Core: illegal-stage Resume returns `NotReady`, no audience returns `NoAudienceDisplay`, state unchanged | PASS | `illegal_lifecycle_edges_leave_every_field_unchanged` (Resume in Idle/Ready-without-session/Presenting), `stopped_session_resumes_…` (no audience) |
| Core: property test covers new command; invariant resumable ⇒ Ready, frozen ≥ 1 s and stable | PASS | `state::property_tests::arbitrary_commands_preserve_stage_privacy_and_error_atomicity` (ops 22/23 = Resume with/without audience) |
| UI: Ready + resumable shows two segments that dispatch Resume / Restart through real clicks; no-resumable shows single Start | PASS | `shell::tests::resumable_session_splits_start_and_dispatches_resume_or_restart` (real click, pending `Start(Resume)` / `Start(Restart)`, busy guard rejects the second segment, committed timer continues ≥ 754 s or restarts < 5 s, page unchanged, Home clears) |
| UI: F5 resolves to Resume when resumable, fresh Start otherwise; cancelled Resume keeps the session | PASS | `shell::tests::start_shortcut_resumes_interrupted_session_and_starts_fresh_otherwise` |
| UI: Presenting header contains Layout & navigation, bottom strip does not; menu opens below; Escape restores trigger focus | PASS | `shell::tests::presenting_header_hosts_layout_menu_and_timer_states`, `shell::tests::layout_menu_applies_each_layout_and_opens_below_header_trigger` (8px below, right-aligned, inside 12px margin) |
| Timer centered at ≥1280 and 800–1279; long filename never moves the digits; narrow (<800) visible | PARTIAL | `shell::tests::header_timer_centers_and_ignores_filename_length`. Centered within 1px at Ready 1100/1280/1600 and Presenting 1000/1280/1600; centered on its own row at 600. Below ~1100 (Ready, five header actions) and ~1000 (Presenting) the right group's min-content exceeds half the free width, so the row degrades to non-overlapping, uncentered placement as anticipated in design.md §6. Digits bounds are identical for short and very long filenames at every tested width. |
| Timer states distinguishable without color only | PASS | Mark shape + label: `timer-mark-{running,paused,stopped,fresh}` asserted in the three UI tests above; labels `wb-elapsed` / `wb-timer-paused` / `wb-timer-stopped` via `capsule_probe` |
| Contrast of new pairs in light/dark | PASS | `theme::tests::timer_capsule_and_split_session_pairs_meet_design_contrast` (muted/body/ink/warning/error text ≥4.5 on soft, marks ≥3, Restart sublabel ≥4.5 on navigation fill/hover/pressed, `ON_PRIMARY_DETAIL` ≥4.5 on all primary fills) |
| Locale parity | PASS | New keys in both `assets/locales/{zh-CN,en-US}/app.toml`; `locale::tests::catalogs_have_identical_keys_and_interpolation_fields` passes |
| Audience snapshot unchanged; core has no new IO/GPUI deps; no settings keys | PASS | `AudienceProjectionSnapshot` untouched (no diff in `snapshot.rs`); `display::tests::core_manifest_does_not_link_io_crates`; no `ferrostage-settings` diff |
| Native manual acceptance (Start → Esc → frozen → Resume continues → Esc → Restart zeroes; header menu; light/dark screenshots) | PASS (user) | 2026-10-20: the user ran the native scenario and confirmed it in chat ("没问题"). Screenshots were not archived in this task. |

## Deviations from design.md and their reasons

- **Segment gap is 4px, not 2px.** Every activation target reserves 4px (2px focus ring + 2px gap). Overlapping the two reservations by exactly 4px (`ml(-4px)` on Restart) keeps both focus rings fully visible. A 2px gap would hide one ring edge under the neighbouring face.
- **Label slot is `12px × 6.5` wide, not `× 5`.** "Running low" / "Remaining" in en-US need about 65px at 12px. A 60px minimum would let the slot grow and move the digits when the label changes.
- **Dividers use `p.border`.** The Rust palette has no `hairline` token, and existing workbench and menu dividers already use `border`. Adding a token was outside scope.
- **State marks are styled divs (rounded fill, 1.5px ring, 1px-radius square), not `canvas` paths.** They are not font glyphs, which was the intent. The Restart arrow is a canvas polyline arc (19 points) with a filled head, not `arc_to`.
- **Resume sublabel color is a precomputed token `theme::ON_PRIMARY_DETAIL = #F1E5E1`** (ON_PRIMARY at 85% over PRIMARY) instead of runtime opacity, so contrast is testable. It measures about 6.4:1 on PRIMARY. When disabled, the sublabel uses the disabled ink.
- **Warning color and short labels apply only to started sessions.** A Fresh (not started) capsule never warns. Paused/Stopped labels and an Overtime label outrank the `wb-timer-low` / `wb-timer-limit` short forms; warning digit color still applies while paused/stopped. The long `wb-timer-yellow/red` text is appended to the capsule's accessible label.
- **Layout & navigation shows only while a document session has steps**, the same condition as the old bottom-strip trigger. A bare `LoadDocument` state without a workbench session has no trigger, so existing Tab-order regressions still hold.
- **Test-only seam `Shell::test_topology`** (`cfg(all(test, feature = "test-support"))`). It replaces the OS topology watcher check and keeps the injected display mapping, because TestPlatform display UUIDs are random. Without it, no headless test can create a real Start transaction from input. Product builds do not contain it.
- **Resume is guarded twice.** `Shell::start(Resume)` returns early if nothing is resumable, and core still rejects it with `NotReady`. The error type is `NotReady` (prd.md), not a new variant.

## trellis-check review (2026-10-20)

Re-ran every command above after the review edits: core 34 PASS; UI lib (`test-support`) 230 PASS; `clippy -p ferrostage-ui --features test-support --all-targets -D warnings` PASS; `just ci` fmt-check/clippy/test PASS, `deny` NOT_RUN (`cargo-deny` not installed; not installed by the check either).

- Lifecycle verified by reading every core/shell path: Esc/Stop button and `no_play` both apply `StopPresentation` (≥1 s → resumable); Home stops then `CloseDocument` (cleared); `finish_import` → `LoadMappedDocument` rebuilds idle (cleared, timer zero, so no stale frozen digits after a new PDF); both reload commit paths clone state and call `commit_reload` (kept). Cancelled/timed-out/failed Start transactions never reach core, so `resumable` survives. F5 during a pending Start resolves to the same timer kind and is refused by `start_reason` (busy); no second transaction.
- `test_topology` compiles only under `cfg(all(test, feature = "test-support"))`; product builds and default-feature CI never contain it. Accepted as the narrowest seam: TestPlatform display UUIDs are random and no watcher can be constructed headlessly.
- DESIGN.md aligned with implemented deviations: 4px visual gap between joined segments (overlapped focus reservations), 1px border-colored dividers instead of "hairline", Layout trigger shown when a document session is loaded.
- Added `shell::tests::split_session_control_stays_visible_in_narrow_windows`: Resume and Restart fully visible at 1100/800/390. At 320 only Resume is asserted: the bottom strip's `session-navigation` row is already wider than the window there (pre-existing, Start-mode/Previous/Next also clip), and Restart clips with it. Left as a known narrow-width gap, not fixed here.
- Centering: unchanged. 1280 (Ready and Presenting) is centered; below ~1100 Ready / ~1000 Presenting it degrades as documented. No cheap fix without folding actions into a menu (design.md §6), so PARTIAL stands.
