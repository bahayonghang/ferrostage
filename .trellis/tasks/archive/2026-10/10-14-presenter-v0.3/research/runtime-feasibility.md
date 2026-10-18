# Research: Runtime feasibility and native integration gates

- Query: Verify the pinned library and Windows feasibility assumptions for children 03, 05, 07, 10 and 11 before implementation.
- Scope: mixed; read-only source inspection and authoritative library/OS documentation. Planning only; no dependency, code or policy changes.
- Date: 2026-10-14

## Findings

### Files found and current implementation evidence

| File | Observed responsibility |
| --- | --- |
| `Cargo.toml:35` and `Cargo.toml:52` | Workspace dependency declarations and `unsafe_code = "forbid"`; native PDF/UI versions appear in comments, not dependency entries. |
| `crates/ferrostage-pdf/Cargo.toml:10` and `src/lib.rs:31` | Links `lopdf` and `thiserror`; `parse_structure()` calls `lopdf::Document::load_mem`. No PDFium runtime. |
| `crates/ferrostage-platform/Cargo.toml:10` and `src/lib.rs:31` | Links `notify` 8.2.0 and names its watcher type; no watcher loop, monitor enumeration or atomic writer. |
| `crates/ferrostage-render/src/lib.rs:10` and `:25` | Cache budget constants and generation equality; no queue, worker, CPU cache or GPU texture ownership. |
| `crates/ferrostage-ui/Cargo.toml:10` and `src/lib.rs:13` | Only `ferrostage-core` is linked; Zed `v1.23.3` and GPUI Kit 0.7.3 are tested constants. |
| `crates/ferrostage-media/src/lib.rs:13` and `:22` | Shared immutable BGRA bytes and checked geometry; no backend, audio, playback or lifecycle metadata. |
| `Justfile:7` and `.github/workflows/ci.yml:10` | Existing checks are fmt, workspace clippy/tests and cargo-deny; hosted CI currently runs only Ubuntu. |
| `deny.toml:11` and `docs/adr/ADR-005-license-gate.md:11` | Short permissive Cargo license allowlist; no blanket LGPL allowance. |
| `docs/adr/ADR-001-gpui-from-zed-tag.md:13`, `ADR-002-pdf-backends.md:15`, `ADR-004-lock-free-reload.md:9` | Recorded UI/PDF pins, single PDF worker, debounce/generation and sibling-temp policies. These are decisions, not working integrations. |
| `.trellis/tasks/10-14-{03-monitor-topology,05-pdf-raster,07-gpui-shell,10-hot-reload,11-media-playback}/prd.md` | Initial requirements inspected to identify API, ownership, CI and native-distribution conflicts. |

**Verified:** `lopdf` and `notify` are actual manifest dependencies. PDFium, GPUI, hayro, image animation decoding and video engines are not linked. A version constant or a comment does not pin a resolved Cargo package or prove it builds. All inspected workspace crates inherit workspace lints. Preserve the existing version baseline; resolve exact dependencies only in the corresponding implementation task.

### 03: Unsafe policy and monitor identity

**Verified facts**

- Rust `forbid` cannot be lowered by a nested `allow`. The initial 03 proposal to add `#![allow(unsafe_code)]` beneath inherited `forbid` is not compilable. Cargo workspace lints govern members that inherit them; unsafe inside a separately compiled third-party dependency is not unsafe source authored in FerroStage. [Rust lint levels](https://doc.rust-lang.org/rustc/lints/levels.html#forbid).
- Windows DisplayConfig exposes adapter/target information. `DISPLAYCONFIG_TARGET_DEVICE_NAME` includes EDID manufacturer/product fields only when validity flags permit, connector instance, friendly name and `monitorDevicePath`. A device-interface path is not itself a documented promise that a physical screen retains its ID after changing ports; manufacturer/product do not identify two identical screens uniquely. [Target device fields](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-displayconfig_target_device_name), [QueryDisplayConfig](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-querydisplayconfig), [device information](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-displayconfiggetdeviceinfo).
- A safe enumeration candidate exists: `display-info` documentation exposes safe enumeration and names, raw handle, desktop position, size, scale and primary status. Its published `DisplayInfo` fields do not expose EDID serial or DisplayConfig target device path. It is evidence for geometry enumeration, not a verified solution for every identity requirement. [DisplayInfo API](https://docs.rs/display-info/0.5.9/display_info/struct.DisplayInfo.html).

**Planned design / unresolved gate**

1. Preserve workspace `forbid`; first evaluate a safe wrapper that covers the necessary DisplayConfig identity and event APIs, including its exact version/license and error behavior. Do not add a capture subsystem merely to obtain monitor metadata.
2. If no sufficient wrapper is found, stop at a recorded decision gate. Any narrowly isolated project FFI would require an explicit manifest/spec/ADR policy change; a module attribute cannot implement it. This research does not authorize that change.
3. Keep stable identity, current connection identity, geometry and confidence separate. Geometry/name fallbacks and missing serials can collide. Duplicate or ambiguous matching must disable audience placement pending explicit rebinding; never silently select the first candidate.
4. Verify same-model screens, absent/invalid EDID, docking, port changes, rotation, mixed DPI and mirrored bounds. Claimed port-independent identity remains an unverified hardware hypothesis.

### 05 and 10: PDFium ownership, bytes, PageLabel and crops

**Verified facts**

- `pdfium-render` 0.9.4 does not include or automatically build PDFium. Its default features are `pdfium_latest`, `image_latest`, `thread_safe`; the supported latest API maps to `pdfium_7881`. `thread_safe` serializes library calls with a mutex and provides no parallel-render speedup. The initial claim that enabling its default dependency automatically downloads PDFium is incorrect. [0.9.4 package documentation](https://docs.rs/pdfium-render/0.9.4/pdfium_render/).
- Safe `load_pdf_from_byte_vec(Vec<u8>, ...)` transfers the byte buffer into the document. The returned document still borrows its `Pdfium`; safe `load_pdf_from_byte_slice` borrows both bytes and owner. `Arc<Vec<u8>>` alone does not remove those lifetimes or make an owner-plus-borrower struct safe. [Loading APIs](https://docs.rs/pdfium-render/0.9.4/pdfium_render/prelude/struct.Pdfium.html#method.load_pdf_from_byte_vec).
- Safe `PdfPage::label() -> Option<&str>` reads PageLabel; safe `render_into_bitmap_with_config` exists. Raw `FPDF_GetPageLabel` calls are unnecessary for this contract. [Page API](https://docs.rs/pdfium-render/0.9.4/pdfium_render/prelude/struct.PdfPage.html#method.label).
- Safe render configuration provides BGRA format selection, transform/translate, destination-pixel clipping and `set_origin`. Clipping disables form-data rendering; `set_origin` is disabled when applying a custom transform. Do not combine them as interchangeable offsets. [Render config](https://docs.rs/pdfium-render/0.9.4/pdfium_render/prelude/struct.PdfRenderConfig.html).
- `PdfBitmap::as_raw_bytes() -> Vec<u8>` avoids channel normalization; `as_rgba_bytes()` normalizes to RGBA and violates this raster boundary. The raw method is an owned copy according to its signature; do not promise a zero-copy bitmap export. [Bitmap API](https://docs.rs/pdfium-render/0.9.4/pdfium_render/prelude/struct.PdfBitmap.html#method.as_raw_bytes).

**Planned design / unresolved gates**

1. Create `Pdfium` inside the one worker thread; load/use/close current and candidate documents there. Keep borrowed library objects within that thread's owner scope. Export only owned metadata, opaque document/request identities and BGRA buffers. Decide explicitly how shared source bytes are retained or copied; do not use unsafe lifetime extension.
2. Correct the initial 10 H-3 conflict: an I/O/structure worker may read/close the file and run pure-Rust `lopdf` checks, but must send candidate bytes to the existing PDF worker for PDFium loading, labels and prerendering. No second thread owns a candidate PDFium instance/document.
3. A sibling candidate is not committed state. Worker readiness and an exact reload request identity precede main-thread promotion; obsolete candidates/results are rejected and closed on the worker. Retain the old document/frames when loading fails. A matching EOF marker or two equal file sizes is only a readiness hint, not proof of a complete valid replacement.
4. Use the safe rendering wrappers before considering raw FFI. Golden fixtures must establish top-left normalization, page rotation/CropBox handling, matrix scale/translation, target bounds, BGRA order, alpha and row packing. Do not assume clipping yields `width * 4` packed rows or automatically solves alpha semantics without verification.
5. Ordinary synchronous page rendering is not a preemptible queue operation. Epoch changes cancel pending low-priority work and suppress stale results; an already running render may finish. A P0-before-P4 queue test must control dispatch timing instead of demanding impossible preemption.
6. Pin the PDFium runtime artifact/API compatibility and checksum separately from the Rust crate. Prefer explicit runtime library binding; `PDFIUM_DYNAMIC_LIB_PATH` is documented as a build-time linker setting, not an application runtime search-path contract. Document app-directory/system lookup and missing-library errors deliberately.

### 07: GPUI display placement

**Verified API surface at the required tag**

- Zed `v1.23.3` `platform.rs` defines `PlatformDisplay::{id,uuid,bounds}`; `bounds` returns `Bounds<Pixels>`. The trait has no display scale-factor field. `WindowOptions` includes `display_id` and `titlebar`, and `WindowBounds::Fullscreen` contains restore bounds. These signatures support an implementation spike, not proof of correct Windows placement. [Pinned GPUI source](https://raw.githubusercontent.com/zed-industries/zed/v1.23.3/crates/gpui/src/platform.rs).

**Planned design / unresolved gate**

- Keep GPUI calls in `ferrostage-ui`; let platform enumeration return plain data. Choose one of the already allowed GPUI dependency routes before implementation; do not infer GPUI Kit 0.7.3 has an identical Windows implementation merely from matching pin comments.
- Mapping `(position, native_size)` directly to GPUI bounds is unverified: physical desktop pixels and GPUI `Pixels` may differ under DPI, rotation or scale, and mirrored/overlapping outputs can produce ambiguous geometry. Prefer a verified native identity bridge where the pinned implementation permits; otherwise convert explicitly and require a unique match.
- On zero/multiple matches, keep the audience unavailable and surface a presenter-only warning. Do not invent a `DisplayId` from an OS enumeration index, assume `uuid()` has the same value as `MonitorId`, or choose the first equal rectangle.
- Verify the pinned Windows backend's display identity/coordinate conversion, display-change notification path, window creation/fullscreen behavior and focus handling before coding the shell. Source-level trait availability is not a passed PoC-1.

### Safe replacement and layered CI/distribution

**Verified:** Rust 1.99 `std::fs::rename` is safe and replaces an existing destination; it requires the same filesystem. On Windows it maps to `MoveFileExW` with a `SetFileInformationByHandle` fallback. There is no need for project-local raw Win32 just to attempt a sibling replacement. This API documentation does not establish universal power-loss durability or behavior under every filesystem/share mode. [Rust rename](https://doc.rust-lang.org/std/fs/fn.rename.html).

**Planned:** Treat atomic settings persistence separately from read-only PDF hot reload. Write/validate/flush a sibling settings temporary file, close its handle, and replace using a safe API; preserve the previous destination on failure and test an existing destination, denied replacement and cleanup. Never turn hot reload into a PDF writer or delete the destination before replacement.

| Verification layer | Planned evidence |
| --- | --- |
| Default portable checks | Preserve meaningful `just ci`: pure state/settings/document/scheduler tests, mocks and dependency licenses without native runtimes/windows. Feature-gated tests must not imply native functionality passed. |
| Native compile checks | Explicit Windows MSVC build/clippy for the selected GPUI/native feature graph; no window launch. A runtime-feature-disabled build alone does not verify the graphical app. |
| Native fixture integration | Known PDFium DLL/API with checksum and controlled fonts; render/page-label/crop fixtures. Selected media SDK/plugins and decode fixtures tested separately. |
| Real hardware | Three-screen placement, mixed DPI, hot unplug and measured PDF/media timings. Record machine, drivers, artifact versions and raw results when actually run. |
| Distribution audit | Exact DLL/plugin/font list, origin, hashes and notices; independent of `cargo deny`. Do not bundle a full system media installation implicitly. |

Current Ubuntu-only hosted CI and skeleton `just ci` do not establish Windows native-build or distribution success. Changes to CI/features/packaging belong to future implementation; none were executed here.

### 11: Media backend decision and license gate

**Verified comparison**

| Candidate | Primary evidence | Decision consequence |
| --- | --- | --- |
| `vlc-rs` | Published 0.3.0 dates to 2018; crate describes missing wrappers and uses MIT (CC0 examples). Its documented high-level video extension does not expose a callback method. [Package](https://docs.rs/crate/vlc-rs/0.3.0), [video extension](https://docs.rs/vlc-rs/latest/vlc/trait.MediaPlayerVideoEx.html). | A raw LibVLC callback path cannot be assumed safe under project `forbid`; ownership/callback and DLL-loader wrappers would need verification. No maintenance conclusion beyond published release evidence. |
| `gstreamer-rs` + AppSink | Maintained package documentation currently lists 0.25.4 and safe Rust APIs; bindings are MIT OR Apache-2.0, native GStreamer is LGPL-2.1-or-later. Safe AppSink callback/sample APIs and BGRA video format/frame mapping exist. [Package/license](https://docs.rs/crate/gstreamer/0.25.4), [AppSink](https://docs.rs/gstreamer-app/latest/gstreamer_app/struct.AppSink.html), [BGRA format](https://docs.rs/gstreamer-video/latest/gstreamer_video/enum.VideoFormat.html), [safe frame mapping](https://docs.rs/gstreamer-video/latest/gstreamer_video/video_frame/struct.VideoFrameRef.html). | Better documented candidate for the no-project-unsafe BGRA route. The observed versions are research snapshots, not a selected or upgraded project pin. |

- GStreamer's plugin sets have different quality/distribution caveats; `ugly`, third-party codecs and transitive native libraries cannot be approved solely from the core LGPL label. [Official plugin split](https://gstreamer.freedesktop.org/documentation/additional/splitup.html).
- GStreamer Rust build instructions require matching native SDK/runtime; `gstreamer-sys` build script probes system dependencies and only suppresses linking for docs. Ordinary native linking is not proven to permit the application to start when DLLs are removed. [Build script](https://docs.rs/crate/gstreamer-sys/latest/source/build.rs).
- `cargo deny` analyzes the Rust dependency graph and current allowlist; it is not a DLL/plugin license audit. Dynamically linking LGPL is not equivalent to a policy exemption or a complete distribution review. Do not blanket-add LGPL to the Cargo allowlist or assume a codec pack is allowed.

**Proposed decision:** Prefer a GStreamer AppSink feasibility spike because safe frame/callback APIs fit the existing lint policy; lock one backend only after the following gates pass. This is a conditional recommendation, not a final backend selection or implementation approval.

1. Choose exact Rust/native versions, toolchain, plugin/codec list and all native notices. Validate against existing Cargo license policy; resolve exceptions as explicit decisions.
2. Establish how V-8 (application starts without media DLLs) works. Feature gating alone separates builds; it does not prove a media-enabled binary tolerates missing startup imports. Verify a safe runtime-loader solution or an explicitly designed isolated media component, or revise the requirement before implementation. Do not assume catching `gst::init()` errors solves loader failure.
3. Negotiate BGRA and honor frame stride; produce packed `Arc<[u8]>` without keeping a borrowed mapped buffer after its owner is dropped. Single decoder and single audio output serve both windows. Do not equate hardware decoder availability with a proved zero-copy CPU BGRA path.
4. Measure decoder stop latency and frame/texture retention. A 1080p60/15-minute result, bandwidth bound or process VRAM stability is a hardware experiment, not an inference from the API.

## Related specs

- `.trellis/spec/backend/index.md`, `directory-structure.md`, `quality-guidelines.md`, `error-handling.md`: dependency direction, concrete errors, no unsafe, no leakage of file bytes, required checks.
- `.trellis/spec/frontend/index.md`, `state-management.md`, `type-safety.md`, `quality-guidelines.md`: snapshot/notes boundary, BGRA, one shared media frame, no child HWND and no UI-thread PDFium blocking.
- `.trellis/spec/guides/index.md`: inspect cross-layer contracts and verify claimed findings against actual sources.
- Initial 03/05/07/10/11 PRDs and parent `design.md`: intended requirements; conflicts above must be resolved in the planning artifacts before implementation starts.

## Caveats / Not Found

- No Pympress implementation was read. No code, dependency manifest, spec, active-task pointer, test, CI configuration or Git state was changed by this research.
- Some official URLs could not be retrieved. The pinned GPUI Windows implementation and a complete safe Windows identity/event wrapper were not established. The `libvlc-sys` package identity/version/license and callback coverage were not established; the existing `vlc-rs` package itself exposes a `sys` module. Do not use an unverified crate name as a dependency plan.
- Searchable docs under `latest` are snapshots and can differ by crate or cached page; exact media versions must be reconciled before locking. No claimed Windows installer size was measured.
- No three-screen machine experiment, native integration build, render benchmark, golden image, media playback test, DLL-removal test, font license audit or GPU-memory report was run. All PoC thresholds remain unverified targets.
