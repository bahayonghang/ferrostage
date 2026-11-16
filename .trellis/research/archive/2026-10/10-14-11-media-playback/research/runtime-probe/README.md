# Isolated backend feasibility evidence

Research inputs and receipts only; not adopted task11 production code. Original working prototype lives in `cache/media-research/rust-probe` with a separate workspace/lockfile. Source copies here make the checked experiment reviewable. Build scripts expect that original research directory layout and the reviewed import libraries; they are not installation commands.

- Safe Rust AppSink/VideoFrameRef:360 real video frames,82,944,000 owned packed BGRA bytes, exact output fingerprint in receipts. Row stride is checked; mapped storage is not retained after the sample.
- Parent/control binary has no GStreamer/GLib imports and starts with or without the runtime; its present-runtime child decodes the real fixture. This proves process isolation feasibility, not FerroStage GUI V-8 acceptance.
- Strict prototype Clippy and project cargo-deny policy passed. Native plugin/license and import inventories are separate from Cargo policy.
-20 selected plugins report LGPL. Whole wheels contain other GPL components and must not be copied/installed as a product runtime. Finish transitive/source/notice packaging review before backend adoption.
- `fixtures/` contains original synthetic test media generated locally:320×180 VP8/Vorbis moving ball/tone for12s, and160×90 three-frame GIF/WebP with120/240/360ms delays. No external visual/audio asset was used. These are project test inputs under the repository license. Generators/identities are included; do not interpret fixture generation or Pillow readback as product decoder acceptance.

Previous failed attempts remain documented in the parent preflight report; the first sandbox native run and early-AppSink-EOS result were not successful decode evidence. No system installation, global PATH edit, unsafe exception or additional Cargo license exception was made.
