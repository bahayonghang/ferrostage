# Task07 I-2 maintenance evaluation

Authorized 2026-10-14 by the user's explicit ADR-008 reply. The existing task07 remains active. Preserve the original route fixtures, reports, hashes and all other workspace edits. Parent records and ADR coordination belong to Root.

## Smallest gap and ownership

The allowed GPUI sources compile but their required dependency graph fails the current policy. Obsolete text/parser/macro edges live in upstream GPUI and its dependency manifests/API uses. They cannot be fixed by changing FerroStage's UI callers or hiding audit targets.

The implementer owns an isolated candidate workspace under ignored `cache/gpui-maintenance-*`, and reviewable task07 `research/maintenance-*` artifacts: exact source identity, reproduction script, manifest/lock, source patch, regression fixtures/results, dependency matrix and evidence hashes. Do not modify the shared Cargo registry/Git cache source, original spikes, product manifests/crates/CI, `deny.toml`, or repository policy. Do not publish a fork. Copy only the necessary permissively licensed GPUI dependency source for evaluation; Zed's application/UI code and Pympress are not implementation inputs.

## Ordered work and evidence

1. Inspect required edges and maintained primary-source migration APIs. Prefer the exact tagged GPUI baseline and the smallest patch over an unrelated framework or a broad upstream rewrite. Record an actual source-level change boundary before coding. If a maintained upstream candidate demonstrably resolves the same edges, pin its full identity for comparison; a mutable branch is not a candidate identity.
2. Prepare the candidate with maintained resvg/usvg text dependencies and GPUI font lookup API, and remove the actual unmaintained macro dependency through a maintained upstream update or a narrowly scoped dependency patch. Do not disguise an unmaintained source by renaming its package. Preserve behavior; any proposed feature reduction is a decision to surface, not an automatic way to pass.
3. Add targeted regressions for the changed behavior: SVG text/shape output, font-face/glyph lookup and fallback, and image formats affected by the macro edge. Run the relevant upstream/harness tests on the candidate with source/fixture/runtime hashes. Do not claim a full GPUI or product runtime check from a standalone port test.
4. Build the Windows candidate and run appropriate lint/typecheck. Run the complete unchanged `cargo deny` policy over its resolved graph. Record each remaining license/advisory with exact package/version, edge, target and build/runtime use. A license-only failure is still FAIL; it does not block gathering authorized technical evidence for a reviewable policy decision.
5. Save deterministic reproduction inputs, minimal patch and exact commands/results. Check product inputs against the task06 final manifest. Hand off to an independent trellis-check review before a production or license decision. No code path is accepted merely because a command was launched.

If the patch grows beyond these dependency/API changes, explain the concrete new requirement and seek Root's design review before widening it. Preserve the full audit coverage, GPL/AGPL exclusion, unsafe-code rule and all native acceptance requirements. No advisory ignore, global allowlist broadening, target filtering or missing-library success shortcut.

## Return to product implementation

### Approved incremental source boundary

The stage1 full unchanged audit confirms a remaining active edge: gpui_platform → gpui_web → gpui_wgpu → cosmic-text0.19.0 → fontdb0.23 → ttf-parser0.25.1. Root approved evaluating the additional MIT OR Apache-2.0 cosmic-text copy with only its fontdb requirement moved to0.24; no feature removal. Fontdb0.24 is already required by usvg0.48. It vendors a limited ttf-parser0.25.1 metadata subset (table records/name/OS2), rather than eliminating every line of parser ancestry. Record that source/maintenance responsibility separately from RustSec package-node results. Validate real metadata, malformed input, collection indices, font fallback and shaping. The crate's native platform integration on non-Windows remains NOT_RUN.

Root also approved the original-tag Rust1.99 clippy compatibility change in GPUI `geometry.rs`: use `std::f32::consts::GOLDEN_RATIO` after a one-shot proof that it equals the old literal's f32 bits0x3fcf1bbd. No lint allow or geometry refactor. The initial mistaken PHI identifier and original lint failure remain evidence. The final technical candidate/report is ready for independent review; the unchanged full policy still fails three licenses. No production or license adoption is implied by completion of this plan.

Once the candidate and any separately reviewed license decision are acceptable, synchronize ADR-001, parent/shared plans, then integrate task07 I-2. Implement shell/theme only after that gate; apply the independently reviewed asynchronous activation/fullscreen confirmation constraints. This evaluation does not pass Gate B or authorize08–11 beyond their existing prerequisites. The user's unavailable physical display operations remain NOT_RUN.
