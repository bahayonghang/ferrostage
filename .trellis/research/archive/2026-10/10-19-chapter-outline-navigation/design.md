# Chapter Directory Design

## Direction and Alternatives

**Visual thesis:** FerroStage warm dark utility surfaces, ivory text and restrained clay emphasis; a flat typographic directory supports rapid scanning during questions.

**Content plan:** orient with page/status, retain numeric entry, choose thumbnails or directory, activate one destination.

**Interaction thesis:** no animated opening, keyboard movement or list transitions. Reuse immediate pointer-press feedback and focus restoration; current-section status uses text as well as color.

Production uses native GPUI and the existing theme.rs/root DESIGN.md token system, with no CSS/web runtime. The review artifact uses plain CSS only and is not native acceptance.

| Candidate | Benefit | Cost / decision |
| --- | --- | --- |
| A: tabs in current browser | Preserves panel footprint and full thumbnail width | Recommended; directory and thumbnails are not simultaneous |
| B: directory beside grid | Simultaneous semantic/visual scanning | Reject for MVP: shrinks thumbnails or widens an already large panel |
| C: separate directory popup | Direct directory entry | Reject for MVP: duplicates navigation chrome, focus and lifecycle |

No collapse or search in MVP. The sample has 39 rows; reveal plus a fully expanded list avoids another click before a second-level target.

## Frontend Layout

Retain contextual host, 880px preferred width, 16px padding, 12px panel radius and window containment. Shared title/status, numbering row, numeric editor and helper remain. Add a >=44px `缩略图 / 章节目录` selector. Thumbnail mode retains density, grid and group footer. Directory mode replaces those three areas with:

1. Quiet column labels: `章节标题` and `内容页 · PDF`.
2. One scrollable, fully expanded list, without per-row cards or thick accent rails.
3. Footer: `39 个目录项 · 最多两级` and `点击标题直接跳页`. No redundant confirmation or directory pagination.

Rows have >=44px targets, 12px horizontal padding, natural height for full titles and 24px depth-2 indent. Reserve a nonshrinking, right-aligned page column. Chapter title: resolved interface font, 16px role, semibold; section title: same 16px role, regular; helper/badge: 12px role. Apply existing font scaling and Segoe UI / Microsoft YaHei UI fallback. Do not introduce a display typeface.

Reuse Palette roles, RADIUS, PANEL_RADIUS and CONTROL. Current dark values include canvas #181715, soft #1F1E1B, card #252320, body #E6DFD8, focus #E5A38B (`theme.rs:180`). Current row uses a quiet surface step and explicit `当前章节`; keyboard focus uses the existing 2px ring. Hover/pressed states reuse shared behavior, including immediate 1px pointer-press movement without animation. No blanket clay chapter fills or shadow-card grid. Light mode uses its corresponding palette.

## State and Keyboard Contract

| State | Presentation and behavior |
| --- | --- |
| Default | New document opens thumbnails first; same-document reopen remembers view. Numbering mode is independent. |
| Current | Reveal applicable row when entering directory; focus/editor changes cannot move the current badge. |
| Disabled | Keep title, show localized notes/unavailable/external reason. No commit or invented numbers. |
| Empty | `此 PDF 没有可用的章节目录`; keep numeric controls and tabs. No OCR prompt. |
| Partial | Retain good entries plus `部分目录无法读取`; safety-budget exhaustion says incomplete. |
| Busy/stale | Shared guard rejects activation. Accepted replacement closes old browser and rebuilds new session data. |
| Long/locale | Wrap full titles, align fixed page column, localize interface only. Preserve authored title text. |

Tab/Shift+Tab follow visible controls and directory rows. Within rows, Up/Down traverse, Home/End reach first/last, Enter/Space commit, Escape dismisses. Left/Right and Home/End in the number editor retain editing behavior. Contextual handlers consume activation before global Previous/Next/Start. Disabled targets cannot activate; native accessible labels include their reason. Reuse supported GPUI accessibility APIs and record unsupported behavior explicitly.

## Architecture and Data Flow

PDF bytes -> `extract_structure` -> owned outline in DocumentStructure -> mapped directory in LoadedDocument -> DocumentSession -> cloned BrowserData -> guarded Shell activation -> existing JumpTo(PhysicalPage).

### PDF Boundary

Add a narrow outline module beside existing structure helpers and a default-empty owned outline field on DocumentStructure. Each retained row needs source-order id, optional parent id, depth 1/2, decoded title, optional PhysicalPage and an unavailable-target reason. Keep outline-level partial/diagnostic status separate from existing page/annotation diagnostics; never fake PhysicalPage(0) for document faults.

Use existing lopdf 0.45 and decode_text_string. The sample uses direct /Dest arrays with page-object references and /Fit. Support explicit destinations, local /A GoTo /D, catalog /Dests and /Names/Dests name trees. Name/string keys and indirect values resolve to a local explicit destination. Map page references through actual get_pages() object ids, never labels, displayed digits, parity or object numbers. Only page identity matters; Fit/XYZ coordinates do not alter presentation crop/zoom.

Traverse /First and /Next with visited-object tracking, preserving order. Descend from depth 1 to depth 2 only; do not visit depth-3 subtrees. Malformed parents must not hide valid children. Keep bad-target rows disabled; a missing/undecodable title uses a localized ordinal unavailable-title placeholder so valid children retain parentage. Preserve duplicate targets/titles. Never execute/follow GoToR, GoToE, URI, Launch, JavaScript or action /Next chains.

Use safety ceilings: 4096 visited nodes per outline/name tree, 4096 retained rows, 32 indirect-resolution hops per value. Exceeding a ceiling returns partial metadata without blocking PDF opening or looping. These are fixed parser bounds, not settings.

Do not blindly delegate traversal to lopdf::Document::get_outlines. Inspected 0.45 source accepts GoToR, descends beyond two levels, lacks /Next cycle detection and indexes destination arrays without a length check. Reuse safe object/decode APIs with local bounded traversal. No dependency changes.

### Document Boundary

Build mapped entries after SlideMap exists (`builder.rs:169`). An entry retains source id/depth/title/raw target and adds content_number: Option<usize> plus target status. Content number is `slide_map.steps().position(target) + 1`, matching PageNumberMode::entries. Never use a logical-slide index, page-label parse or divide physical pages by two.

Nonmember targets stay disabled. A bookmark to recognized notes never redirects to an adjacent content page, even when NotesMapping knows its owner. Rebuild mapping on every successful option reconfiguration/reload.

### UI Boundary

Add a document-session enum `Thumbnails | Outline`, independent from PageNumberMode. Browser identity remains per-open and generation-bound. Proposed intents: `BrowserView { id, view }` and `BrowserOutlineJump { id, entry_id }`. Shell looks up the entry in current LoadedDocument, verifies browser identity/generation and SlideMap membership, then uses the existing submit path. Do not accept a raw stale rendered destination.

Switching view renews interaction id, preserves numeric edit/target/error and thumbnail group, and focuses/reveals the appropriate row after layout. It never navigates. Numbering changes preserve physical target/current section and existing editor conversion.

BrowserData::from_shell snapshots rows/view without synchronous Shell reentry during first draw (`presenter/page_browser.rs:7`). Keep current section, focused row and numeric target distinct. New documents reset to thumbnails. Same-document replacement may retain view preference but closes the browser; no old row survives. Successful options changes rebuild directory and dismiss the surface; failed changes preserve the usable old session. Apply existing Home/Stop/Swap dismissal behavior.

Current row: among enabled entries whose target <= current physical page, choose greatest target; tie by deeper depth, then earlier source order. Before the first target mark none. This handles chapter covers, duplicates and out-of-order targets without reordering the list. Mark one row; focus is not current section.

Workbench.grid gates thumbnail requests (`workbench.rs:306`), while contextual browser identity is separate. Keep contextual-open true in Outline, make grid visibility/scheduling false, retire thumbnail frames/targets/exact-key texture work, and let existing scheduling advance epoch/cancel queued work. Returning to thumbnails restores group and <=16 visible slots. Do not treat grid=false as contextual closed. Preserve speaker/next/notes/prefetch/audience requests. Audit GridMove/density so stale hidden controls are inert after switching.

No core type changes, audience fields, settings schema or persisted preference are necessary. Titles remain presenter-only; a committed target uses existing annotation/playback navigation, including concurrent Beamer work.

## Coordination, Rollout and Rollback

The dirty baseline includes defense-experience edits in PDF structure, document builder, actions, page-browser/presenter/workbench and DESIGN.md. Inspect latest diffs before implementation, extend current fields/fixtures and do not revert those edits. One independent task owns this end-to-end directory feature; do not attach it to the existing review tree by assumption.

After approval, update only the page-browser contract in root DESIGN.md before native implementation and update relevant specs at finish. Roll back only task-owned module/fields/intents/view/documentation hunks, retaining old numeric/thumbnail navigation. No version/install/release action is authorized.

## Evidence Limits

The probe verifies PDF structure/targets. HTML layout screenshots verify only the proposed appearance. GPUI behavior, focus/input routing, render scheduling, Presenting activation and audience privacy remain NOT_RUN until implementation/native verification.
