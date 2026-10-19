# Overlay and timing implementation evidence (2026-10-19)

Implementation complete for this child's production code and automated checks. Parent owns final full-workspace checks, native dual-display acceptance and version 1.1.0. No commit, release or task archive performed here.

## Implemented behavior

- Compiled overlays retain the existing document model and content/notes mapping. New original Beamer source/PDF exercises pause, only, uncover, visible, onslide, alt, temporal, incremental lists, built-in animate/animatevalue, multiple and overlay-specific independent notes.
- Navigation is allocated on accepted destination selection, then attached only after the assembled generation, content page, crop and exact audience target match the available public raster. Retained old pixels retain their previous identity. Initial/load/jump paths replace; sequential Next/Previous authorize the destination transition.
- Audience first-visible/settled events are fenced by current source window, generation, navigation and physical content page. One pure scheduler runs only from a later Shell poll turn. No recursive zero-duration stepping, missed-step backlog or final-page wrap.
- `/Dur` is independent from `/Trans /D`. Missing, negative, nonfinite or unrepresentable dwell falls back to manual; zero waits for paint and settlement; positive duration starts after successful settlement. This corrects the original plan's first-visible assumption to PDF Reference 1.4 Figure 8.1, printed page 487: https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.4.pdf . The PRD/design/implementation plan were updated accordingly.
- One contextual Pause/Resume animation button for timed documents; explicit pause persists across manual navigation, separate from the talk timer. Focus/modal/contextual-menu/blank/upload-error suspension preserves remaining dwell. Pointer/drawing settles the transition and explicitly pauses. Stop, reload preflight/structure/transaction, document generation and window replacement invalidate old events. Rejected reload recovery requires a fresh identity/paint.
- Bounded known JavaScript and SetOCGState action diagnostics (including catalog JavaScript names) use existing warning UI. Static optional-content layers alone are not treated as animated. No script execution, extraction/reflow of Notes, or raw script/note diagnostic output.

## Checks actually run

- `cargo test -p ferrostage-ui --features test-support --lib --locked --offline`: final 204 PASS (including actual GPUI audience paint/settlement, button behavior and reload preflight). Focused `timed_`: 6 PASS / 198 filtered.
- `cargo test -p ferrostage-ui --lib autoplay --locked --offline`: 6 fake-clock scheduler tests PASS. Cases cover Settled+Dur, pause during transition retaining full dwell, zero/later turn, stale events, explicit pause/navigation, temporary suspension/remainder, invalid/missing/final durations.
- `cargo test -p ferrostage-ui -p ferrostage-pdf --lib --locked --offline`: 67 PASS. Includes bounded active-content detection/cycles/static-layer distinction and duration parser cases.
- `cargo test -p ferrostage-document --test overlay_timing --locked --offline`: 1 PASS using checked-in compiled PDF and explicit label oracle.
- `$env:FERROSTAGE_TEST_PDFIUM=(Get-Item target/debug/pdfium.dll).FullName; cargo test -p ferrostage-document --features pdfium --test overlay_timing --locked --offline -- --nocapture`: 2 PASS. Actual PDFium labels agree with all 33 physical pages; all 16 content rasters change between steps, every independent notes raster remains distinct and has no playback metadata; forward/backward mapping and positive/zero/missing durations verified.
- `cargo clippy -p ferrostage-ui -p ferrostage-pdf -p ferrostage-document --all-targets --features ferrostage-ui/test-support --locked --offline -- -D warnings`: PASS after final scoped changes.
- `git diff --check`: PASS. Scoped rustfmt applied; parent performs final workspace fmt gate.
- Fixture compiled twice with local pdfTeX / TeX Live 2025, Beamer 3.72. Normal tests do not invoke TeX. PDF SHA256: `8e40489ef4fe594585b19e64c20fd22e958aeb94b801e4b5a7ad7016c59dedd9`.

## Native handoff and limits

`fixtures/beamer-timing/overlays-timing.pdf`: physical pages 18/20/22 have zero dwell, 24 is manual; 26 has 1.5 s dwell, 28 has Fade 0.3 s + zero dwell, 30 is manual; final content page 32 has zero dwell and must stay. Notes occupy intervening pages and must never appear in audience or Next preview. Parent should verify visible motion, input responsiveness, pause/resume and dual-display notes isolation. GPUI test drawing and PDFium raster tests are not physical monitor visual acceptance; native manual cases remain pending parent evidence.

No advertised support for the separate animate package, arbitrary JavaScript engines or exhaustive active-content recognition. Existing ambiguous/missing-label physical fallback stays unchanged, with prior document regression retained.

## Touched files for this child

- `crates/ferrostage-ui/src/autoplay.rs`, `lib.rs`, `actions.rs`
- `crates/ferrostage-ui/src/shell.rs`, `shell/timed_playback.rs`, `shell/timed_playback_tests.rs`, `shell/reload.rs`
- `crates/ferrostage-ui/src/presenter.rs`, `presenter/workbench_view.rs`, `presenter/layout_navigation.rs`
- `assets/locales/en-US/app.toml`, `assets/locales/zh-CN/app.toml`
- `crates/ferrostage-pdf/src/structure.rs`, `structure/active_content.rs`, `structure/presentation.rs` (duration bound/test only; transition worker owns other parser implementation)
- `crates/ferrostage-document/tests/overlay_timing.rs`
- `fixtures/beamer-timing/overlays-timing.tex`, `overlays-timing.pdf`, `expected.json`, `README.md`
- `docs/pdf-transitions.md` (overlay/timing/known-active-content compatibility addition)
- This task's `prd.md`, `design.md`, `implement.md`, and this evidence.

## Final independent review repair

The independent reviewer found and fixed a failed-worker recovery gate omission in
`shell/reload.rs`: playback now remains cancelled while `reload.failed_worker` is
latched, even after the failed pending transaction disappears. The new regression
failed before repair and passed afterward. Focused `timed_reload_` tests: 3 PASS,
202 filtered (205 tests now in the UI inventory). Native-feature all-target UI
Clippy with `-D warnings`, scoped formatting and `git diff --check` also PASS after
the repair. The original 204-test receipt above remains the worker's historical
full-suite result. See the parent's `research/review-final.md` for integrated-flow
review and `.trellis/spec/frontend/pdf-playback.md` for the completed contract.
