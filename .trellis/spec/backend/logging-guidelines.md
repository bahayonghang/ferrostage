# Logging Guidelines

Use `tracing`. Do not use `println!` in libraries. The binary may print the resolved settings and the skeleton banner; diagnostics go through the subscriber installed in `ferrostage-app`.

## Setup

```rust
tracing_subscriber::fmt()
    .with_env_filter(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
    )
    .init();
```

Override with `RUST_LOG`. Default level is `info`.

## What to log

| Level | Use |
|-------|-----|
| `error` | A command failed and the user needs the reason. |
| `info` | A document descriptor was accepted: generation and page count. |
| `debug` | Command name (`PresentationCommand::name`), not the payload. |
| `trace` | Per-page cache hits, once the worker exists. |

The core commit path logs metadata, never bytes:

```rust
tracing::info!(
    generation = descriptor.generation.0,
    pages = descriptor.page_count,
    "committed document descriptor"
);
```

## Forbidden

- `Debug` of a `Vec<u8>` PDF, video frame, or font file.
- Logging speaker notes.
- A second logging facade (`log::info` in new code). `tracing` is the API. The subscriber may bridge `log` later.
- Logging on every pointer move at `info`. Laser updates are `debug` at most.
