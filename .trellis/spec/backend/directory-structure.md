# Directory Structure

## Layout

```text
crates/ferrostage-core/        state machine, no GPUI, no file I/O
crates/ferrostage-document/    notes and media slots
crates/ferrostage-pdf/         lopdf structure; optional PDFium raster adapter
crates/ferrostage-render/      single-worker raster protocol, scheduling and cache budgets
crates/ferrostage-media/       pure controller/protocol, Rust animation and shared BGRA frames
crates/ferrostage-media-helper/ optional native GStreamer executable; never a GUI dependency
crates/ferrostage-platform/    debounce and %%EOF for read-only external PDF replacement
crates/ferrostage-settings/    TOML
crates/ferrostage-ui/          GPUI (see frontend spec)
crates/ferrostage-app/         binary only
docs/                          plan, architecture, ADRs
assets/                        icons, locales, font README (no font binaries yet)
```

Each library crate has `src/lib.rs`. Split a crate into modules when a file passes roughly 400 lines. Do not add a `src/bin/` next to a library unless the binary is a developer tool for that crate.

## Dependency direction

- `ferrostage-core` does not depend on any other FerroStage crate.
- `ferrostage-ui` uses document/media control APIs only on the Presenter side; Audience consumes the pure snapshot.
- Native GStreamer bindings belong only to the optional `ferrostage-media-helper` executable. Keep them out of the GUI dependency graph even under workspace feature unification. Default CI does not require its native SDK; explicit helper checks do.
- `ferrostage-core` must not depend on `ferrostage-ui`, GPUI, `lopdf`, or `notify`.
- `ferrostage-settings` owns atomic replacement of the configured settings file (unique sibling `<config-file-name>.tmp-<pid>-<seq>`, sync, close, then replace). `ferrostage-platform` watches an externally replaced PDF and does not rename the user's PDF.

## Example

Page identity is a newtype, not a bare `usize`, so a logical slide cannot be passed where a physical page is required:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PhysicalPage(pub usize);
```

That definition lives in `crates/ferrostage-core/src/ids.rs`.

## Monitor topology boundary

`core::MonitorId` and `MonitorRole` are pure. Platform owns physical-pixel `MonitorInfo`, identity confidence, resolution and watcher events. App converts settings bindings to `(MonitorId, MonitorRole)`; platform has no production settings dependency. Windows adapter dependencies are target-gated and exact-pinned. `Unavailable` carries diagnostic last-known topology, never usable Audience availability. GPUI mapping belongs to UI/app and must verify the process-dependent scale source.

The pinned winsafe bitflag `has(mask)` tests whether **any** bit overlaps. When native output requires several fields, use `(fields & required) == required` and test each required field missing independently. Missing DEVMODE geometry fields must return `InvalidGeometry`, even when their numeric storage happens to contain plausible values.

## Raster ownership and budgets

`core::BgraFrame::new` consumes a validated, packed BGRA `Vec<u8>` and stores private `Arc<Vec<u8>>`. Construction moves the pixel allocation; cloning the frame shares it. Consumers receive only `&[u8]`. Use `allocation_bytes()` (Vec capacity, including spare capacity) for CPU cache accounting. L2/L3 each share one budget across every generation; eviction releases the cache's reference, while consumer clones keep their own ownership. The UI supplies actual GPU allocation bytes to `L1Budget` and releases every evicted texture.

PDFium instances, probes and documents live on the single render worker. Channels carry owned metadata, immutable frames and full request identity. Candidate preparation returns metadata before consumer-selected prerender requests enter the same priority queue as active work. A synchronous native render is not preemptible; stale generation/epoch results are suppressed before publication. Native runtime configuration is explicit and process-sticky after successful initialization. See ADR-002 for the optional feature and exact runtime pin.

Candidate readiness is scoped to the worker instance that issued it, as well as generation, epoch and batch nonce. A stopped worker's token must not authorize a replacement worker even when its counters repeat. Keep the private shared identity alive in both worker and token and compare allocation identity at commit; do not use a per-worker numeric counter as the sole capability identity.


## Document mapping boundary

`core::SlideMap` owns validated content steps and logical groups only; labels, ordered PDF notes pages, Markdown, diagnostics and media sources belong to document. Descriptors require a valid map and crop; use explicit `identity` only for metadata-only shell work, `LoadedDocument::descriptor` for real PDF imports. ReconfigureDocument retains generation/page count and stage/timer; CommitDocument requires a newer generation. Both clear preview and clamp to content. Failed construction/commands leave the old model/state intact.

The compatibility `PresentationState::set_slide_crop` also returns `Result<(), CoreError>` and uses the descriptor's existing rectangle predicate; invalid geometry returns `InvalidDocumentCrop` before mutation. Document viewing options should use `ReconfigureDocument` to keep map and crop changes atomic.

PDF structure extraction reads bytes into owned metadata, resolves inherited MediaBox/CropBox/Rotate, clips to visible bounds and rotates rectangles before document crop-relative normalization. Non-default UserUnit is rejected until the native/structure geometry contract supports it. Unknown media parameters stay diagnostic/None; unscaled Movie ticks must not be relabeled as seconds. Native PDFium-to-builder integration lives in document's explicit `pdfium` feature; default tests remain native-free.

A malformed or unresolved Filespec `EF` entry must not silently become an external source; report it and retain no source. PDF null is equivalent to an absent entry. Diagnose unsupported Movie parameters in both the annotation activation dictionary and the selected Movie action, since action values override the annotation defaults.
