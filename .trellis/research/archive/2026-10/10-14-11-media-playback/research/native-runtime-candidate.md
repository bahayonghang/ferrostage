# Native runtime candidate inventory

Research input for task11 I-1; no product adoption or distribution approval is asserted here. Use the exact per-file archive entry, byte count, hash and PE import closure in `runtime-probe/selected-import-audit.json`. Official PyPI1.28.8 wheels were verified against their published SHA256 before extracting named files. The complete wheels contain GPL components; only individually reviewed files may become runtime candidates.

## Source and notice mapping

The following versions are from official Cerbero tag1.28.8 recipes, retained in `cache/media-research/recipes`. Native binaries report GStreamer1.28.8(GIT)/plugin1.28.8.1; the archive hashes identify the binaries, and recipe references identify the source/notice inputs. This is not a claim that these local DLLs were rebuilt reproducibly from those recipes.

| Component | Selected files / use | Source version and notice |
| --- | --- | --- |
| GStreamer core | gstreamer/base DLLs, coreelements, plugin scanner; inspect/launch are research tools |1.28.8 source; LGPL COPYING |
| Base plugins/libs | app, playback, typefind, videoconvertscale, audioconvert/resample, ogg/vorbis; app/audio/video/pbutils/riff/tag/rtp libraries |1.28.8 source; LGPL COPYING |
| Good plugins | autodetect, matroska, vpx, isomp4 |1.28.8 source; LGPL COPYING; include vpx and compression notices below |
| Bad selected plugins/libs | wasapi/wasapi2, d3d11, mediafoundation, videoparsersbad; codecs/codecparsers/d3d11/d3dshader/dxva/winrt libraries |1.28.8 source; selected plugin metadata all LGPL. The package's `gpl=enabled` recipe does not authorize its other plugins |
| Test generators | videotestsrc/audiotestsrc |Base1.28.8 LGPL; research/fixture generation, not necessary product plugins |
| GLib | glib/gobject/gio/gmodule |2.82.4; actual `LICENSES/LGPL-2.1-or-later.txt` target, not the one-line COPYING link |
| proxy-libintl | intl-8.dll |0.5; LGPL COPYING |
| libffi | ffi-7.dll |3.2.9999.5; MIT, checksum-verified source-archive LICENSE |
| ORC | orc-0.4-0.dll |0.4.42; BSD-3-Clause recipe and full COPYING |
| PCRE2 | pcre2-8-0.dll |10.42; full LICENCE including JIT/third-party notices |
| bzip2 | bz2.dll |1.0.8; bzip2 LICENSE from checksum-verified archive |
| zlib | z-1.dll |1.3.1; original README license |
| Ogg / Vorbis | ogg-0/vorbis-0/vorbisenc-2.dll |1.3.5 /1.3.7; BSD-3-Clause COPYING |
| VPX | statically linked by Windows vpx plugin per recipe |1.14.1; BSD-3-Clause LICENSE plus PATENTS |
| Windows/VC runtime | PE imports resolved from existing Windows/System32 |OS prerequisites; not copied from the wheels or treated as project-owned redistributables |

Original notices and URL/hash receipt are in `cache/media-research/notices` and `runtime-probe/notices-receipt.json`. The HTML anti-bot response named `libffi-LICENSE` is explicitly invalid; use only `libffi-LICENSE-verified`. Keep full notices, source locations and dynamic replacement rights in any eventual runtime packaging. No Cargo LGPL exception is needed or authorized: the Rust binding graph passed existing policy; native libraries are a separate review.

Durable handoff: hash-verified original notice texts are now also under `runtime-probe/notices/`; the invalid HTML and GLib's one-line link are deliberately excluded (the full LGPL target is included). Exact recipe inputs are copied under `runtime-probe/recipes/`, with both trees included in `evidence-manifest.json`. These are research inputs, not a finished distribution. Final packaging must associate each selected binary with the appropriate full notices/source references and must not copy the wheels wholesale.

## Measured feasibility and remaining decisions

`runtime-probe/product-runtime-candidate.json` narrows the51-file research closure to47 files by excluding only gst-inspect, gst-launch, videotestsrc and audiotestsrc. None of the retained files statically imports an omitted file. All three needed official archive hashes were rechecked; exact download URLs, per-file archive entries/imports/hashes remain in the candidate. The18 retained plugins have the original loaded-license evidence. This is a proposed product input inventory, not adoption or redistribution:11 still owns the final helper, dynamic behavior and complete source/notice packaging review.

All20 selected plugin DLLs were loaded individually by gst-inspect with isolated search paths and returned0/LGPL; originals are in `runtime-probe/all-selected-plugin-licenses.json`. Safe Rust AppSink produced360 BGRA frames from the owned WebM. A separate std-only supervisor retained startup with the whole runtime absent; its PE imports exclude native media libraries.

Before product adoption, task11 must review this mapping against its final minimal plugin list, carry original notices/source references into packaging, and define bounded helper IPC, start/pause/seek/loop, one audio owner and cancellation. Actual media-enabled FerroStage GUI startup with libraries absent and real GIF/video integration remain required. No plugin installed globally, global PATH edit, GPL codec fallback, child HWND or unsafe waiver is authorized by these experiments.
