# Settings design and typography contract

## Boundaries and evidence

Read parent `design.md` sections 1–3/5–7 and research F4–F7/F10. `settings_view.rs:357` stacks all labels/controls; `:78` and `:86` produce full-width Clear and raw paths. `theme.rs:16` hardcodes UI family; `settings_panel.rs:293` routes edits through canonical parsing. The font API exists in the pinned vendor at `text_system.rs:284`; no version/dependency research is needed before using that API.

## Composition

Settings header: sans title, concise category context, Close aligned as a utility. Category rail: retain existing seven categories and IDs; quiet text, selected soft surface plus weight/indicator, shared focus/activation. Main column: max 960 logical px, natural height, one content scroll; footer status stays associated with that column. The displays child removes its nested host after this foundation lands.

Use section headings and hairlines to group related fields. Ordinary rows have label and optional short help on the left, compact value control on the right. Keep 44px interactive target and content-driven row height; compact does not mean small text/hit areas. Language/theme use one segmented control each. Toggles state their value and can expose native switch semantics supported by GPUI. Numeric steppers join minus/value/plus into one field group, retaining direct editing and boundary disabling. Keybinding recording/conflict controls, pointer preview/color and reset confirmations keep their existing behavior and can use a multi-line row when content needs it. Do not force all seven categories into a one-line template.

General sections: Appearance (language with restart note, theme, family selector, base size with mixed-script preview), Startup (reopen note), Recent files (count, limit and content-sized Clear, followed by history). History rows are informational as today, not new file-open actions. Reuse home basename/parent formatting and path discovery where useful without inheriting its click action or top-three cap; settings still represents the configured history. Empty state says no recent files and disables Clear. Same basenames keep distinguishable parent paths, full path available on hover/focus/accessibility text.

## Data flow

Add `general.ui_font_family = ""` and `general.ui_font_size = 16` to schema/defaults, parser and known-key merge. Family is either default sentinel or a stored family name; reject wrong TOML types and control characters, trim outer whitespace. Do not verify installation in the IO crate. Size is integer 12..=24, single source for UI and parsing limits. Optional leaves retain schema version 1 and existing unknown-key/read-only/save contracts. Update config examples and documentation of these leaves.

Add dynamic family-selection handling to the preferences model, not a fabricated static choice list. Session-cache available names, offer filtering in a bounded popover/list with keyboard selection and Escape returning to the invoker. Choose commits through the same checked preference-edit/apply/save flow; filter text alone does not save. Selection missing from available list shows its saved name and effective System default fallback. Default action stores the empty sentinel. No modal font wizard or font installation feature.

Resolve interface typography from effective general settings in the existing theme layer, then hand the resolved values through Presenter data. Scale existing UI roles by base/16; timer keeps its mono face, home wordmark its Georgia face. Replace targeted hardcoded interface font/size overrides, including settings title, modal/error/tooltip and display consumers. The Audience remains snapshot-only. Do not reinterpret presenter.notes_font_scale. A fallback warning reflects effective resolution and never silently rewrites the requested saved value.

Save feedback remains Saved/Unsaved/Failed plus Retry. Pending display assignment is a different state owned by the displays child. Do not relocate errors out of reach or advertise that all settings are immediately effective when language/startup options require restart.

## Ownership and rollback

Owned targets: `DESIGN.md`, `theme.rs`, `settings_panel.rs`, `presenter/settings_view.rs`, necessary Presenter/home/tooltip font call sites, `shell/preferences.rs`, preference tests, settings schema/parse/merge/tests, locales and settings docs/examples. Necessary supporting files may be added only inside these existing modules. No platform/core/document/raster/vendor modifications. Source paths are investigation targets, not manifest entries.

Update root DESIGN first, retaining its provenance. Remove serif settings-heading and per-field-card assumptions; document optional font keys and role exceptions. Narrow the existing Overview phrase “No new settings keys” to its original theme-resolution scope so it does not contradict the two explicitly approved font leaves. Roll back font schema/load/merge/resolver/UI/docs as one unit; older binaries preserve new leaves as unknown data. Shared layout and typography must not accidentally change the home brand or audience projection.
