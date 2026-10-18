# Settings layout, UI fonts and recent files

## Goal

Make all seven settings categories easy to scan, provide persistent live UI font family/size controls, and turn raw recent paths into readable history rows. First child of `10-18-workbench-settings-design`; parent R4–R6/R9 and AC4–AC7/AC11–AC12 apply.

## Requirements

- Extend the existing warm utility design and one native token layer. Quiet category rail; sans headings; grouped rows rather than one oversized panel per field.
- General contains Appearance, Startup and Recent files. Font setting means UI family plus size, explicitly confirmed by the user. Existing PDF/raster notes and audience pixels do not change.
- Prefer System default plus installed font families, mixed-script preview, size/default restoration, immediate update and normal autosave feedback.
- Recent history preserves current ordering, limit, clear semantics and home integration. Show filename plus parent context; do not add history management features.
- Keep all existing category controls, editing rules, read-only/future-config handling and save failure/Retry.

## Acceptance Criteria

| ID | Observable result / requirement mapping |
| --- | --- |
| S1 | General, Displays, Presenter, Timer, Pointer, Keys and Advanced use the shared compact nav/heading/row vocabulary. Controls align as meaningful rows; local explanation replaces detached boilerplate. Parent R4/AC4. |
| S2 | User can select System default or an installed UI family and size, preview Chinese/Latin text, and restore default family/size. Settings remains open and usable; interface consumers update live. Parent R5/AC5. |
| S3 | Restart preserves selection. A missing family displays fallback notice while retaining the requested value; body/heading roles scale, brand and monospaced roles stay distinct. PDF, raster notes, audience content and OS titlebar are unaffected. Parent R5/R9/AC5–AC6. |
| S4 | Legacy v1 files load defaults; invalid leaves and invalid edits follow existing respective fallback/reject behavior; unknown keys survive save. Future config is read-only. Failed saves retain effective values and show unsaved/Retry. Parent R5/R9/AC6/AC11. |
| S5 | 0/1/multiple recents have real filename/parent and count/limit, distinguish same-named files, expose full path, and preserve order. Clear empties settings and home history; no full-width Clear or fake metadata/actions. Parent R6/AC7. |
| S6 | Relevant schema/UI regressions and normal-window native General/settings checks have recorded evidence. Existing fields, key recording, reset confirmations and preference behavior remain intact; display allocation transaction remains separate. Parent R9/AC12. |

## Notes

- User approved the parent plan on 2026-10-18; this child is activated for implementation. Complete before workbench and display children touch shared styling.
- Out of scope: document fonts, font downloading, OS titlebar fonts, new history search/open/pin/remove actions, dependency additions, changes to settings atomicity or exhaustive dimension/DPI tests.
- Shared display body/scroll cleanup and role feedback belong to the displays child; this child establishes the host layout and truthful ordinary preference status.
