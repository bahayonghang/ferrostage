# Task07 dependency gate — blocked handoff

Captured 2026-10-14. Both permitted routes compile as isolated Windows MSVC spikes, but both fail the repository's unchanged `cargo deny` gate. The smaller `gpui-pre` engine/platform graph also fails without kit/base/component, so optional styled components do not explain away the blocker. Product `Cargo.toml`, `Cargo.lock`, crate sources, CI, `justfile` and `deny.toml` remain unchanged from task06's independently checked compilation inputs. Task07 I-2 integration and I-3–I-7 are not delivered.

## Tested fixtures and reproducibility

All spikes are independent workspaces under ignored `cache/`; no dependency was added to FerroStage. Copies of each `Cargo.toml`, `Cargo.lock` and `src/main.rs` are retained under `research/route-fixtures/`. `evidence-manifest.json` records the original paths, hashes, binaries, diagnostic sources and logs. Keep the manifest/lock/source together when reproducing; no executable has been launched.

| Route | Original workspace | Exact dependency inputs | Main source |
| --- | --- | --- | --- |
| A, tagged | `cache/gpui-route-spike` | `gpui` + `gpui_platform`, Git tag `v1.23.3`, both `default-features=false` | `let _application = gpui_platform::application();` |
| B, kit/component | `cache/gpui-kit-route-spike` | `gpui-pre` + `gpui-pre-platform =0.3.9`, `gpui-component =0.7.3`, `gpui-kit =0.7.3`; defaults disabled, kit `component` enabled | `let _application = gpui_kit::application();` |
| C, minimal pre | `cache/gpui-pre-minimal-route` | only `gpui-pre` + `gpui-pre-platform =0.3.9`, both defaults disabled | `let _application = gpui_platform::application();` |

A resolves the permitted tag to `96a1b1b5389efff473fc2306f7f8c690f2e1a417`; its package version `gpui 0.2.2` does not mean crates.io 0.2.x was substituted. B/C registry package metadata identifies upstream Zed `93d745668805029476e6d7fbb19652116fed51de`. Exact registry checksums are in each lockfile. Toolchain: rustc1.99.0, cargo1.99.0, host `x86_64-pc-windows-msvc`, cargo-deny0.20.2; full versions in `toolchain.log`.

## Actual commands and exits

Commands ran from repository root. `route-spike-status.txt` records RUNNING before the long operations and actual returned exits. Cache/network retry used approved escalation; no concurrent duplicate build was started. Initial failures are retained rather than replaced by the successful retry.

| Exact command | Result | Raw log |
| --- | --- | --- |
| `cargo build --manifest-path cache/gpui-route-spike/Cargo.toml` (sandbox) | exit101: cannot create Cargo Git-cache directory, access denied | `route-spike-build-initial.log` |
| Same build, approved retry | exit0, 3m20s; session28920 completed | `route-spike-build.log` |
| `cargo deny --manifest-path cache/gpui-route-spike/Cargo.toml check --config deny.toml` | exit2: wrong CLI argument placement; no policy conclusion | `route-spike-deny.log` |
| `cargo deny --manifest-path cache/gpui-route-spike/Cargo.toml --config deny.toml --locked check` | exit5; also exposed missing license metadata on our spike | `route-spike-deny-corrected.log` |
| `cargo deny --manifest-path cache/gpui-route-spike/Cargo.toml --config deny.toml --locked --offline check` | exit5, authoritative A audit after fixture license correction | `route-spike-deny-final.log` |
| `cargo build --manifest-path cache/gpui-kit-route-spike/Cargo.toml` | exit0, 1m44s; session94324 completed | `kit-route-build.log` |
| `cargo deny --manifest-path cache/gpui-kit-route-spike/Cargo.toml --config deny.toml --locked check` | exit5; session94991 completed | `kit-route-deny.log` |
| `cargo deny --manifest-path cache/gpui-pre-minimal-route/Cargo.toml --config deny.toml check` | exit5; session17355 completed | `pre-minimal-deny.log` |

Only our initial spike package lacked a `license` field; adding `license = "MIT OR Apache-2.0"` corrected that fixture. The final A audit has no spike-license failure. No product dependency license, allowlist, ignore or policy was changed. B/C had the license field from creation. C was not redundantly compiled: its exact minimal dependency graph was resolved and audited to establish unavoidable findings.

For each A/B/C manifest, these diagnostic commands returned exit0:

```text
cargo tree --manifest-path <manifest> --locked --offline --target x86_64-pc-windows-msvc --edges normal,build --prefix none --format '{p}'
cargo metadata --manifest-path <manifest> --locked --offline --no-deps --format-version 1
```

Outputs are `tagged-windows-tree.log`, `kit-windows-tree.log`, `pre-minimal-windows-tree.log` and corresponding `*-metadata.json`. The target-specific trees explain actual Windows normal/build membership; they do not replace or narrow the full policy audit. Every authoritative audit says `advisories FAILED, bans ok, licenses FAILED, sources ok`. Duplicate-version and unknown-Git warnings are not the failing conditions.

## License findings and target/build attribution

Existing `deny.toml` has a short explicit list: MIT, Apache-2.0, Apache-2.0 WITH LLVM-exception, BSD-3-Clause, ISC, Unicode-3.0, Zlib and CC0-1.0. It does not allow any license below; private crates are not ignored. ADR005 explains the restrictive baseline and GPL/AGPL exclusion. No allowlist broadening or target-coverage reduction is authorized.

| Crate/version | License | Routes | Dependency path and actual attribution |
| --- | --- | --- | --- |
| arrayref0.3.9 | BSD-2-Clause | A/B/C | `tiny-skia` / `tiny-skia-path0.11.4` → `resvg` / `usvg0.46.0` → GPUI/pre; B also component's resvg0.45.1. Present in Windows normal graph. |
| libbz2-rs-sys0.2.5 | bzip2-1.0.6 | A | `bzip2 0.6.1` → `compression-codecs0.4.45` → `async-compression0.4.50` → Zed `http_client` → GPUI. Windows normal graph. |
| cbindgen0.28.0 | MPL-2.0 | A/B/C | **Build dependency** of `gpui_apple` / `gpui-pre-apple` under Apple target cfg → macOS platform. Absent from actual Windows tree; retained in full audit. |
| dwrote0.11.5 | MPL-2.0 | B | `zed-font-kit0.14.1-zed` → pre Apple/WGPU backends. dwrote is font-kit's Windows-target edge, but those parent backend combinations are not selected by this Windows build; absent actual Windows tree. Present in full policy graph. |
| enum-iterator2.3.0 | 0BSD | B | direct required component dependency; Windows normal graph. |
| enum-iterator-derive1.5.0 | 0BSD | B | proc macro → enum-iterator → component; Windows build-time execution. |
| option-ext0.2.0 | MPL-2.0 | B | `dirs-sys0.5` → `dirs6` → `shellexpand3.1.2` → `rust-embed-impl8.13` proc macro → `rust-embed` → kit-assets → component; present in Windows graph as part of macro/build tooling. Also dirs-sys0.4 → dirs5 → font-kit in full graph. |

The actual arrayref LICENSE permits source/binary redistribution with retained notices; libbz2-rs-sys LICENSE includes source notice, attribution/alteration and no-endorsement conditions. These are candidates for a separately reviewed policy decision, not automatic acceptance. MPL-2.0 is **not permissive**: its covered-source obligations and larger-work rules require distinct consideration. The cbindgen LICENSE sections3.1–3.4 were read; being build-only does not change the current gate. Exact local license files/hashes are retained in the evidence manifest. No claim that all rejected licenses are GPL/AGPL or equally restrictive is made.

## Advisory findings and unavoidable edges

These are **INFO Unmaintained** advisories, not demonstrated vulnerabilities or exploitability findings. The five official advisories report no patched versions. The local RustSec database revision is `caee11f41fab07d676c0a4655a2a5dd1f94da97c`; advisory markdown files are hashed in the evidence manifest.

| Crate/version | Advisory | Routes and dependency edge |
| --- | --- | --- |
| paste1.0.15 | [RUSTSEC-2024-0436](https://rustsec.org/advisories/RUSTSEC-2024-0436.html) | A/B/C: GPUI's image0.25.10 → exr1.74.2 → pulp0.22.3 → paste proc macro, present in Windows graph. B also component requires paste directly. Mac metal paths add another full-graph use. |
| rustybuzz0.20.1 | [RUSTSEC-2026-0206](https://rustsec.org/advisories/RUSTSEC-2026-0206.html) | A/B/C: required GPUI/pre resvg0.46 **text feature** → usvg0.46 text → rustybuzz, Windows normal graph. B also resvg0.45.1 text. |
| ttf-parser0.25.1 | [RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html) | A/B/C: GPUI/pre direct required dependency; also fontdb/usvg/rustybuzz. Windows normal graph. |
| instant0.1.13 | [RUSTSEC-2024-0384](https://rustsec.org/advisories/RUSTSEC-2024-0384.html) | B: gpui-base and component direct dependencies; Windows graph. |
| rustls-pemfile2.2.0 | [RUSTSEC-2025-0134](https://rustsec.org/advisories/RUSTSEC-2025-0134.html) | B: gpui-pre-reqwest0.12.15 → kit-assets' wasm-target dependency → component. Absent actual Windows tree, present full graph. |

`gpui-kit0.7.3` does **not** force the optional component feature: defaults are component + assets and both can be disabled. It does always depend on gpui-base; component separately requires kit-assets. Therefore route B is not asserted to be the smallest possible alternative. Route C tests the strictly smaller allowed engine/platform family, omitting kit, base, component and their assets altogether. It still has arrayref/cbindgen license failures and all three paste/rustybuzz/ttf-parser advisories.

The exact-tag workspace and pre0.3.9 manifests make resvg0.46 nonoptional with `text`, `system-fonts`, `memmap-fonts`, `raster-images`, and make ttf-parser0.25 a direct nonoptional dependency. Although rustybuzz is optional in **usvg's** manifest, unconditional GPUI resvg/text enables it. Turning off GPUI's default features does not disable that edge. The image feature edge to exr/pulp/paste is likewise present in the audited minimal Windows graph. No component-only failure is used to infer a blocker for the whole family.

## Bounded maintenance evidence, not an approved workaround

The official [resvg changelog](https://github.com/linebender/resvg/blob/main/CHANGELOG.md) records the 0.48.0 text-stack migration from ttf-parser/rustybuzz to skrifa/harfrust. The [v0.48.0 release](https://github.com/linebender/resvg/releases/tag/v0.48.0) exists at release commit `af7573d`; the changelog is the verified migration source (the web fetch for the tag-specific changelog returned a cache miss). This is the first migration version confirmed in that source, not a claim of exhaustive upstream patch discovery.

This does not provide a compatible `cargo update` within either approved route. Their resvg requirement0.46 excludes0.48 under Cargo's 0.x compatibility rules. GPUI also directly calls `ttf_parser::Face::parse(font_data, face_index).glyph_index(ch)` in `font_has_char`, at [tagged `crates/gpui/src/svg_renderer.rs:42–46`](https://github.com/zed-industries/zed/blob/v1.23.3/crates/gpui/src/svg_renderer.rs#L42-L46); pre0.3.9 has the same call at the same local line. Updating resvg alone therefore leaves direct ttf-parser. A maintained text-stack port needs code/API/rendering regression work, a new approved source identity, and separate policy decisions for remaining licenses.

The RustSec pages suggest pastey/with_builtin_macros, harfrust, skrifa, web-time, and rustls-pki-types `PemObject`, respectively. Those are replacement APIs/packages, not patched versions of the resolved crate identities. No compatible patch/fork within the existing exact pins has been validated here. No maintenance fork was implemented, no alternative GPUI version was silently selected, and no advisory ignore was introduced. Root will prepare any concrete Proposed ADR and scope choice after independent evidence review.

## Product preservation and remaining work

`product-input-comparison.json` compares every file in task06's 117-file independent manifest, and separately identifies compilation/product inputs (crates, root Cargo files, CI, justfile, deny and toolchain). It records current hashes, missing/different files, and the original checkpoint hash. Task07 research and checkpoint documents are the only tracked-area writes by this implementer, plus ignored spike workspaces.

Task07 product build `cargo build -p ferrostage-app --features gpui,pdfium`, feature clippy, default `just ci`, core dependency-tree verification, pure shell regressions and all native-window checks are **NOT_RUN for task07**. No product code changed, so task06 tests were not redundantly rerun and are not relabeled as task07 acceptance. The two successful spikes establish native source/link feasibility only. No theme/shell source, CI feature, ADR approval or platform bridge implementation is delivered.

Resume at I-2 only after a route/policy decision is approved and checked. Then implement the scoped shell transaction, safe import, app-level input/focus, settings role authority, single topology subscription and shared theme, and perform the planned build/test/UI checks. Physical unplug/port changes stay NOT_RUN this turn per the user; single/zero-display modes also need real environments. Gate B remains incomplete and cannot be inferred from source research, compilation or synthetic transactions.
