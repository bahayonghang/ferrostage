# Media CI repair evidence

Recorded on 2026-10-18. This is the media portion of the user's authorized CI
repair and delivery follow-through. Production sources were frozen before this
receipt was added. Remote rerun results belong to the coordinator's delivery
receipt; local success below does not establish a hosted-runner pass.

## Root cause and repair

Remote run `39122532405` first failed
`actual_helper_video_seek_repeat_pause_audio_and_reap` with a media timeout.
Four later native tests failed when acquiring the mutex poisoned by that panic.
The original log did not identify which wait timed out.

Temporary stage/callsite/latest-state probes and the selected runtime's plugin
inventory established the missing boundary: the original runtime supplied only
D3D11 H.264 video decoders. Media Foundation supplied AAC decoding and H.264
encoding, but no software H.264 decoder. With all D3D11 H.264 factories disabled,
playbin3 reported a missing decoder and continued playing AAC audio. The first
progress assertion passed; the following video-frame assertion timed out even
though audio reached EOS. This reproduces the remote failure shape on a host
that normally has working GPU decoding.

The repair adds only `gstopenh264.dll` and `openh264-7.dll` from the already
checksum-pinned GStreamer1.28.8 wheels. OpenH2642.6.0 provides software decoding;
the runtime is now49 DLL/EXE files, including19 plugins. Runtime filename checks
and the packaging manifest were updated together. Source recipes, the Cerbero
assembly patch, original BSD notices and the plugin's LGPL header notice are
retained with the existing LGPL text, making17 notice files. ADR-010 records
the decision and exact file hashes. No GPL/AGPL decoder or license/advisory
waiver was introduced.

The helper also checks missing-decoder bus messages after preroll and during
playback. It returns `MediaError::Unsupported` instead of claiming successful
audio-only playback when video decoding is missing. Existing supported
audio-only playback remains covered. All temporary probes were removed;
test timeouts now retain only callsite and compact state diagnostics.

## Executed red/green checks

| Check | RED, executed before repair | GREEN, executed after repair |
| --- | --- | --- |
| `software_h264_video_seek_repeat_pause_audio_and_reap_without_hardware` | Original47-file runtime reconstructed in `cache/media-runtime-no-software`; new subprocess regression ran with the old helper/preflight and all D3D11 H.264 ranks disabled. Failed at the video-frame wait with latest `(position_ms=790, duration_ms=800, paused=false, ended=true, error=None, frame=None)`. Child test22.82s, parent22.86s. | Updated49-file runtime passed the same original MP4/WebM/Ogg seek, pause/resume, repetition, frame, EOS and process-reap assertions. Included in the final11/11 suite below. |
| `missing_h264_decoder_reports_unsupported_instead_of_audio_only_success` | Before the helper bus fix, disabling D3D11 H.264 and OpenH264 produced audio progress without a typed error. Failed after15.01s, latest `(5957, 12010, false, false, None, None)`. | The same negative configuration returned `Some(MediaError::Unsupported)` without a frame. Included in the final11/11 suite. |

These are actual red executions, not inferred outcomes. The original47-file
staging command ran before the manifest/preflight update; repeating that command
on the final source stages49 files. A historical red replay therefore requires
the pre-repair helper/runtime manifest/preflight plus the new regression tests,
in an isolated checkout. The final preflight deliberately rejects an incomplete
old runtime earlier with `RuntimeUnavailable`.

The subprocess regressions use fresh plugin registries and process-local
`GST_PLUGIN_FEATURE_RANK` overrides. They disable the base D3D11 H.264 factory
and adapter slots0..63; the negative case disables `openh264dec` too. They do not
mutate the machine environment, replace the original MP4 fixture, increase
timeouts, skip tests, or recover poisoned locks.

## Final executed checks

| Command/check | Result |
| --- | --- |
| `./scripts/media-native.ps1 -Action build` | PASS, final helper rebuilt with `--locked` |
| `./scripts/media-native.ps1 -Action clippy` | PASS, native helper/all targets, `-D warnings` |
| `cargo clippy -p ferrostage-media --all-targets --features native-tests --locked -- -D warnings` | PASS |
| `cargo test -p ferrostage-media --features native-tests --locked` with the isolated clean Windows harness below | PASS:8 unit tests,11 integration tests,0 ignored,0 failed,0 doc tests; final integration duration38.90s |
| Earlier complete clean-process integration run | PASS:11/11 in37.56s |
| `rustfmt --check --edition 2024 crates/ferrostage-media/tests/playback.rs crates/ferrostage-media-helper/src/native.rs` | PASS |
| Scoped `git diff --check` | PASS; only Git's expected LF/CRLF normalization notices |
| `python scripts/stage-media-runtime.py --archives cache/media-research --output cache/media-runtime` | PASS, all49 selected binaries staged with sources/notices |
| Independent manifest/filename/staged-hash readback | PASS: manifest destinations equal `runtime-files.txt`,49 entries,19 plugins,17 notices, every selected size/SHA256 matches |
| `dumpbin /dependents` on both new DLLs | PASS: only existing selected DLLs and Windows/VC prerequisites; no unresolved additional native dependency |
| Pinned source archive readback | PASS: OpenH264v2.6.0 source SHA256 `558544ad358283a7ab2930d69a9ceddf913f4a51ee9bf1bfb9e377322af81a69` matches the retained Cerbero recipe; original LICENSE extracted from that archive |

The11 integration tests include ordinary native playback, software-only playback,
the missing-decoder negative path, GIF/WebP, partial/missing runtimes, embedded
video, original Beamer Movie metadata, pipe EOF, stderr draining and bounded
off-caller child termination/reaping. Native audio always used fakesink; no
desktop window or audible output was part of these checks.

## Local test-host caveat and reproduction

The Codex nested sandbox had independent execution problems: some embedded-media
temporary-file operations returned `SourceUnavailable`; some cold helper starts
timed out. Its TMP/TEMP paths were rewritten by the host. Executing the helper
outside that sandbox with only child PATH configuration instead returned Windows
`STATUS_DLL_NOT_FOUND` (`0xC0000135`), while placing the verified native DLLs beside
the same executable allowed loading. Those failed runs are not counted as passes
or treated as the remote missing-video failure. No production workaround or
timeout change was made for this local tool-host behavior.

Final native validation used a clean, approved Windows process and an isolated
copy of the final helper with selected `bin/*.dll` files adjacent. The runtime
argument still pointed at the actual staged runtime. Independent SHA256 readback
confirmed both helper copies were byte-identical:
`5767d5c5e08c7bc3bb43cf552af594473e222f2953435f4101f7f0f0a05319b8`.
The adversarial helper remained under the expected sibling `examples/` path.

The executed final harness, from the repository root, was:

```powershell
# Runtime/import libraries had already been prepared. For a fresh machine:
# ./scripts/prepare-media-runtime.ps1
./scripts/media-native.ps1 -Action build
./scripts/media-native.ps1 -Action clippy
cargo build -p ferrostage-media --example ipc_stall --features native-tests --locked

New-Item -ItemType Directory -Path cache/media-helper-run/examples -Force | Out-Null
Copy-Item -LiteralPath target/debug/ferrostage-media-helper.exe -Destination cache/media-helper-run/ferrostage-media-helper.exe
Copy-Item -LiteralPath target/debug/examples/ipc_stall.exe -Destination cache/media-helper-run/examples/ipc_stall.exe
Get-ChildItem cache/media-runtime/bin/*.dll | Copy-Item -Destination cache/media-helper-run
$env:FERROSTAGE_TEST_MEDIA_HELPER = (Resolve-Path cache/media-helper-run/ferrostage-media-helper.exe).Path
$env:FERROSTAGE_TEST_MEDIA_RUNTIME = (Resolve-Path cache/media-runtime).Path
$env:GST_REGISTRY_1_0 = Join-Path (Resolve-Path cache/media-helper-run) 'registry-final.bin'
cargo test -p ferrostage-media --features native-tests --locked
```

`registry-final.bin` was absent before the final run. For a fresh rerun use a
new registry filename; each new subprocess regression independently creates its
own fresh registry. The isolated helper/DLL copies live only in ignored local
cache directories and are not product packaging inputs.

## Remaining verification boundary

The remote runner's original log lacked the new callsite/state detail, so the
hardware-disabled local reproduction is the direct causal evidence; the next
hosted run must confirm the deployed repair. The coordinator owns workflow,
push, remote-run monitoring and issue closure. No native GUI, GPU performance,
hardware-audio latency or installer/distribution qualification is asserted here.
