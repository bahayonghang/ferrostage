# Database Guidelines

FerroStage does not use a database, an ORM, or migrations.

## What is stored

| Data | Where | Format |
|------|--------|--------|
| Open PDF | Caller-owned bytes | PDF file on disk. The core does not retain it. |
| Speaker notes | `ferrostage-document::SlideNotes` | In memory for the session. Not in the audience snapshot. |
| User preferences | `ferrostage-settings::Settings` | Local TOML. No network. |
| Render cache | Future L1/L2/L3 budgets in `ferrostage-render` | Process memory, dropped on generation change. |

## Settings

`Settings` derives `Serialize` and `Deserialize` and uses `#[serde(default)]`. Missing keys keep `Settings::default` (L1 256 MiB, L2 512 MiB, L3 64 MiB, debounce 300 ms).

```rust
pub fn from_toml(text: &str) -> Result<Settings, SettingsError> {
    toml::from_str(text).map_err(|error| SettingsError::Toml(error.to_string()))
}
```

## Forbidden

- SQLite, Postgres, or any ORM "for later".
- Writing settings or telemetry off the machine.
- Storing speaker notes inside `AudienceProjectionSnapshot`.
- Truncating the user's PDF. A write uses `*.ferrostage.tmp` and an atomic rename (`SHADOW_TMP_SUFFIX` in `ferrostage-platform`).
