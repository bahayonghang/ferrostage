# Native v4 checkpoint

Recorded 2026-10-15T01:48:07.300783+00:00. Historical executable `0e435312372a914db92e7dcb1509c28ad6c739ab3db660514a205879bcb10615`, source snapshot `208d6c5ec1069ab5e323cb0112c2191a902509f0f0ff47a534ef2ea87114c052`. All602 source files/exe/PDFium matched at resume (`native/v4-resume-identity.json`). Subsequent focus/Open repair is IN_PROGRESS and must receive its own freeze.

Environment: Windows three-screen setup; actual mappingA=DISPLAY1/131073, B=DISPLAY2/65731, C=DISPLAY5/65735 from product logs. DPI differs: A logical bounds1706.6666x1066.6666 for physical2560x1600, B2048x1152 for2560x1440. These are observed GPUI bounds, not a claim that GDI scale1.0 is effective DPI. No monitor was physically unplugged.

| Case | v4 observed result | Limits / next action |
|---|---|---|
|N01|Existing Idle PresenterB/noAudience, then normalclose exit0|Resumed window had pre-existing rawEnglish error; trigger UNKNOWN. Fresh CLI run did not show it.|
|N02|CLI own long-Chinese-name9-page PDF→Ready/noAudience. Ctrl+O creates real chooser.|Chooser path import UNVERIFIED: Computer Use indices/focus inconsistent; single key d works, type_text removes selection but inserts no path. Drag/drop and recent activation not yet run.|
|N03|NOT_RUN|Need malformed/missing import while prior document retained; native text injection limitation remains.|
|N04|Start→actualA fullscreen Presenting; Stop→Ready/noAudience|Curtain transient not directly captured; separate automated and placement evidence only.|
|N05|Audience PageDown1→2, B black, Escape→Ready/page2. Ctrl+O moved chooser to PresenterA afterSwap.|Disabled Open button versus allowed shortcut inconsistency requires fix; no final Open guard acceptance yet. Cursor/OS IME overlay is not app-only cursor proof.|
|N06|NOT_RUN|C vertical role startup pending.|
|N07|Swap→new PresenterA/AudienceB; page2/black held, B restores2, persisted config roles read back|Timer UI belongs08; no visual timer continuity claim. Relaunch saved bindings pending.|
|N08|NOT_RUN|Conflicting keys and isolated save failure/Retry pending.|
|N09|Light observed|Dark/system pending, OS theme-change eventNOT_RUN.|
|N10|Actual captures1323x929,801x655,391x655,322x640: primary visible, filename truncates, warnings wrap|Exact1440x900/800x600/390x844/320x640 logical matrix incomplete. Border drag works only at outermost pixel; tool disallows out-of-window growth endpoints. Maximize/restore then shrink may cover larger probes.|
|N11|TabOpen/ShiftTabStart ringsPASS; native Start-focused Space increments page, Enter doesnothing FAIL|Red images saved; implementation repair active. Hover/loading/modal restoration not all accepted.|
|N12|Both runs closed viaactualtitlebar; PID167888 and66348 exit0 receipts|Latest A/B persistedrole relaunch pending; workers shutdown inferred from owned process exit, no unrelated process touched.|
|N13|Chinesefilename visibly rendered|en-US startup pending.|

Runs: `v4-a-light` session30311, `v4-a-cli` session54803; both completed exit0. Original screenshots/metadata and stdout/stderr reside in `native/`. Old firstdraw/close/input failures in previous freeze reports remain historical red evidence.

GateB remains incomplete: native matrix, input bugfix/retest and independent product check pending; physical unplug/port and real single/zero-display casesNOT_RUN by user availability.08–11 remain in original tree and retain full scope. No commit/push/archive/journal.
