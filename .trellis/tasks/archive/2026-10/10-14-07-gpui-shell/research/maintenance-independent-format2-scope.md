# Remaining format-2 rendering compatibility boundary

Status: **proposed scope only; no harfrust/read-fonts changes applied**. The GPUI helper repair does not resolve the consumer failure.

`maintenance-independent-render-probe.log` records a real-outline fallback comparison: the original resvg/usvg0.46.0 stack paints268 pixels through one fallback; resvg/usvg0.48.1 paints304 pixels through two fallbacks for a format-2-only cmap. Exact pixels differ and the regression test exits101. The format13 control paints268 pixels in both stacks with identical full RGBA buffers. Both fixtures derive the same IBM Plex Sans `A` outline in memory, map it to U+2705, and first request Lilex, which lacks that character. No derivative font is saved. A previous nonempty-pixel assertion was insufficient and is retained separately, not treated as rendering equivalence.

## Actual call paths

- `harfrust-0.12.0/src/hb/face.rs`: `ShaperData::new` constructs `TableRanges::new`; the normal `ShaperBuilder` builds `Charmap::new` from that cached selection.
- `src/hb/tables.rs`: `find_best_cmap_subtable` preserves the established symbol/Unicode/Mac encoding priority. Its `find_cmap_subtable` admits formats0/4/6/10/12/13 and excludes2.
- `src/hb/charmap.rs`: `Charmap::map` also has no format2 arm. Its alternate `Charmap::from_tables` uses `read_fonts::Cmap::best_subtable`, which independently excludes2. `src/hb/font_funcs.rs` uses this alternate path for `FontKind::FontInstance`; fixing only cached selection would miss it.
- `read-fonts-0.41.0` already parses `CmapSubtable::Format2`, with `Cmap2::sub_header_keys`, `sub_header_keys_byte_range`, `offset_data`, `length`, and a checked `FontData::read_ref_at::<SubHeader>` / `read_at::<u16>`. `SubHeader` exposes first-code/count/delta/range-offset. Its generic mapping method does not implement2.

## Smallest coherent candidate repair to evaluate

One additional locally copied **MIT harfrust0.12.0** package, two source files, approximately45–60 production lines plus focused tests:

1. Admit `Format2` in the existing selection helper in `hb/tables.rs`; expose that helper only to sibling modules (`pub(super)`). Keep encoding priority unchanged.
2. Route `Charmap::from_tables` through that same internal selection helper so cached `FontRef` and alternate `FontInstance` paths agree. This avoids modifying the shared/public `read-fonts::Cmap::best_subtable` behavior.
3. Add one format2 arm and a private adapter in `hb/charmap.rs`, using the already checked raw data APIs. Keep legacy glyph/delta results required by the old stack. No unsafe, font-wide parser, outline/GSUB/GPOS/normalization change or read-fonts fork is indicated by this reproducer.

This is an assessment from the actual sources, **not proof that the patch passes**. Adding this package expands the maintenance obligation and candidate provenance/lock/source-policy records; approval is required before applying it. The existing GPUI helper has its own coverage semantics, so any shared implementation or public API refactor is outside this minimal proposal.

Verification would retain the strict red test, require identical old/new pixels for both format2 and13, add direct shaping-glyph comparisons and checked offset/delta tests on both constructors, run the real GPUI SVG regressions, direct harfrust lint/typecheck and relevant unit tests, and refresh source/archive/lock hashes plus the unchanged full deny audit. It would not establish all-format, all-font or native platform equivalence.

The adapter calculation follows the [OpenType format2 specification](https://learn.microsoft.com/en-us/typography/opentype/spec/cmap#format-2-high-byte-mapping-through-table). GPUI's multi-record presence probe remains its legacy behavior; it is not a proposed renderer selection strategy. Source hashes for these exact examined files are retained in the independent review manifest.
