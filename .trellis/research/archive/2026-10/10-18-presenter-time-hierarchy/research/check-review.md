# Independent implementation review

Reviewed the integrated task against PRD AC1-AC16, design.md, page-browser-design.md, implement.md and the frontend/document contracts. This report distinguishes source/headless verification from the main session's native evidence.

## Fixed finding

- AC16: clearing Workbench thumbnail frames on numbering changes did not retire Presenter GPU rollback images. `Surface::Thumbnail(index)` changes physical meaning between Content and PDF numbering; `picture()` could obtain old pixels through `TextureCache::frame`, while `TextureCache::sync` retained that index from requested surfaces. An unavailable replacement raster/upload could therefore leave an old page beneath a new caption.
- Fix: `presenter.rs:767` observes the existing per-open browser identity before texture synchronization. On change, synchronize non-thumbnail frames/surfaces first to retire thumbnail rollback slots, then synchronize current frames normally. Current/next/notes caching remains on its existing path; no generic texture-cache API changes.
- Regression: `shell/presenter_time_tests.rs:4`, `numbering_switch_retires_cached_thumbnail_pixels_before_new_rasters`, populates the actual presenter image cache, switches modes before replacement raster completion and checks the remapped slot cannot draw its old surface. Without the fix it fails at the old-GPU-image assertion (0 passed, 1 failed); restored fix passes.

## Other checked paths

- Mode projection is shared by input, targets, captions, counts, reveal/group movement and scheduling; content uses ordered steps, PDF uses all physical positions, with notes placeholders excluded from thumbnail requests.
- Mode changes renew interaction identity and retain annotations, timer, grouping and settings; stale edit/submit/tile/mode intents are rejected. New import resets mode, reopen retains it and hot reload preserves it while retiring old interaction state.
- Notes input has exact Chinese feedback `备注页不可投影`, disabled Go and no redirection. Direct submit still validates membership in the public content map.
- Safe fresh No split defaults retain notes recognition; explicit Automatic and directional detected labels remain supported independently from the three visible menu choices.
- The standalone Navigate tool is absent; all four remaining tools retain toggle-off cleanup. Clock placement, numeric reservation, semantic labels and existing timer dispatch are covered by focused tests.
- The existing focus-reveal fix now runs through Presenter render lifecycle; the final-tile test delivers scheduled frames and checks visible bounds.

## Verification

- `just ci`: PASS, exit 0 after final changes; default workspace fmt, strict Clippy, all tests and cargo deny. Existing deny duplicate/unmatched-exception warnings remain, no new waiver.
- `just check-shell`: PASS, exit 0 after final changes; actual GPUI/PDFium app build, strict feature Clippy, and all 166 GPUI tests (0 failed; test runtime 1.19s).
- Focused cache regression: RED without fix, GREEN with fix and included in final full suite.
- `git diff --check`: PASS.
- Isolated native PDFium retention and initial-load acceptance receipts belong to the implement agent; final native visual/mode smoke belongs to the main session. No source/headless test is claimed as native visual acceptance.

No remaining concrete product-code defect found. Main owns final spec wording, task acceptance/evidence, and the refreshed-executable native smoke. The untracked presenter_time_tests.rs must remain part of the eventual scoped change; no commit/push/archive was performed by this review.
