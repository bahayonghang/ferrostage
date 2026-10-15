# GPUI candidate: exact license review

已获明确批准：用户回复“批准采纳及这三个精确例外（推荐）”。允许把下述最终维护候选接入任务07，并只为 `arrayref@0.3.9`、`libbz2-rs-sys@0.2.5`、`cbindgen@0.28.0` 增加列出的精确许可证例外。这是在先前维护评估授权之外、独立审查之后的具体采纳授权。产品接入正在实施；尚未提交或发布。

独立审查已完成，两处兼容性缺陷已修复；Windows构建、GPUI/消费端lint及类型检查通过。完整原政策审计仍exit4，只有这三个许可证失败。直接HarfRust严格lint仍有6项上游既存错误，与未修改0.12版本的消息和位置完全一致；没有新增诊断，但不能称所有lint通过。产品接入后仍须通过项目自身质量门禁，不新增lint豁免。

## Candidate adoption scope

The authoritative [independent review](../../10-14-07-gpui-shell/research/maintenance-independent-report.md), [combined final package diff](../../10-14-07-gpui-shell/research/maintenance-independent-final-source.patch), and [single reconstruction entry](../../10-14-07-gpui-shell/research/maintenance-independent-prepare-final.py) define this candidate. **The original maintenance patch/preparer alone is a historical defective version and must not be adopted.**

The source remains Zed tag `v1.23.3` at `96a1b1b5389efff473fc2306f7f8c690f2e1a417`. The five locally maintained packages and exact changes are:

- GPUI0.2.2: resvg/usvg0.48.1, skrifa0.44.0, preserved SVGZ/codecs, a bit-identical Rust1.99 constant-lint fix, and the repaired Unicode cmap presence helper with checked format2 mapping.
- pulp0.22.3 and metal0.33.0: manifest-only replacement of the macro dependency with the maintained pastey0.2.3 package.
- cosmic-text0.19.0: manifest-only fontdb0.24.0 selection for the full WGPU/Web graph.
- MIT harfrust0.12.0: two source files restore format2 selection/mapping through both existing constructor paths, preserving encoding priority and using checked read-fonts APIs. No public API, GSUB/GPOS, outline or normalization changes.

The final526-file source snapshot has SHA256 `cc9a8ded6bfeeed1c0286ba5bf2e3ecf750b6ce34987f11d4b6e23d85ec8a960`; the combined package diff has SHA256 `003fd8d088ce9d5466c059608cf3990418cb2b4fdb3094798b1407b0f70b0a83`. The [final review manifest](../../10-14-07-gpui-shell/research/maintenance-independent-manifest.json) has SHA256 `4cc87ec3a4f2fe7bbad8757eefd78869be8ae62d7f73e3a6e027e3201ea23bba`. Fresh reconstruction matched all526 paths; Root independently read back all753 evidence/source records with zero differences ([receipt](maintenance-final-readback.json)). Final locks and notices are retained in the review collection. The three updated locks only change harfrust0.12 from registry to the local source; no package/version addition or removal occurs.

GPUI16 tests, root7 tests, HarfRust205 tests and the separate4 diagnostic probes passed. The probes overlap some library coverage and are not added to those counts. The strict old/new SVG format2 and13 controls each produced268 pixels, one fallback and identical entire RGBA buffers. Two previous red regressions and the insufficient nonempty-pixel test remain preserved in the report. The two new adapters require future maintenance when their dependencies change.

Adoption would authorize incorporating these reviewed runtime changes within task07, retaining exact provenance/notices and running the product's required checks. Evaluation-only workspace/test scaffolding is documented separately and is not itself a delivered UI. No publishing, pushing, committing, archiving, broader dependency upgrade or advisory waiver is included. Apple native compilation/runtime and Linux/Web native runs remain unverified; the Windows WGPU consumer check only demonstrates host compilation of that API. The original font notices remain required; the generated RFN-bearing TTC is confined to ignored local validation cache.

## Observed package scope

The unchanged full audit was independently rerun after all repairs: [maintenance-independent-final-deny.log](../../10-14-07-gpui-shell/research/maintenance-independent-final-deny.log). Exact license files and hashes were retained in [the original evidence manifest](../../10-14-07-gpui-shell/research/evidence-manifest.json). The final candidate graph rejects exactly these three package/license pairs. Advisories, bans and sources pass; license policy remains FAIL until separately approved and changed, then rechecked in the actual product graph.

| Package | License | Observed use | Proposed handling if separately approved |
| --- | --- | --- | --- |
| `arrayref@0.3.9` | BSD-2-Clause | Windows normal graph through tiny-skia/resvg; also full graph | Exact package/version exception; retain copyright, terms and disclaimer in source and binary distribution materials. |
| `libbz2-rs-sys@0.2.5` | bzip2-1.0.6 | Windows normal graph through bzip2/compression-codecs/async-compression/http_client | Exact package/version exception; retain source notices, identify original authors and mark any alterations; no author endorsement claim. |
| `cbindgen@0.28.0` | MPL-2.0 | Apple GPUI build dependency; absent from the observed Windows build tree | Exact package/version exception for the audited graph; preserve covered-source license and provenance. Any distribution containing the tool or its covered source must include the corresponding source access and notices. Apple native execution remains unverified. |

BSD's source and binary notice requirements are in the [SPDX license text](https://spdx.org/licenses/BSD-2-Clause.html). The bzip2 conditions distinguish required source/origin/alteration/endorsement terms from an appreciated but optional product acknowledgment; see its [SPDX text](https://spdx.org/licenses/bzip2-1.0.6.html).

MPL-2.0 is file-level copyleft, not a permissive license. Mozilla distinguishes private use from external distribution and covered files from larger works in its [FAQ](https://www.mozilla.org/en-US/MPL/2.0/FAQ/); the distribution/source/notice obligations are in [sections 3.1–3.4 of the license](https://www.mozilla.org/en-US/MPL/2.0/). A build-only dependency is still subject to this repository's policy. No claim about a future distribution's compliance follows merely from its target or from compilation success.

## Approved exact policy delta — applied, initial product audit passed

The user explicitly approved this field within the existing `[licenses]` table; apply only these entries and audit the actual product graph:

```toml
exceptions = [
    { crate = "arrayref@0.3.9", allow = ["BSD-2-Clause"] },
    { crate = "libbz2-rs-sys@0.2.5", allow = ["bzip2-1.0.6"] },
    { crate = "cbindgen@0.28.0", allow = ["MPL-2.0"] },
]
```

The existing global allowlist, build-dependency coverage, full target graph, private-crate checks, advisory checks and GPL/AGPL exclusion remain unchanged. Upgrades outside these exact versions require a fresh decision. cargo-deny supports per-package exceptions and includes build dependencies by default; see its [license configuration](https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html) and [exact PackageSpec syntax](https://embarkstudios.github.io/cargo-deny/checks/cfg.html).

The adoption review must also identify the exact GPUI source revision, reviewed local patch/lock hashes, maintained dependency versions and retained notices. The fontdb0.24 upstream metadata parser still contains a maintained subset derived from ttf-parser; removal of a RustSec package node must not be described as removal of all old parser source. The repaired candidate is now reviewable; accepting it does not convert the recorded upstream lint or native limits into PASS. The explicit user reply now authorizes this product linkage and these exact exceptions. It does not authorize fork publication, commits/archive, additional policy changes or Gate B acceptance.

The actual product I-2 all-features/locked/offline audit and GPUI/PDFium build now return0. Root verified only these exact three entries changed the parsed policy ([application receipt](gpui-license-application-check.json)); all other values match the previous policy. Shell implementation and final product checks continue, so this does not certify Gate B or the final product state.
