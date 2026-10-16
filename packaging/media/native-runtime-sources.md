# Selected native runtime sources and notices

ADR-009 selects the Windows x64 GStreamer 1.28.8 runtime in `runtime.json`: 47 executable/DLL files including 18 plugins. The manifest records the exact official archive URLs, sizes and SHA256, selected archive entries, per-file hashes and PE import closure. `scripts/stage-media-runtime.py` verifies and extracts only that allowlist. Entire upstream wheels contain unselected GPL components and must never be distributed wholesale.

## Source and notice mapping

The source versions below come from official [Cerbero tag 1.28.8](https://gitlab.freedesktop.org/gstreamer/cerbero/-/tree/1.28.8/recipes); complete recipe inputs are retained in `recipes/`. Recipes carry source URLs, commits/checksums and build options. Native binaries report core 1.28.8 (GIT) and plugin 1.28.8.1. Archive hashes identify the selected binaries; recipes identify source and notice inputs. This does not claim that local DLLs were rebuilt reproducibly from those recipes.

| Component | Selected files / use | Source version and notice |
| --- | --- | --- |
| GStreamer core | gstreamer/base DLLs, coreelements, plugin scanner |1.28.8 source; LGPL COPYING |
| Base plugins/libs | app, playback, typefind, videoconvertscale, audioconvert/resample, ogg/vorbis; app/audio/video/pbutils/riff/tag/rtp libraries |1.28.8 source; LGPL COPYING |
| Good plugins | autodetect, matroska, vpx, isomp4 |1.28.8 source; LGPL COPYING; include vpx and compression notices below |
| Bad selected plugins/libs | wasapi/wasapi2, d3d11, mediafoundation, videoparsersbad; codecs/codecparsers/d3d11/d3dshader/dxva/winrt libraries |1.28.8 source; selected plugin metadata all LGPL. The package's `gpl=enabled` recipe does not authorize its other plugins |
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


Complete original notices are retained in `notices/` and staged unchanged. The libffi notice is `libffi-LICENSE-verified`, from the checksum-verified source archive; GLib uses the full LGPL-2.1-or-later text rather than its one-line COPYING link. All 15 notice files and source recipes accompany the selected runtime. No new Cargo license exception or GPL/AGPL codec fallback is introduced.

Dynamic LGPL replacement remains supported: acquisition and staging check original hashes, while the installed application checks availability by required filenames and permits compatible replacements. Keep source references, license notices and this mapping with the runtime. Windows/VC components are system prerequisites, not copied redistributables.

## Product boundary and verification

The product GUI links the pure media controller and GIF/WebP decoder. Only the separately built helper links GStreamer; one supervisor owns one active pipeline. Product background tests use the selected 47-file runtime for actual MP4/WebM/Ogg seek, pause, repeat, audio-only progress and process retirement, plus complete/partial runtime absence. GUI import inspection is a separate build gate. Actual visible media-enabled GUI startup/interaction belongs to the consolidated native smoke; it is not implied by these background tests.

The selected closure excludes gst-inspect, gst-launch, videotestsrc and audiotestsrc. No global plugin installation or PATH modification, child HWND, second media backend or unsafe waiver is part of this preparation. This manifest and staging tooling do not authorize installer publication.
