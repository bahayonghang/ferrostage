# State Management

There is one presenter state and one audience projection.

| State | Owner | Who may read it |
|-------|--------|-----------------|
| `PresentationState` | `ferrostage-core` | Presenter commands via `apply` |
| `AudienceProjectionSnapshot` | Built by `audience_snapshot` | Both windows |
| `SlideNotes` | `ferrostage-document` | Presenter window only |
| `Settings` | `ferrostage-settings` | App startup, then read-only for the session |

`SwapScreens` flips a flag on the state. It does not change `current_page` or `slide_crop` inside the snapshot. Tests in `crates/ferrostage-core/src/state.rs` lock that in.

Blank mode is `Option<BlankMode>` on the snapshot so the audience window does not invent a second blank flag.

## Forbidden

- A global `lazy_static` deck.
- Cloning speaker notes into the snapshot "just for the audience highlighter".
- Two copies of the current page index, one in the view and one in the core. The view sends a command and renders the snapshot it gets back.
