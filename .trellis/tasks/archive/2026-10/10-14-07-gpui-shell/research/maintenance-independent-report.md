# Independent maintenance review — repaired candidate, adoption still gated

The review found and repaired two real compatibility regressions. The final candidate is reproducible and its relevant tests/build/type checks pass. **The unchanged full policy still fails exactly three licenses, and direct HarfRust strict lint retains six independently reproduced upstream errors.** This is a reviewable maintenance candidate, not task07 completion, production adoption, license approval or native UI acceptance.

## Authoritative final candidate and recovery

Use **`maintenance-independent-prepare-final.py`** as the single reconstruction entry point from the repository root:

```powershell
python -X utf8 .trellis/tasks/10-14-07-gpui-shell/research/maintenance-independent-prepare-final.py
```

It writes `cache/gpui-maintenance-independent-final-reproduction`, preserves the tested candidate, and verifies all526 final files. The verified order is: frozen original preparation → `maintenance-independent-repair.py` (GPUI helper/tests) → `maintenance-independent-harfrust-repair.py` (HarfRust selection/mapping/tests and three patch declarations) → retained final locks → complete source hash comparison. The fresh run returned0 and matched all526 paths; its original441-file intermediate also matched. See `maintenance-independent-final-reproduction.json` and `maintenance-independent-final-preparation/commands.json`.

**The original `maintenance-prepare.py` or `maintenance-source.patch` alone reconstructs the defective historical candidate.** They are retained as provenance, not the final recovery recipe. `maintenance-independent-final-source.patch` combines the complete maintained-package source/manifest changes against their original inputs; unmodified upstream files, evaluation scaffolding, fixtures and locks are reconstructed by the entry point and covered by the complete source snapshot. The final locks/manifests/notices and diagnostic source/lock are under `maintenance-independent-retained/`.

| Integrity item | SHA256 |
| --- | --- |
| Frozen original manifest | `a0316c9af7a46dfd4f8bb331520c1fb3d4938b1468b02c27de77b2b470e81b0f` |
| Frozen original patch | `d907642badb418203c3ca97472d2acd464fd4025e9b756b7c3f4fef0d98f9440` |
| Final526-file source snapshot | `cc9a8ded6bfeeed1c0286ba5bf2e3ecf750b6ce34987f11d4b6e23d85ec8a960` |
| Combined maintained-package diff | `003fd8d088ce9d5466c059608cf3990418cb2b4fdb3094798b1407b0f70b0a83` |

`maintenance-independent-manifest.json` is the final review integrity index, including this report, exact source snapshot, all review logs/scripts, retained locks/notices, inspected primary-source files, toolchain and generated binaries. Its sidecar `maintenance-independent-manifest.sha256` provides its own digest. Recompute with `maintenance-independent-capture.py final`; that operation requires the source snapshot to still match.

## Findings (fixed)

1. **GPUI `src/svg_renderer.rs`: coverage probe changed semantics.** Replacing `ttf_parser::Face::glyph_index` with `skrifa::Charmap::map` selected one preferred cmap (including symbols) instead of traversing the former Unicode records; skrifa's high-level map also omitted formats2/13. The retained real-font-backed red probe failed four cases: format2, format13, symbol priority and symbol-only coverage. Cargo exited101 in `maintenance-independent-cmap-probe-red.log`.

   The repair uses the maintained raw cmap APIs, restores the original platform/encoding filter and record order, and adds a small private format2 adapter using checked `FontData`/`SubHeader` reads. No unsafe or old font-wide parser was copied. The original five cases and28 expanded cases now agree with the original parser. Tests cover formats0/2/4/6/10/12/13, a later Unicode record, symbol/Mac exclusion, Windows encoding rules, single/double-byte format2, missing/non-BMP characters, zero glyph before/after delta, wrap/negative delta, truncation and invalid offsets. Existing `Option`/`is_some` quirks are deliberately retained: several formats can report `Some(0)`, and the old negative-remainder rejection is preserved. These are compatibility choices, not new claims about the OpenType recommendation.

2. **HarfRust0.12.0: actual format2 rendering remained broken after the probe repair.** The strict SVG comparison reproduced loss of the original outline: old resvg/usvg0.46.0 painted268 pixels with one fallback; the new stack painted304 with two fallbacks and different RGBA bytes. The format13 control matched exactly. `maintenance-independent-render-probe.log` retains the exit101. A preceding nonempty-pixel-only probe returned0 but was insufficient; its source/log are retained with `insufficient-pixel-count` in the name and are not rendering-equivalence evidence.

   Root approved a fifth maintained package, **MIT HarfRust0.12.0**, confined to `src/hb/tables.rs` and `src/hb/charmap.rs`. The patch admits format2 without changing encoding priority, routes the cached and alternate `from_tables` constructors through the same internal selector, and adds checked format2 mapping. It does not change public APIs, GSUB/GPOS, outlines, normalization or read-fonts. Direct tests compare exact glyph/None/zero results through both constructors. Strict old/new SVG fallback now produces268 pixels, one fallback, and identical entire RGBA buffers for both formats2 and13. The real GPUI renderer separately verifies the same outline through `select_emoji_font`. The full probe exits0; all205 HarfRust library tests and16 GPUI SVG tests pass.

The format2 offset calculation is documented by the [OpenType specification](https://learn.microsoft.com/en-us/typography/opentype/spec/cmap#format-2-high-byte-mapping-through-table). The GPUI multi-record presence check restores its existing behavior; the renderer retains its established single-table selection priority. The two local adapters now require maintenance when these dependencies change. The read-only scope assessment is preserved in `maintenance-independent-format2-scope.md`; its “proposed” status describes the pre-approval checkpoint, superseded by the implementation and evidence here.

## Findings (not fixed)

- **Full unchanged license policy: FAIL / exit4.** `arrayref0.3.9` BSD-2-Clause and `libbz2-rs-sys0.2.5` bzip2-1.0.6 occur in the Windows normal graph; `cbindgen0.28.0` MPL-2.0 is an Apple build dependency in the preserved full graph. Final log: `maintenance-independent-final-deny.log`, ending `advisories ok, bans ok, licenses FAILED, sources ok`. Exactly three rejected records remain. There is no ignore, target exclusion, broad allow, source renaming or private-package exemption. No local exception file exists in the checked root/candidate locations.
- **Direct upstream HarfRust strict lint: FAIL / exit101, no new diagnostics.** The patched and untouched isolated0.12.0 copies produce exactly the same six message/location pairs: unused `shape` at `hb/face.rs:439`, `manual_midpoint` at `hb/aat/layout_trak_table.rs:157`, two `question_mark` findings at `hb/glyph_metrics.rs:199/247`, and two at `hb/unicode.rs:614/631`. `maintenance-independent-harfrust-lint-comparison.json` proves the exact match. These files are outside the two-file approved repair; no unrelated algorithms, source attributes or lint policy were changed. The deprecated upstream manifest lint spelling is also retained. Root and GPUI strict lint pass separately; this report does not claim all lint passes.
- Native Apple/Linux/Web execution, GPU/window/focus/fullscreen/display migration, theme UI, real single/zero-screen behavior and physical unplug/interface changes are **NOT_RUN**. The WGPU consumer check is Windows host API compilation only. Task07 I-2 product linkage and subsequent work still require the explicit candidate/license decision and product gates. Gate B/C and unavailable physical operations remain unchanged.

## Verification and evidence boundaries

Exact commands, timestamps, exits and log paths are in `maintenance-independent-status.txt`, `maintenance-independent-command-results.json` and `maintenance-independent-final-command-results.json`. Final builds ran sequentially; no duplicate Cargo work or window launch occurred. Environment: Windows x86_64 MSVC, pinned Rust/Cargo1.99.0; precise identities are captured by the final manifest and process-state record. The CIM query was unavailable, so OS identity uses .NET runtime/environment data.

| Check | Final result | Evidence |
| --- | --- | --- |
| Actual GPUI SVG/font tests, no default features, locked/offline | 16 pass | `maintenance-independent-final-gpui-tests.log` |
| Root image/font/cosmic harness, locked/offline | 7 pass | `maintenance-independent-final-root-tests.log` |
| Copied pulp upstream tests | 17 pass | `maintenance-independent-pulp-tests.log` |
| Unmodified fontdb upstream integration | 1 pass | `maintenance-independent-fontdb-tests.log` |
| Copied cosmic upstream library tests | 4 pass | `maintenance-independent-cosmic-tests.log` |
| Patched HarfRust full library tests | 205 pass, including the two-constructor format2 case | `maintenance-independent-final-harfrust-tests.log` |
| Separate original/repaired parser and strict SVG probes | 4 pass; five original cases plus28 edge cases and two full-pixel controls | `maintenance-independent-full-probe-final.log` |
| Root and direct GPUI `clippy ... -- -D warnings` | exit0 each | `maintenance-independent-final-root-clippy.log`, `maintenance-independent-final-gpui-clippy.log` |
| Direct HarfRust same strict lint, patched and untouched copies | exit101 each, same six existing diagnostics | final/baseline HarfRust clippy logs and comparison JSON |
| Root all-targets check; direct WGPU consumer host check | exit0 each, locked/offline | final root/WGPU typecheck logs |
| Final native Windows candidate build | exit0, locked/offline | `maintenance-independent-final-root-build.log` |
| Modified GPUI/HarfRust Rust formatting | exit0 | status record |
| Full unchanged `cargo deny ... --config deny.toml --locked --offline check` | **exit4 / FAIL** | `maintenance-independent-final-deny.log` |
| Fresh complete final source reconstruction and hash audit | exit0,526/526 equal | final reproduction JSON |

These are separate test sets; the four diagnostic tests include repeated coverage comparisons and are not added to the library counts. Pulp/fontdb/cosmic test results remain applicable after the HarfRust repair because their sources/locks and the older cosmic HarfRust branch did not change. EXR checks cover Uncompressed/RLE/ZIP16/PIZ and arbitrary131×67 SIMD tails; font tests cover fallback, collection indices and malformed metadata. The tests do not prove all codecs, all fonts, every shaping result or every platform. The strict pixel cases use in-memory cmap derivatives with original real outlines, not redistributed modified fonts. The alternate constructor test directly exercises `Charmap::from_tables` with a table provider; it is not an end-to-end native FontInstance runtime test.

## Provenance, preservation and policy proposal review

The initial handoff independently matched all441 candidate source files,99 maintenance artifacts and7 generated files. All12 original registry archives match the frozen lock checksums, and all2,004 extracted file bytes match the shared registry source. This includes the original MIT HarfRust0.12.0 archive (`c03d949a14aa089bbb282f7dd76a498a7f684428e4257202efc119ec010376f9`), now copied locally with its original license. Exact tagged Git diff returned0 and status was empty for GPUI/workspace metadata, the12 inherited manifests and both original font directories. The tag/revision remains `v1.23.3` / `96a1b1b5389efff473fc2306f7f8c690f2e1a417`.

`maintenance-independent-final-package-comparison.json` verifies the five maintained packages: GPUI changes only manifest/SVG helper/tests/bit-identical geometry constant; pulp/metal/cosmic change only their approved manifests and isolated locks/notices; HarfRust changes only its standalone manifest, two source files/tests and notice. The three updated consumer locks have no package/version additions or removals: only HarfRust0.12.0 changes from the verified registry source to the local copy. Full feature boundaries, SVGZ and image codecs remain as in the reviewed handoff. The fixed golden-ratio constant retains the recorded `0x3fcf1bbd` value.

Fontdb0.24 still includes its maintained upstream metadata subset derived from ttf-parser0.25.1. Absence of the old package node is not absence of all old source ancestry, a security proof or a vulnerability finding. The diagnostic comparison workspace intentionally contains the original parser/renderer outside the candidate graph. The full audit applies to the candidate, not this diagnostic workspace.

Original OFL notices, copyright/RFN declarations, original font hashes, TTC input hashes and Tuffy notice remain retained. Original and generated font binaries stay in ignored cache; the independent audit finds no `.ttf`, `.otf` or `.ttc` under task research. Initial cache/network failures, the Cargo inactive-target resolver panic, mistaken PHI proof and preparation/old-source lint run remain untouched. New review failures are retained too: original red coverage/rendering tests, the insufficient nonempty-pixel check, an omitted fixture in the first reviewer reproduction (fixed in its driver), Git/cache permission failures, offline missing HarfRust dev dependency, the initial selector-visibility compile error, and final-audit checksum lookup against the now-local HarfRust lock (corrected to the frozen retained registry lock). None is labeled a pass. A first lint-log comparison read hit UTF16 encoding; the corrected comparison handles the recorded PowerShell encoding.

The parent `gpui-license-review.md` proposal is technically bounded to the exact three package/version/license pairs above, consistent with cargo-deny's [per-package exceptions](https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html) and [PackageSpec rules](https://embarkstudios.github.io/cargo-deny/checks/cfg.html). No proposal was applied. The adoption materials must include this fifth maintained package, the upstream direct lint limitation and this final reconstruction set; they cannot approve the original patch alone. Root owns that decision and the corresponding ADR/parent updates.

Task06's final manifest remains SHA256 `6d00530903376138990099497bf797ee847aac6e6b233905a7da76dbfdcadf47`. All117 baseline inputs,43 evidence files and10 external inputs still match; all92 classified product inputs match, and all86 current crate files are covered. The original99 route evidence records still match either their current bytes or the two explicitly preserved pre-review document versions. Thus prior128 default/63 native product tests remain applicable; they were not rerun or relabeled as task07 acceptance. Product Cargo/lock/crates/CI/deny are unchanged. No commit, push, archive, task switch or journal write occurred. All reviewer commands have completed, with no active Cargo/rustc/rustfmt process or reviewer tool session at closeout.
