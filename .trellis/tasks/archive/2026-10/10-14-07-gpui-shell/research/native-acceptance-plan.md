# Task07 native acceptance plan

Status: PARTIAL, remaining manual/native acceptance required. v4/v5 checkpoints retain the observed failures and historical passes. V6 F7 disappearing-control focus and F8 stale persistence feedback passed implementation, independent check and targeted native retest; successful chooser import and saved-role relaunch also passed. See native-v6-checkpoint.md. No full native acceptance or Gate B PASS; no controlled app or settings-lock process remains live.

## Inputs and isolation

Use the exact product executable/lock/source identity delivered by07, the verified PDFium runtime already present in ignored cache, repository-owned PDF fixtures, and independent `--config` files beneath07 research/native. Never use or overwrite the default user settings. Record executable SHA, source manifest, environment, startup arguments, DLL identity, config bytes and fixture hashes before launch; preserve app logs and expected failures. Root alone controls the desktop through Computer Use. The implementer/checker do not launch competing UI sessions.

Query current monitor enumeration and GPUI UUID mapping from the actual product context. The older DISPLAY1/2/5 inventory is a lead, not current placement proof. Give A/B/C identities only after complete current mapping; persist MonitorIds/roles, not indexes. GdiDeviceCapsRatio is not effective DPI evidence.

## Required observed steps

| ID | Procedure | Acceptance evidence |
| --- | --- | --- |
| N01 | Start with fresh isolated config, Presenter B / Audience A / None C, no PDF | Presenter is on actual B, Idle text and dominant Open action; no audience; no running timer |
| N02 | Import repository PDF using CLI, then separately native chooser and supported drag/drop path | Ready and filename/page placeholder; no audience or timer; chooser belongs to confirmed Presenter; only tested import mechanisms marked passed |
| N03 | Import malformed/missing PDF with existing document | Presenter shows meaningful error/retry; old document, state and audience safety preserved; no user PDF bytes change |
| N04 | Click Start then Stop | Actual requested A and observed fullscreen placement/mode; pending output black; only verified success publishes Live; Stop removes audience and returns Ready |
| N05 | Focus audience; invoke PageDown, B and Escape; invoke Open intent while audience has focus | Current page/black state/stop actions observed; all UI returns to current Presenter; audience has no controls/dialog/focus ring; Escape modal/preview ordering tested where07 supports it |
| N06 | Change only isolated role config to C Audience and relaunch; repeat Start/Stop | Actual C receives audience, A unchanged; requested roles and actual IDs/bounds in log plus screenshots |
| N07 | Invoke Swap during presentation, including observed state before/after | Replacement windows go to mapped roles; core page, stage and timer continuity; old close callbacks do not stop replacement; persisted bindings read back from isolated config |
| N08 | Exercise invalid/conflicting keybindings and settings save failure via isolated fixtures | Named warnings and disabled conflicts; effective values retained dirty with Retry; only actually observed error paths accepted |
| N09 | Light and dark native UI; system theme selection without changing Windows settings | Actual configured themes observed; system selection resolves to current OS appearance. A real OS theme-change event remains NOT_RUN unless separately observable |
| N10 | Resize native presenter to DESIGN.md sizes1440x900,800x600,390x844,320x640 logical or record enforced minimum | Text and primary actions remain visible, long filename handled,44px targets, no clipped states; record actual size/scale, not requested size alone |
| N11 | Keyboard Tab/Shift-Tab, Enter/Space, hover/pressed/focus/disabled/error and observable loading | Accessible names/states where API supports them; visible2px focus gap, restored focus after modal, no essential animation; each state recorded individually |
| N12 | Clean close and relaunch with saved isolated config | Owned process/window identity disappears; worker/poller shutdown observable; next startup uses intended roles; no unrelated process terminated |
| N13 | Relaunch with the existing en-US language setting in isolated config |07 static labels, stage and accessibility names use the selected locale; Chinese filename/font fallback remains legible; no09 language settings UI is implied |

Native screenshot URLs are observed through Computer Use; save original captures as durable test evidence only, never substitute browser mockups or generated screenshots. Logs/automated transaction tests complement screenshots but do not prove actual focus or physical display transitions. Product pixels are placeholders in07; no real PDF/notes/GPU performance acceptance is claimed.

## Explicit unavailable cases

User explicitly cannot perform physical unplug/replug or interface changes this turn.03 I-6 and Gate B physical topology stability and latency remain NOT_RUN. Current three-screen hardware with disabled app roles is not a real single/zero-display machine. Single-display presenter_only/windowed and zero-display state machines can be tested automatically, while corresponding native hardware scenarios remain NOT_RUN. Do not re-ask for physical cooperation or alter OS display/security settings to manufacture evidence.

Gate B remains incomplete until its physical/native requirements pass;08 integration cannot start merely because available three-screen cases pass. Gate C/D, M1 long presentation, real raster workbench, VRAM, settings UI, reload and media remain downstream acceptance. This plan creates no additional task and does not authorize commit/push/archive/install/publication.

## Save-failure fixture protocol (v5 feedback FAIL F8; v6 repair retest PASS)

After launching a-dark or a-conflicting-keys with the next frozen executable, run `native/hold-settings-read-lock.ps1 -Profile <profile> -RunTag <unique-tag>` and retain its live exec session. The helper exclusively targets that task-owned config, records its hash and opens a read-only handle without delete sharing; no user config or PDF is touched. Trigger Swap on an eligible actual two-role setup and inspect the unsaved warning/Retry. Read back that runtime roles changed while disk bytes retained their before hash. Create the named release marker after observation, collect unlock receipt, click Retry, and verify saved bindings by readback/relaunch. The handle releases automatically after ten minutes if interrupted; a timeout is not evidence of Retry success. This is an actual Windows replacement error, separate from unit failure injection. No fabricated PASS until UI/log/readback outcomes exist.

## Import-error native path (v5 observed PASS via recent-file path)

`a-conflicting-keys.toml` now seeds two task-owned recent paths: malformed fixture and deliberately missing fixture, preserving its original bytes in a sibling backup and hashes in `native/error-recents-preparation.json`. Launch with valid CLI PDF so it occupies recent0 and Ready, then click malformed/missing recent entries through actual UI. This exercises the production recent→deferred Import path and checks old-document retention without depending on the chooser's failed Computer Use text injection. It does not count as chooser or drag/drop acceptance. Both conflicting Space bindings remain part of N08.


Current results: native-v6-checkpoint.md and retained native-v5-checkpoint.md. Root repaired its own CRCRLF fixture generation before the successful recent/conflict run; use error-recents-preparation-v2.json. The first malformed TOML fallback and temporarily overlapping system run remain excluded from their intended acceptance. All controlled V5/V6 runs and lock handles returned exit0. V6 chooser selection used folder navigation after the earlier CU text-injection limitation; this proves chooser import, not drag/drop.
