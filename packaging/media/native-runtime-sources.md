# Selected native runtime sources and notices

ADR-009 and ADR-010 select the Windows x64 GStreamer 1.28.8 runtime in `runtime.json`: 49 executable/DLL files including 19 plugins. The manifest records the exact official archive URLs, sizes and SHA256, selected archive entries, per-file hashes and PE import closure. `scripts/stage-media-runtime.py` verifies and extracts only that allowlist. Entire upstream wheels contain unselected GPL components and must never be distributed wholesale.

## Source and notice mapping

The source versions below come from official [Cerbero tag 1.28.8](https://gitlab.freedesktop.org/gstreamer/cerbero/-/tree/1.28.8/recipes); complete recipe inputs are retained in `recipes/`. Recipes carry source URLs, commits/checksums and build options. Native binaries report core 1.28.8 (GIT) and plugin 1.28.8.1. Archive hashes identify the selected binaries; recipes identify source and notice inputs. This does not claim that local DLLs were rebuilt reproducibly from those recipes.

| Component | Selected files / use | Source version and notice |
| --- | --- | --- |
| GStreamer core | gstreamer/base DLLs, coreelements, plugin scanner |1.28.8 source; LGPL COPYING |
| Base plugins/libs | app, playback, typefind, videoconvertscale, audioconvert/resample, ogg/vorbis; app/audio/video/pbutils/riff/tag/rtp libraries |1.28.8 source; LGPL COPYING |
| Good plugins | autodetect, matroska, vpx, isomp4 |1.28.8 source; LGPL COPYING; include vpx and compression notices below |
| Bad selected plugins/libs | wasapi/wasapi2, d3d11, mediafoundation, videoparsersbad; codecs/codecparsers/d3d11/d3dshader/dxva/winrt libraries |1.28.8 source; selected plugin metadata all LGPL. The package's `gpl=enabled` recipe does not authorize its other plugins |
| OpenH264 | openh264-7.dll and gstopenh264.dll; CPU H.264 fallback | OpenH2642.6.0, BSD-2-Clause LICENSE from the source archive SHA256 `558544ad358283a7ab2930d69a9ceddf913f4a51ee9bf1bfb9e377322af81a69`; plugin1.28.8.1 reports BSD, with Ericsson BSD and GStreamer LGPL source-header notices retained |
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


Complete original notices are retained in `notices/` and staged unchanged. The libffi notice is `libffi-LICENSE-verified`, from the checksum-verified source archive; GLib uses the full LGPL-2.1-or-later text rather than its one-line COPYING link. All 17 notice files and source recipes accompany the selected runtime. No new Cargo license exception or GPL/AGPL codec fallback is introduced.

Dynamic LGPL replacement remains supported: acquisition and staging check original hashes, while the installed application checks availability by required filenames and permits compatible replacements. Keep source references, license notices and this mapping with the runtime. Windows/VC components are system prerequisites, not copied redistributables.

## Product boundary and verification

The product GUI links the pure media controller and GIF/WebP decoder. Only the separately built helper links GStreamer; one supervisor owns one active pipeline. Product background tests use the selected 49-file runtime for actual MP4/WebM/Ogg seek, pause, repeat, audio-only progress and process retirement, plus complete/partial runtime absence. GUI import inspection is a separate build gate. Actual visible media-enabled GUI startup/interaction belongs to the consolidated native smoke; it is not implied by these background tests.

The selected closure excludes gst-inspect, gst-launch, videotestsrc and audiotestsrc. No global plugin installation or PATH modification, child HWND, second media backend or unsafe waiver is part of this preparation. This manifest and staging tooling do not authorize installer publication.

## Software H.264 decoding

The two OpenH264 entries come from the same three already pinned archives; no additional archive or unselected codec is staged. `recipes/openh264.recipe` is the original [Cerbero1.28.8 recipe](https://github.com/GStreamer/cerbero/blob/1.28.8/recipes/openh264.recipe), including its retained assembly build patch. Its source archive is [Cisco OpenH264v2.6.0](https://github.com/cisco/openh264/archive/v2.6.0.tar.gz). The complete original BSD notice is `notices/openh264-LICENSE`; `notices/gst-openh264-LICENSE` preserves the original Ericsson BSD and GStreamer LGPL headers from [the pinned plugin sources](https://github.com/GStreamer/gstreamer/tree/1.28.8/subprojects/gst-plugins-bad/ext/openh264). Existing gst-plugins-bad-COPYING supplies the full LGPL text. This source/license inventory does not claim Cisco binary-service patent coverage for the Cerbero-built DLL.

Native regressions create an isolated plugin registry and disable all selected D3D11 H.264 adapter factories before exercising the original MP4 seek/pause/repeat/EOS and process-reap assertions. A separate regression disables OpenH264 as well: a missing decoder must return Unsupported, rather than successful audio-only progress for a video file. Hardware audio/GPU timing qualification remains separate.
