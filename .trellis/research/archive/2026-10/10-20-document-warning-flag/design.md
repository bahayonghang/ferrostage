# Document-warning flag

## Boundary

Presenter chrome only. `ferrostage-core` and `ferrostage-document` stay unchanged. The audience view keeps reading `AudienceProjectionSnapshot` and gains no warning string.

Dismissal is session UI state on `Shell`, next to `error`. It is not a setting, not a `PresentationCommand`, and not part of the audience snapshot.

## Data flow

`PresenterData` splits the current mixed `warnings` vector:

- `shell_warnings`: clone of `shell.warnings`, always rendered as inline lines.
- `document_warnings`: `layout_navigation::document_warnings(shell)`, minus strings dismissed for the current generation.

`Shell` stores:

- `dismissed_document_warnings: HashSet<String>`
- `dismissed_warning_generation: DocumentGeneration`

`Intent::DismissDocumentWarning(String)` records the string only when it is one of the current document-warning strings. If the stored generation differs, replace the set with this string and store the current generation. A mismatched generation does not filter. Layout reconfigure keeps the generation, so the set still applies. Import and hot reload change the generation, so the old set stops filtering without a separate clear.

Keys are the localized strings `document_warnings` already dedups. Page-specific warnings stay distinct because the page number is in the string.

## Rendering

`presenter-content` is `relative`. After the workbench child, when `stage != Idle` and `document_warnings` is non-empty, paint one absolute column at left 12px, bottom 12px, `max_w(420px)`, gap 8px.

Each flag id is `document-warning-{index}` in visible order. The close id is `dismiss-document-warning-{index}`. Ids are indexes, not translated labels.

The close control calls `activation_target` so focus, Tab suppression while a menu or browser is open, deferred dispatch, and accessibility click match the other presenter buttons. Its face is not `ButtonKind::Utility`: that face would nest a second bordered chip inside the card. The face is a 44×44 quiet `×`.

Inline shell warnings gain `shell-warning-{index}` so tests can tell them from flags. Their look stays the current bare `p.warning` line.

Add `dismiss-warning` beside `dismiss-error` in `en-US` and `zh-CN` (`Dismiss notice` / `关闭这条说明`). The warning sentences themselves do not change.

## Compatibility

No settings schema change. No migration. A build that drops this UI forgets dismissal because it lived only in memory.

Many structure diagnostics can stack until `presenter-content` clips them. That is the bound. There is no "and N more" summary.

## Trade-off

String keys follow the existing deduped copy. A locale change at runtime would show the same condition again under the new sentence. The product already applies language after restart, so this is acceptable. A typed `DocumentWarning` key would survive a locale switch that does not happen in-session, and would need a second identity for structure diagnostics that are not `DocumentWarning`.

## Rollback

Remove the flag child, the intent, and the two shell fields, and paint `document_warnings` with the old flex line. No stored data to revert.
