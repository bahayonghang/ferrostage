# ADR-010: Software H.264 decoding in the isolated media runtime

Status: accepted for the authorized remote CI repairs on 2026-10-18.

The original ADR-009 runtime selected D3D11 H.264 decoders only. A hosted Windows
runner without a compatible hardware decoder let playbin3 continue with AAC
audio, emitting progress and EOS without a video frame. The native video test
then timed out; its mutex panic caused four subsequent test failures. Disabling
every D3D11 H.264 adapter factory reproduces that exact missing-frame state.

Add only `gstopenh264.dll` (40,448 bytes, SHA256
`aa9117b6deaa27d6048043316843e7e9666b9bd44498785d0b618c17670d9029`) and
`openh264-7.dll` (920,064 bytes, SHA256
`899c5d69bc871e575eeb04aaad10592010b0e36d3ebc30a4b2dcf116f942e120`) from the
same GStreamer 1.28.8 archive identities already recorded in
`packaging/media/runtime.json`. The selected closure is now 49 DLL/EXE files,
including 19 plugins. PE inspection finds only existing selected dependencies
and Windows/VC system prerequisites; no additional native dependency is needed.

OpenH264 2.6.0 is BSD-2-Clause. Retain the original Cerbero 1.28.8 source recipe,
its assembly build patch, and the original license extracted from the recipe's
checksum-verified source archive. The GStreamer OpenH264 plugin reports BSD;
retain its complete Ericsson BSD and GStreamer LGPL source-header notices with
the existing LGPL COPYING text. The source mapping and 17 original notices are
staged with the runtime. This does not assert Cisco binary-service patent
coverage for Cerbero-built binaries. No Cargo license/advisory exception or
GPL/AGPL codec is added, and no complete upstream wheel is distributed.

The hardware decoder keeps its existing rank; OpenH264 is the software fallback.
The GUI remains independent of native media imports. The helper now treats a
missing-decoder bus message as Unsupported, both after preroll and during
playback, instead of silently downgrading a video file to audio-only success.
Audio-only files with a supported audio decoder remain valid.

Native regression tests run in subprocesses with fresh plugin registries and
disabled D3D11 H.264 adapter factories. They reuse the original MP4/WebM/Ogg
seek, pause, repeat, frame, EOS and child-reap assertions. Another regression
disables OpenH264 too and requires the typed Unsupported result. Neither skips,
fixture substitution, timeout increases nor poisoned-lock recovery are used.
