# Sequential implementation checkpoints

User approval: 2026-10-18, explicit sequential Trellis sub-agents, then review and normal-window native core/visual verification. No dimensions/DPI/performance qualification expansion. No commit, push or archive requested. Branch: `codex/workbench-settings-design`, based on home commit `7c25e46`.

## Baseline

Coordinator launched the pre-existing home executable with task-owned `baseline.toml` and read-only `fixtures/pdf/perf-100.pdf`. Executable SHA256 matches the previous home receipt: `2C109A1CC8CC944DE2AB3F11F24FC0C605E1921B6E6480E404A2C8E739DB3E9A`. Saved native before-workbench, before-general and before-displays JPEG/metadata under `research/native`. Process 62260 exited 0. This preserves the observed before state; no size/DPI probes were performed.

## Settings child

Trellis implement sub-agent completed schema/parser/merge, cached installed fonts, UI typography, compact settings and history. Its `evidence.md` records 42 settings, 35 default UI and 15 focused GPUI settings tests plus strict native clippy passing. Native installed-font/persistence/aesthetic acceptance and independent review await final integration. Displays nested-body cleanup is intentionally deferred to the displays child.

## Workbench child

Activated and dispatched after settings implementation returned. Completed header/progress, unified session/tool strip, semantic controls and related tests/design while preserving the shared settings baseline. Its evidence records 35 default UI, 12 workbench, 11 Presenter neighbor tests and strict native clippy passing. Native acceptance and independent full-diff review remain pending.

## Displays child

Activated and dispatched after the workbench handoff. Owns compact shared display body/host framing, actual/configured/draft projection and lifecycle observations, allocation feedback and related tests. Native capture and final integration checks remain coordinator-owned.

Implementation checkpoint returned: 38 default UI tests, 22 display-focused tests, strict native clippy and fmt/diff checks pass. The callback/current-owner and one ordinary two-host scenario were added; no matrix expansion. Actual Windows movement and ordinary-launch density are not yet claimed.

## Independent integration review

Parent activated after the third implementer finished. A separate Trellis check agent owns the full approved diff review, scoped repairs and final `just ci` / `just check-shell` gates. Coordinator owns actual desktop interaction after that final build, so source and executable identity remain comparable.

## Final integration receipt

All three implementation handoffs and independent review completed in order. Native findings were repaired and regression-tested, then `just ci` passed194 default tests and `just check-shell` passed native build/strict lint and136 GPUI tests on final executable `0ecb9c1c...`. The coordinator completed actual ordinary-window core/visual checks, font restart/default/fallback, recent path containment, display selection/Cancel/Apply/movement, audience lifecycle and dark/light inspection. Final sources match the build manifest and the PDF fixture matches baseline. See `verification.md` for AC mapping, identities, screenshots and exact limits.

Native clipboard paste remains inconclusive: automation also failed in the OS filename edit, while ordinary filtering works. It is not marked native PASS. No size/DPI/performance expansion, commit, push or archive. Tasks stay in progress awaiting the separately authorized commit/finish lifecycle; implementation and requested core acceptance are recorded here as complete.
