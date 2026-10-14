# Hook Guidelines

There are no React hooks, no `useEffect`, and no data-fetching hooks.

## What to do instead

GPUI will notify a view through its context (`cx.notify()` after a model update). Until that dependency exists, the only update API is `PresentationState::apply`:

```rust
presentation
    .apply(PresentationCommand::ToggleBlank(BlankMode::Black))?;
let snapshot = presentation.audience_snapshot();
```

Subscribe to the snapshot. Do not poll the filesystem from a view. Hot reload is `ferrostage-platform`'s job (300 ms debounce, then `StageShadowDocument`).

## Forbidden

- A custom hook crate, a signals library, or `yew`/`leptos` patterns copied into GPUI code.
- `unsafe` view code to poke a window handle.
- Starting a thread inside a view. The render worker lives in `ferrostage-render` and is singular because PDFium is not parallel.
