# Database Guidelines

FerroStage does not use a database, an ORM, or database migrations. Settings schema compatibility belongs to `ferrostage-settings`.

## Recent successful-open metadata

### 1. Scope / Trigger
Apply when recording, loading, trimming or clearing recent file history. A last-opened time is an application event, not the PDF's modification/access time.

### 2. Signatures
Keep `schema_version = 1` and `GeneralSettings::recent_files: Vec<String>`. Optional `recent_opened_at: BTreeMap<String, i64>` stores nonnegative UTC Unix seconds keyed by the same exact path strings. `GeneralSettings::record_recent_file(path: String, opened_at: i64)` updates MRU order/time, truncates and removes orphan metadata.

### 3. Contracts
Missing timestamps default to an empty map. Parse effective recent paths before metadata so dedup/limit pruning applies to both. Invalid table/individual timestamp values fall back locally with warnings; unrelated valid values survive. Replace the entire known timestamp-map leaf on save, rather than recursively preserving removed path keys. Preserve unknown general siblings and other settings metadata.

Record only after the imported document and core state accept the candidate, under the existing writable policy. Failed/cancelled opens and reload do not change history. Save failure retains the successful runtime event with Unsaved/Retry. Clear history clears both containers; limit edits prune both through existing settings validation. Old consumers retain the unchanged string path list and preserve the unknown map, but cannot update its event times.

### 4. Validation / Error Matrix

| Input/action | Result |
| --- | --- |
| Old v1 settings or missing per-path timestamp | Preserve paths; UI shows localized unknown |
| Negative/noninteger timestamp or invalid map type | Diagnose and fall back locally |
| Orphan path metadata or limit reduction | Prune metadata to effective history |
| Clear then save/reload | Both path list and map stay empty |
| Successful open plus failed save | Runtime time retained, dirty/Retry remains truthful |
| Future read-only schema | No history mutation or write |

### 5. Good / Base / Bad Cases
Good: record a new successful open and retain valid timestamps for other retained paths. Base: old paths remain usable with unknown times. Bad: fill migration time/file mtime, update history before PDF acceptance, or resurrect cleared map keys through deep merge.

### 6. Tests Required
Cover omitted/mixed-invalid map, effective-path pruning, record/reopen ordering, whole-leaf round-trip with unknown siblings, clear, failed save/retry and read-only policy. Display conversion must handle nonnegative but unrepresentable i64 values without panicking.

### 7. Wrong vs Correct
Wrong: replace recent string paths with a new incompatible object array or read file metadata during rendering. Correct: keep the v1 path list, store explicit successful-open events in an optional map and capture display-ready metadata before rendering.

## What is stored

| Data | Where | Format |
|------|--------|--------|
| Open PDF | Caller-owned bytes | PDF file on disk, opened read-only. The core does not retain it. FerroStage never writes that path. |
| Speaker notes | `ferrostage-document::SlideNotes` | In memory for the session. Not in the audience snapshot. |
| User preferences | `ferrostage-settings::Settings` | Local TOML. Atomic replacement targets only the configured settings file. No network. |
| Render cache | CPU caches in `ferrostage-render`; future GPU textures in `ferrostage-ui` | Generation-scoped process memory with a global byte budget per tier. Retire the old native document after replacement commits. |

## Settings

The current schema has nine sections and `schema_version = 1`, including task11's compatible `media` section. `Settings` derives `Serialize`; loading goes through `from_toml(text) -> Result<LoadOutcome, SettingsError>` in `parse.rs`, not whole-struct deserialization. `LoadOutcome` carries resolved settings, the original TOML tree, warnings, write policy, and `needs_writeback`. Missing keys keep defaults (L1 256 MiB, L2 512 MiB, L3 64 MiB, debounce 300 ms, media decode height1080). `media.max_decode_height` accepts integers1..=2160 and applies to the next playback instance; decoded/composed output width is at most1920. Invalid leaves fall back individually; an invalid section type falls back for that section only.

A keybinding action is one `Vec<String>` leaf: any non-string element falls back to that action's defaults and leaves other actions unchanged. An explicit empty array is valid and keeps that action unbound. Empty strings, duplicates, and cross-action conflicts are diagnosed separately; runtime grammar and dispatch belong to the UI. Saved files persist whole lists, so a changed default needs a load-time upgrade: `Keybindings::upgrade_legacy_defaults` replaces `next`/`previous` only when they equal the legacy defaults exactly and the added arrow (`Down`/`Up`, for presenter remotes that send arrows) is unclaimed by any other action. Customized lists are never rewritten; no `schema_version` bump.

`from_toml` and `check-config` never write. `SettingsStore` owns the configured path, backup, and explicit save. Save recursively merges known leaves while preserving unknown top-level, section, and monitor-element fields. Current writable versions can fill missing sections at startup. Future versions stay read-only; malformed files must be backed up successfully before defaults can replace them. Failed saves retain the effective session value and an unsaved status. These source contracts do not substitute for task 02's persistence verification.

## Interface font preferences

### 1. Scope / Trigger

Changes to presenter UI typography must extend the existing v1 general section, loader and known-key merge together. These preferences do not edit PDF/notes fonts or audience content.

### 2. Signatures

`GeneralSettings::ui_font_family: String` defaults to `""` (system default). `GeneralSettings::ui_font_size: u32` defaults to 16; `UI_FONT_SIZE_RANGE` is 12..=24 logical pixels. Loading remains `from_toml`; runtime edits remain checked through `settings_panel::edit` and the existing preference application/save flow.

### 3. Contracts

Font family is a trimmed string without control characters. Installation/availability is resolved by the UI, never by the IO parser. A missing installed family retains the requested saved string while the interface uses a fallback. The base size scales interface roles, not document point sizes. These optional leaves do not increment schema_version. Known-key merge must write both leaves while preserving unknown values and monitor-element metadata.

### 4. Validation / Error Matrix

| Condition | Loading / Editing result |
| --- | --- |
| Missing leaves in old v1 file | Default family and 16px |
| Wrong family type/control characters or non-integer/out-of-range size | Loader diagnoses the individual leaf and falls back; checked runtime edit rejects it |
| Valid but unavailable family | Preserve string; UI fallback/notice, no parser warning about installation |
| Future schema | Existing read-only policy, no normalization/writeback |
| Save fails | Effective session typography remains with Unsaved/Retry; durable value requires successful save/readback |

### 5. Good / Base / Bad Cases

Good: select an installed family, save through SettingsStore, and read back both leaves with unknown vendor keys intact. Base: absent keys preserve the shipped look. Bad: derive Serialize fields but omit leaf loading or merge, or overwrite an unavailable requested family with the fallback name.

### 6. Tests Required

Retain `interface_font_leaves_default_validate_and_preserve_future_policy` and `interface_fonts_roundtrip_with_unknown_leaves`, plus effective-value retention on failed save. UI tests separately prove live actual Presenter typography and selection input; storage tests do not prove native font rendering.

### 7. Wrong vs Correct

Wrong: reuse `presenter.notes_font_scale` for UI size or validate fonts by enumerating the OS inside settings loading. Correct: optional general leaves plus Presenter-owned cached availability and role-aware resolution.

## Audience click preference

### 1. Scope / Trigger

The playback-window input preference is an optional v1 display leaf; extend the typed schema, leaf loader and known-key merge together.

### 2. Signatures

`DisplaySettings::audience_click_to_advance: bool`, TOML `display.audience_click_to_advance`, defaults to true. Keep `SCHEMA_VERSION = 1`.

### 3. Contracts

Missing values use true. Invalid types warn at the full leaf path and fall back independently. Explicit false survives save and reload. Preserve unknown display keys and monitor-element metadata. A missing optional leaf need not trigger immediate startup writeback; explicit saves merge it normally. No alternate settings file or PDF write is allowed.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| Existing v1 settings omit the leaf | Effective true |
| Non-boolean leaf | Warning and default true |
| Explicit false, then save/reload | Effective false and unknown values retained |
| Save fails | Effective value retained with existing dirty/error and Retry behavior |
| Future schema | Parsed effective value; file stays read-only and bytes unchanged |

### 5. Good / Base / Bad Cases

Good: save false beside unknown monitor metadata and recover both on reload. Base: older configurations require no manual migration. Bad: serialize the field but omit parser/merge support.

### 6. Tests Required

Cover defaults, explicit true/false, invalid type, roundtrip, unknown-data retention, failed-save/retry and future-version refusal. UI separately verifies immediate behavior without a display placement transaction.

### 7. Wrong vs Correct

Wrong: bump the schema or rewrite user settings for a missing optional bool. Correct: compatible leaf default plus existing checked save pipeline.

## Forbidden

- SQLite, Postgres, or any ORM "for later".
- Writing settings or telemetry off the machine.
- Storing speaker notes inside `AudienceProjectionSnapshot`.
- Writing the user's PDF, including truncate-and-write and atomic replace of that path. FerroStage opens the PDF read-only. Session ink stays in memory.
- Creating a shadow temporary file beside the user's PDF or replacing that PDF. The historical `SHADOW_TMP_SUFFIX` constant has been removed; hot reload holds its read-only shadow bytes in memory. Settings save through a unique sibling `<config-file-name>.tmp-<pid>-<seq>`, sync and close it, then atomically replace the configured settings path. The default basename is `settings.toml`; `--config` may select another basename. FerroStage's own artifacts may use their own sibling temporary files. Neither target is the PDF the user opened.
