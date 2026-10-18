# Hook Guidelines

There are no React hooks, no `useEffect`, and no data-fetching hooks.

## What to do instead

GPUI will notify a view through its context (`cx.notify()` after a model update). Until that dependency exists, commands use `PresentationState::apply_at` with the app's monotonic clock sample:

```rust
presentation
    .apply_at(PresentationCommand::ToggleBlank(BlankMode::Black), now)?;
let snapshot = presentation.audience_snapshot();
```

Subscribe to the snapshot. Do not poll the filesystem from a view. Hot reload is `ferrostage-platform`'s job (300 ms debounce, validation, then a coordinated descriptor/document/visual commit). The temporary `StageShadowDocument` bridge does not implement that transaction.

## Forbidden

- A custom hook crate, a signals library, or `yew`/`leptos` patterns copied into GPUI code.
- `unsafe` view code to poke a window handle.
- Starting a thread inside a view. The render worker lives in `ferrostage-render` and is singular because PDFium is not parallel.
