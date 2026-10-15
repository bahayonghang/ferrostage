# Database Guidelines

FerroStage does not use a database, an ORM, or database migrations. Settings schema compatibility belongs to `ferrostage-settings`.

## What is stored

| Data | Where | Format |
|------|--------|--------|
| Open PDF | Caller-owned bytes | PDF file on disk, opened read-only. The core does not retain it. FerroStage never writes that path. |
| Speaker notes | `ferrostage-document::SlideNotes` | In memory for the session. Not in the audience snapshot. |
| User preferences | `ferrostage-settings::Settings` | Local TOML. Atomic replacement targets only the configured settings file. No network. |
| Render cache | CPU caches in `ferrostage-render`; future GPU textures in `ferrostage-ui` | Generation-scoped process memory with a global byte budget per tier. Retire the old native document after replacement commits. |

## Settings

The current schema has eight sections and `schema_version = 1`. `Settings` derives `Serialize`; loading goes through `from_toml(text) -> Result<LoadOutcome, SettingsError>` in `parse.rs`, not whole-struct deserialization. `LoadOutcome` carries resolved settings, the original TOML tree, warnings, write policy, and `needs_writeback`. Missing keys keep defaults (L1 256 MiB, L2 512 MiB, L3 64 MiB, debounce 300 ms). Invalid leaves fall back individually; an invalid section type falls back for that section only.

A keybinding action is one `Vec<String>` leaf: any non-string element falls back to that action's defaults and leaves other actions unchanged. An explicit empty array is valid and keeps that action unbound. Empty strings, duplicates, and cross-action conflicts are diagnosed separately; runtime grammar and dispatch belong to the UI.

`from_toml` and `check-config` never write. `SettingsStore` owns the configured path, backup, and explicit save. Save recursively merges known leaves while preserving unknown top-level, section, and monitor-element fields. Current writable versions can fill missing sections at startup. Future versions stay read-only; malformed files must be backed up successfully before defaults can replace them. Failed saves retain the effective session value and an unsaved status. These source contracts do not substitute for task 02's persistence verification.

## Forbidden

- SQLite, Postgres, or any ORM "for later".
- Writing settings or telemetry off the machine.
- Storing speaker notes inside `AudienceProjectionSnapshot`.
- Writing the user's PDF, including truncate-and-write and atomic replace of that path. FerroStage opens the PDF read-only. Session ink stays in memory.
- Using `SHADOW_TMP_SUFFIX` (`.ferrostage.tmp` in `ferrostage-platform`) as a user-PDF write path. Settings save through a unique sibling `<config-file-name>.tmp-<pid>-<seq>`, sync and close it, then atomically replace the configured settings path. The default basename is `settings.toml`; `--config` may select another basename. FerroStage's own artifacts may use their own sibling temporary files. Neither target is the PDF the user opened.
