# Directory Structure

`crates/ferrostage-ui` is the view layer. The skeleton is one `src/lib.rs` that exports `WindowRole` and the GPUI pin. When windows land, split them like this:

```text
crates/ferrostage-ui/src/
  lib.rs
  presenter.rs    notes, next slide, laser, pen
  audience.rs     snapshot only
  theme.rs        colors and type size
```

Strings that the user sees go in `assets/locales/en-US/` and `assets/locales/zh-CN/`. Do not hard-code a second language inside a view.

The binary stays in `crates/ferrostage-app`. It installs the tracing subscriber and then calls into `ferrostage-ui`. Views do not parse CLI arguments.

## Example

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowRole {
    Presenter,
    Audience,
}
```

`WindowRole::Audience` must not gain a method that returns `SlideNotes`.
