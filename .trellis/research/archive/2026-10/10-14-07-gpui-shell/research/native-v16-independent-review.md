# V16 independent native evidence review

Reviewed2026-10-15 after Root completed both V16 core-native runs. **No report inaccuracies or gate inflation found. The bounded F14/native close PASS is supported. Gate B remains INCOMPLETE.** This is an evidence audit: no desktop operation, native/hidden HWND launch, product change or replacement of sealed records was performed.

## Findings (fixed)

None required. Root's core results explicitly distinguish physical-page placeholders from actual PDF/notes rendering, bounded teardown tests from universal race safety, and core checks from complete Gate B acceptance.

## Findings (not fixed)

No new defect found. Existing limitations remain: simultaneous destruction/conversion, HWND reuse, full provider-method/drain coverage, physical unplug/reconnect/port/latency and real single/zero-display evidence are not established. The user deferred the size matrix. Task07 still renders placeholders; actual current/next PDF and private notes UI remain task08 work. Historical V14 failures and the earlier V15 baseline-EXE receipt clarification remain unchanged.

## Verification

- Independently recomputed all138 native manifest file sizes/hashes, including47 original JPGs. Manifest SHA256 is `cad5c4ee34a7573c774efed77edb10cb53895201736fed2afd390501b13f91dc`. All47 screenshot-sidecar hashes also agree.
- Recomputed all648 frozen source hashes and the current EXE/PDFium/PDF hashes; all match the V16 freeze. EXE remains `cf228c03655ddd45d25520bd88dab025c219af9bec9195314ce541dbb1cd89d1`. No source checks needed repetition. Prior strict lint/typecheck PASS remains applicable, not rerun.
- Independently parsed raw launch/exit/inventory records and scanned full126/48-line stdout plus both empty stderr files. No panic, ERROR, failed, window-not-found, invalid-window or cannot-unwind diagnostic appears. Both launches enable `RUST_BACKTRACE=full` and bind the current EXE/fixture/runtime.
- Main PID42672 exited0 at10:19:07.7565187Z; empty final inventory is later at10:19:17.829Z. Relaunch PID58568 exited0 at10:21:18.4957160Z; empty inventory is later at10:21:41.493Z. Thus the report does not mistake a transitional immediate-close capture for final emptiness. A fresh read-only process query during this review returned no FerroStage/compiler/AccessKit test process.
- All raw placement observations match requested/actual GPUI display IDs and fullscreen mode. Complete native monitor records show BKX2560x1440 primary, BOE2560x1600 left, RTK1600x2560 rotated270 degrees right, mapped to65729/131073/65733. These are runtime mappings; saved bindings contain full MonitorIds.
- Before-Cancel bytes hash to `d97a7e55c0ef19f93d3a815369603e9a2a903401df636355048d09db1efaa587`, matching both hashes in Root's immediate before/after readback receipt. Parsed settings match the launch fixture. The after-Cancel raw byte sequence is represented by that readback receipt, not a separately archived TOML copy; the reopened panel and unchanged-window observations corroborate the recorded Cancel result.
- Independently calculated the no-play gap from raw log timestamps10:18:36.092057Z and10:18:59.164054Z: **23.071997 seconds**, both Ready/page2/elapsed191785ms. The commit has no Audience. Before that, migration stays Presenting/page2 while elapsed grows87767→121758→165136ms.
- Saved no-play settings contain Presenter BKX, no Audience and `presenter_only`. Main after-config equals relaunch before-config byte-for-byte. Final config restores RTK Audience with full IDs. Fresh document page1 on relaunch is correctly distinguished from within-session page preservation.
- Visually inspected the three original Identify overlay images:1,2,3; the conflict image shows same-target selection, explanatory rejection and disabled Apply; no-play image shows Ready3/5 with disabled Start. Representative page3/page4 images contain only numerals, confirming the report's placeholder limitation. The Jump image is a Presenter modal; the10:16:40.115766Z log confirms Presenter activation. The migrated Audience image is white, and the post-Audience-close image shows a surviving Ready Presenter.
- Root's adopted-source hidden regression receipt exits0 and records all16 distinct destruction cycles, each performing four retained-context COM queries:64 total. Source equivalence was already independently verified before release. The initial exit1 is explicitly retained as `mbx[error]: Access is denied`, before tests. The passed deterministic post-destruction test is not represented as concurrent race coverage.

The first ad-hoc manifest inspection assumed a wrapper `files` key and returned KeyError without modifying data. Inspection then used the actual top-level path map and successfully checked all138 records. This reviewer tooling mistake does not affect native evidence.

## Release and scope

Root may reference the reviewed bounded V16 results in current task/parent status and continuation instructions. No sealed report correction is needed.03/07 remain in progress;08–11 remain dependency-gated and P2 remains intact. There is no approval to advance Gate B from this audit. No commit, archive, add_session or product modification occurred. Only this report and its independent evidence manifest were added.
