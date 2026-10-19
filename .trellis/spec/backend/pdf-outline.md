# PDF Chapter Directory

## 1. Scope / Trigger

Read presenter-only chapter navigation metadata while extracting PDF structure. Outline problems must not turn a readable PDF into a load failure. This uses the existing lopdf dependency; no OCR, PDF edits or action execution.

## 2. Signatures

`DocumentStructure.outline: PdfOutline` contains `entries: Vec<PdfOutlineEntry>` and `partial: bool`. Entries carry `id`, `parent`, structural `depth`, optional decoded `title`, optional `PhysicalPage`, and optional `PdfOutlineIssue::{Unavailable,External}`. The document layer exposes mapped `OutlineEntry`, `OutlineIssue::{Notes,Unavailable,External}`, `LoadedDocument.outline`, `outline_partial` and `current_outline_entry(PhysicalPage) -> Option<usize>` returning an entry id.

## 3. Contracts

Retain structural PDF outline depths 1 and 2, in source order, including unnumbered entries and duplicate titles/targets. Read direct destinations, local GoTo actions and catalog/name-tree destinations. Resolve actual page-object identity through `get_pages()`. Never infer physical identity from labels, object numbers or page parity. Ignore destination fit coordinates for presentation crop/zoom.

Traversal tracks visited identities and uses 4096-node/row limits and 32-hop indirect resolution. Never traverse depth-3 subtrees, external actions or action chains. Missing titles retain an ordinal localized placeholder at the UI boundary; malformed parents do not hide readable children. Preserve unavailable-target reasons and partial status without failing page loading.

Map content numbers through actual ordered SlideMap steps after building the map. A recognized note target remains disabled; do not redirect it to the owner slide. Reconfiguration/reload rebuilds mapping. Current row is the valid target at or before committed page with greatest page, then deepest depth, then earliest source id. Focus and numeric drafts are irrelevant.

## 4. Validation / Error Matrix

| Input | Required result |
| --- | --- |
| No outline | Empty directory, ordinary navigation available |
| Malformed destination/title | Disabled target or localized title fallback; valid children retained |
| Note destination | Original physical target preserved, no content number, Notes reason |
| GoToR/URI/Launch/JavaScript | Disabled External reason, never executed |
| Cycle / budget exhaustion | Stop bounded traversal; partial status, readable pages remain usable |
| Irregular notes / overlays | Numbers index actual content steps, no divide-by-two or logical-slide shortcut |

## 5. Good / Base / Bad Cases

Good: a second-level section targets physical 44 and maps to content 23 in an alternating-notes fixture. Base: a plain PDF without bookmarks keeps numeric/thumbnail navigation. Bad: assume every note follows exactly one slide, renumber missing sections or use lopdf's unrestricted outline traversal as proof of these bounds.

## 6. Tests Required

Retain synthetic parser tests for Unicode, direct/named/local actions, indirect chains, malformed/short arrays, external actions, duplicate targets, invalid parent/valid child, depth exclusion, cycles and budget ceilings. Document tests use irregular notes and overlay steps and cover current-row tie rules plus successful reconfiguration/reload. Private PDFs are read-only optional research inputs, never checked-in fixtures or required CI paths.

## 7. Wrong vs Correct

Wrong: follow external bookmarks, silently redirect a notes target or send titles to the audience. Correct: presenter-owned typed metadata, explicit disabled reasons and guarded entry-id activation through the established content-only JumpTo path.
