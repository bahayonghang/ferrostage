# Type Safety

Prefer newtypes over bare integers.

```rust
pub struct DocumentGeneration(pub u64);
pub struct PhysicalPage(pub usize);
pub struct LogicalSlide(pub usize);
```

`DocumentGeneration` is ordered. A shadow load must pass a strictly greater value or `apply_at` returns `CoreError::StaleGeneration`.

`NormalizedRect` is page space in the unit square. Use `FULL`, `LEFT_HALF`, `RIGHT_HALF`, `TOP_HALF`, and `BOTTOM_HALF` instead of repeating the numbers. `is_inside_unit_square` rejects non-finite or overflowing crops.

`BlankMode` is an enum (`Black`, `White`). Do not pass a `bool` and a comment.

## Notes boundary

`AudienceProjectionSnapshot` has no notes field. `LoadedDocument::project` accepts `Option<&SlideNotes>` and does not read it, so a test can prove the secret string never appears in the snapshot's `Debug` output.

## Forbidden

- `String` for a page index.
- RGBA color tuples on the audience path. Stroke color is `[u8; 4]` BGRA.
- `unsafe` transmute between `PhysicalPage` and `LogicalSlide`.
