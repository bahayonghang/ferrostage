# Independent review, stage 2

2026-10-19. Scope: page-transition metadata, content mapping, pure composition,
Audience paint/event lifecycle and texture-cache resource handling. Shell scheduler
integration was actively owned by another worker. Parent explicitly assigned the
two bounded repairs below after the initial read-only findings; no Shell code was
modified by this reviewer.

## Findings (fixed)

### 1. Enlarged Fly content did not reach offscreen endpoints

- File: `crates/ferrostage-ui/src/transitions.rs`, Fly composition.
- Issue: fixed one-page translation was insufficient when `/SS > 1`. An inward
  rightward `/SS 3` effect immediately after progress zero produced an image near
  `x=-2, width=3`, leaving one full page width visible. The exact endpoint branch
  hid the error, creating an abrupt appearance; outward motion had the equivalent
  disappearance. This violates Fly's offscreen endpoint except for `/Di /None`.
- Fix: derive centered-image travel from `(1 + authored_scale) / 2`, retaining the
  existing minimum one-page travel for ordinary/smaller scales. Scale-only motion
  stays stationary. Added regression for all cardinal directions, both motions and
  scales 0/0.3/1/3, sampling just inside each endpoint instead of testing only the
  special endpoint branches.
- Fixture: appended eight enlarged Fly cases to both generated PDFs. Existing
  first 43 pages retain their order; 51 pages now cover the larger-scale variants.
  Parser fixture assertions and compatibility documentation match the new count.

### 2. Blank publication could permanently consume deferred acknowledgements

- Files: `crates/ferrostage-ui/src/audience/playback.rs` and `playback/tests.rs`.
- Issue: `painted` set `first_visible`/`settled` before deferred event delivery.
  If a blank snapshot arrived before the callback, the visibility fence correctly
  discarded the event, but unblanking with the same navigation identity could
  never send it again. A scheduler waiting for that phase would remain waiting.
- Fix: commit each delivered-phase flag inside the eligible deferred callback,
  checking both current snapshot and playback identity. Repeated queued paints
  still deliver each phase once; a callback rejected because of blanking leaves
  the phase eligible for the next visible paint.
- Regression: queue both acknowledgements, publish black before deferred delivery,
  verify no event, then unblank the same identity and queue two paints. Exactly
  one FirstVisible and one Settled are delivered.

Both new regressions failed against the original mechanism before repair. Fly
failed with the visible enlarged rectangle; the acknowledgement regression got
zero events instead of two. RTK receipts: `1792385971_cargo_test.log` and
`1792385978_cargo_test.log` under the local RTK tee directory.

## Findings (not fixed by this reviewer)

- **Timing design versus standard:** source review of Adobe's presentation timing
  figure showed that `/Dur` follows completion of the incoming transition, whereas
  the original approved design started it at the first effect frame. Parent accepted
  the correction and assigned `Settled.at + Dur` plus corresponding tests/spec changes
  to the active timing worker. This reviewer did not change its shared scheduler.
  See [Adobe PDF reference, Figure 8.1, printed page 487](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.4.pdf).
- Native recordings for all advertised effect families, enlarged Fly, dual-display
  interrupts and the thesis example remain parent-owned. Pure geometry and actual
  headless GPUI scene/atlas assertions cannot establish displayed Windows output.
- Final Shell identity/clock integration remains the next review stage. Stage 2
  is not a claim that all parent acceptance criteria are complete.

## Other source checks

All 12 style branches are distinct; Split/Blinds orientation, Box in/out, directed
Wipe, Push/Cover/Uncover stationary layers and Fly scale/motion match the inspected
mechanisms. Fly prepares a single changed-pixel image; `/B` fills the changed
bounding rectangle. Dissolve/Glitter use bounded deterministic spatial cells.
These semantics were checked against [ISO 32000-1 Table 162](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf).

The document lookup gates presentation metadata through actual content mapping;
independent notes cannot acquire playback. Audience receives only the public
snapshot. Incoming preupload must succeed before effect playback; failed incoming
upload retains old public pixels and emits no destination acknowledgement. Retained
old/new/difference allocations use the existing texture cache and budget checks;
blanking/staging/media cancel effects, and final cleanup releases outgoing/difference
images. No new PDF rendering or full-image CPU copy occurs on each animation tick.

## Verification

- PASS: targeted enlarged Fly regression (after demonstrated failure).
- PASS: `cargo test -p ferrostage-ui --features test-support --lib audience::playback --locked --offline`, 3 tests, including the new delivery race and existing actual GPUI scene/atlas failure/cleanup test.
- PASS: `cargo test -p ferrostage-ui --features test-support --lib transitions::tests --locked --offline`, 4 tests.
- PASS: `cargo test -p ferrostage-pdf --lib fixture_covers_parameters_and_indirect_transition_dictionaries --locked --offline`, 1 test, 51-page fixture.
- Lint / native type-check PASS: `cargo clippy -p ferrostage-ui --all-targets --features gpui,pdfium --locked --offline -- -D warnings`.
- PASS: scoped Rust formatting checks for the three owned UI files and working-tree
  `git diff --check`. The shared PDF parser file's newly added dwell test still
  needed the timing worker's formatting pass; that owner was notified.
- Initial test attempts encountered the timing worker's temporarily inconsistent
  `settled(at)` signatures while its edits were in progress. Those are not reported
  as product failures or passing runs; successful red/green runs followed its update.
- Full workspace checks and native visual qualification were not repeated here.
