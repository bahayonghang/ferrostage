# Integrated validation ledger

User approved implementation of the complete revised plan on 2026-10-19. This ledger separates source evidence, automated checks, and actual native acceptance. PENDING is not PASS.

| Area | Required evidence | Status |
| --- | --- | --- |
| Notes source | Stable 95 frame payloads, repeat synchronization, compiled labels and formulas | PASS; Notes child implementation-evidence.md |
| Notes layout | All pages rendered; dense/formula pages inspected; no clipping | PASS; Notes child implementation-evidence.md |
| Public thesis slides | All 95 audience rasters compared with preserved baseline | PASS; all 95 RGB pixel buffers identical |
| Fullscreen | Native Start/Stop and state restoration; failed Start regression | PASS normal/maximized/already-fullscreen native; deterministic rollback tests PASS; forced native failure NOT_RUN |
| Presenter layout | Current left, Next upper-right, original Notes lower-right; essential controls | PASS normal and full-screen laptop; dense Notes and independent multi-Notes scrolling observed |
| Transitions | Twelve standard styles, authored parameter geometry and interruption | Automated PASS; all 12 families sampled on actual audience; exhaustive native variants/recording NOT_RUN |
| Timed overlays | Positive/zero/missing dwell, rendered-step gating, pause and lifecycle cancellation | Automated PASS; native manual steps, zero-chain final, positive pause/resume and final stability PASS; per-step OS scanout NOT_RUN |
| Privacy | Notes excluded from snapshots/navigation/retained frames/diagnostics | PASS code/fixture checks and actual public-only audience samples |
| Default gates | just ci | PASS at 1.1.0: 223 tests, fmt, strict Clippy and deny |
| Desktop gates | just check-shell and relevant native lint/PDF tests | PASS at 1.1.0: 205 UI tests, actual native build/strict Clippy; earlier actual-PDFium test-native PASS; all-feature deny PASS |
| Independent check | Trellis check review and fixes | PASS; enlarged Fly, deferred paint acknowledgement and failed-worker timing defects fixed with regressions; review-final.md |
| Version | All workspace metadata and lock versions 1.1.0 | PASS ten workspace packages; external dependencies unchanged |
| Distribution | just release, receipt and SHA256 verification | PASS Setup/ZIP/checksums, 22 inventory files; actual packaged 1.1.0 thesis-open/fullscreen smoke PASS |
| Standing readability | Actual user posture/distance judgment | NOT_RUN; cannot infer from font size |
| Install/publish/signing/clean-machine | Outside this request | NOT_RUN |

## Delivery

Implementation and local 1.1.0 packaging are complete. Functional tasks remain in review for the explicitly unrun native qualification and human standing-distance judgment, rather than marking every original acceptance clause as fully verified. No code blocker remains from independent review. No commit, archive or publication has been performed.

- Native evidence: [native-validation.md](native-validation.md).
- Release evidence: [release-validation.md](../../10-19-release-version-1-1-0/research/release-validation.md).
- Notes source/render evidence: [implementation-evidence.md](../../10-19-presenter-readable-notes/research/implementation-evidence.md).
- Final code review: [review-final.md](review-final.md).
- Full pre-release native logs: `C:/Users/lyh/AppData/Local/Temp/ferrostage-defense-1-1-0/`.
- Final 1.1.0 gate and release logs: `C:/Users/lyh/AppData/Local/Temp/ferrostage-1.1.0-validation-20261019/`.

## Evidence rules

- The app uses an isolated configuration for native checks; original user settings and history are not test fixtures.
- The app reads the thesis PDF. Its authorized compiler regenerates it from source.
- Do not copy private thesis note text into this repository. Keep local raster comparisons in the thesis analysis directory.
- Record failures and unavailable native scenarios honestly. Headless geometry and a successful build do not prove native window behavior or an animation's appearance.
