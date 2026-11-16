# Native validation — 2026-10-19

## Isolation and launch

The first desktop build (before timing integration) completed successfully with one expected unused PlaybackEvent-field warning, pending its Shell subscriber. Verified PDFium staging completed. This is not the final strict lint gate.

Used `%TEMP%/ferrostage-defense-1-1-0/settings.toml`, Chinese locale, dark theme, WenYuan Serif SC at 18 logical pixels, default pane ratios. The app opens the authorized regenerated thesis PDF read-only. No user settings/history are used or changed.

The first test process was launched with hidden startup state and produced no targetable window; it was terminated after verifying its exact executable/PID and lack of a main window. Relaunched as the explicitly interactive native-test app. PID 71860, initial HWND 14684792, matched against the returned Computer Use window. The inherited OpenAI.Codex AppUserModelID does not identify the executable; exact PID/handle corroboration establishes that this is FerroStage, not Codex automation.

## Observed before fullscreen

- An unobscured normal Ready window displayed the final thesis PDF: current page left, Next upper-right, original Notes lower-right; page count 1/95. No text-mode or font-size controls were added.
- Display settings detected three monitors. Selected the built-in NE160QDM-NM9 as Presenter and horizontal BKX270 as PDF audience. The app correctly rejected the temporary same-monitor role conflict before the audience selection changed.
- Apply migrated the existing document/session to the laptop with a new Presenter HWND 596254. Runtime placement confirmation matched the built-in MonitorId and `actual_fullscreen=false`; stage remained Ready and elapsed time zero.
- Laptop screenshot capture was blocked by a full-screen video. One fresh activation retry still returned the occluding video. Those captures are not accepted as FerroStage visual evidence. User was asked asynchronously to move the occlusion while implementation/checks continue.

## External-display smoke, pre-integration build

Used a copy of the same debug executable at `%TEMP%/ferrostage-defense-1-1-0/native-smoke.exe` (PID 153848) and a second isolated `external-settings.toml`. This avoids holding the build output open during integration. Selected horizontal BKX270 as Presenter and the third portrait external monitor as audience through the app's actual display picker. Laptop occlusion was not bypassed or treated as laptop acceptance.

- PASS: Start entered native Presenter fullscreen, removed the title bar, compacted the controls, and created confirmed fullscreen audience output on the assigned third display. Presenter remained HWND 12718084 in place. Runtime stage committed only after placement confirmation.
- PASS: The audience's returned untitled window displayed the public title slide with aspect-preserving letterboxing and no Notes or presenter controls.
- PASS: End closed audience and restored the original normal Presenter bounds `[474.4, 184.0, 1100.0, 760.0]`. Runtime logged `presenter restored` with `actual_fullscreen=false`; the screenshot showed the original title bar/window size. Remaining talk time was retained on Stop.
- PASS: A second Start worked. Actual content-page input accepted 78 and Enter navigated to physical content page 155, paired with the dense original Notes page 156. The complete last paragraph was visible without clipping; Next showed logical slide 79. This is display/layout evidence, not a standing-distance judgment.
- PASS: Blackout made the actual audience window fully black while Presenter slide/Notes remained visible. The toggle could be released. Targeted Alt+F4 closed the task-owned process and its audience without stderr errors.
- INCONCLUSIVE: Maximize was visibly observed, but the following coordinate action reported a non-target foreground window; after activation retry, Presenter was normal again. This does not establish a product restore failure or native maximized-Start acceptance. Do not claim it passed.

Screenshots are private local evidence in the temp directory: `presenter-fullscreen.png`, `audience-public-only.png`, `presenter-restored.png`, `presenter-dense-notes.png`. Runtime receipt is `external-stdout.log`. No private images were copied into the repository.

## Integrated laptop and dual-display acceptance

The user cleared the video obstruction. The final integrated debug executable (including the failed-worker playback guard) ran from the isolated temp directory, PID 40528 / Presenter HWND 10750796. Laptop NE160QDM-NM9 was Presenter; horizontal BKX270 was Audience.

- PASS: normal Start entered native laptop fullscreen in place. Current/Next/Notes followed the requested hierarchy, title bar disappeared, and essential session controls remained visible.
- PASS: original dense Notes for content slide 78 / physical Notes 156 were complete and legible in the captured laptop pane, including the final paragraph. The source typography was 33.3% larger at equal raster width. This is visual layout evidence, not a judgment at standing distance.
- PASS: the actual assigned audience window showed only the public content slide. An unrelated OS security notification briefly overlaid its corner; that notification is not FerroStage content and was not interacted with.
- PASS: End restored the normal window. Maximizing through the native title-bar button, Start, and End then restored the maximized title-bar/window state. This replaces the earlier inconclusive maximize attempt.
- Evidence: `laptop-fullscreen-final.png`, `laptop-dense-final.png`, `laptop-maximized-restored.png`, `integrated-stdout.log`, `integrated-stderr.log` under the isolated temp directory. No runtime error was observed.

## Standard transitions on the real audience display

PID 118480 / Presenter HWND 1186200 / Audience HWND 26939868 used a clean-room local derivative of the committed generator. The initial 51-page fixture used six-second effects to make screenshots possible. After capturing Replace, Fade and Dissolve, the task-owned temporary fixture was regenerated as a compact 14-page sequence: initial R, one representative of each of the twelve standard families, and an additional Fly `/Di 180 /M /O /SS 3 /B false`. Only the temporary fixture was changed; no user PDF or committed fixture was overwritten. Hot reload retained the current public page and accepted the new document generation.

- PASS: actual audience intermediate screenshots show distinct Fade, Dissolve, Split H/I, Blinds H, Box I, Wipe 0, Push 0, Cover 0, Uncover 0, Glitter 0 and Fly 0/I/SS1/Bfalse behavior. Replace switches directly. Presenter previews remain immediate and stable.
- PASS: Fly moves changed rectangles while the shared grid/background remains fixed. Enlarged outgoing Fly SS3 moves offscreen; the final capture has only the destination rectangles, with no residual old pixels.
- Local evidence: `transition-{fade,dissolve,split,blinds,box,wipe,push,cover,uncover,glitter,fly}-mid.png`, `transition-fly-large-mid.png`, `transition-fly-large-end.png`, `transitions-stdout.log` and `transitions-stderr.log`.
- Qualification: these are actual native screenshot samples, not a continuous recording or exhaustive native verification of every parameter combination. All 51 authored fixture cases and geometry/scene regressions are covered separately by automated checks. Do not label every native variant or interruption scenario PASS from representative samples.

## Compiled overlays, multiple Notes and authored dwell

PID 148104 / Presenter HWND 8132158 / Audience HWND 60691008 opened the committed `fixtures/beamer-timing/overlays-timing.pdf` (33 physical pages, 16 content steps, 6 logical slides).

- PASS: manual Next exposed the second pause step with correct slide/step numbering; Next preview skipped the independent Notes pages.
- PASS: two original Notes pages retained their natural raster heights. Scrolling only the Notes pane brought the second page fully into view, while Current/Next stayed fixed (`multiple-notes-scroll.png`).
- PASS: jumping to content step 9 / physical 18 ran the zero-duration chain and stopped at physical 24 / logical 4 step 4 with the final bar/value 140. Native screenshots establish final progression, while automated paint-event tests establish that intermediate steps are acknowledged before advancing; screenshots alone cannot prove every zero-duration scanout.
- PASS: explicitly pausing animation, then manually entering the positive 1.5-second dwell step, kept it stable for over 20 seconds. Talk countdown continued independently. Resume advanced through the zero-duration Fade step to the manual step and stopped (`timing-paused.png`, `timing-resumed-manual.png`).
- PASS: the final timed page remained stable without wrapping. The actual audience screenshot contains `PUBLIC-FINAL-STABLE` and no Notes (`timing-audience-public.png`). All task-owned test processes and audience windows closed normally.

## Packaged 1.1.0 final smoke

Launched the verified release payload's actual `ferrostage.exe`, PID 65092 / HWND 4332738, using a separate `already-fullscreen.toml`. No explicit PDFium-library override was used: the staged DLL and notices came from the release payload. Opened the regenerated thesis PDF successfully.

- PASS: startup-fullscreen Ready state was actually fullscreen. Start retained fullscreen, created assigned audience output, and displayed the original Notes. Actual thesis navigation to content page 2 updated the content/Next/Notes pairing.
- PASS: End closed audience and retained the prior fullscreen Ready state, current page and stopped talk time (`release-already-fullscreen-restored.png`). Native readback/logs corroborated the state. Targeted Alt+F4 closed the test instance; stderr was empty.
- Final thesis PDF SHA256 remained `3f7246010bfbd5e6352d4abcdeb119c73093f693d588042aa8f49398d5d24f45` after all app acceptance. The application did not modify it.

## Explicit qualification limits

NOT_RUN natively: forced placement failure/monitor unplug recovery, every transition parameter combination, continuous animation recording, every rapid-input/blank/reload race, and per-step OS scanout instrumentation for zero dwell. The corresponding deterministic lifecycle/geometry/paint-order regressions passed; they are not substituted for native observations. Actual standing-distance comfort remains a human judgment. No installation, signing, clean-machine test, publication or push was performed.
