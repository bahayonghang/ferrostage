# Integration check summary, 2026-10-18

The independent Trellis checker audited the full combined feature diff including untracked modules, shared host/input/focus/lifecycle paths, menu transactions, localized warnings, notes isolation and the pure-core annotation query. Detailed evidence is in `../../10-18-layout-navigation-popover/research/check-review.md`.

One concrete integration bug was reproduced and fixed: switching from Settings to Jump or reopening Jump from a focused browser control did not focus the numeric editor. Presenter now tracks browser per-open identity. Actual key-input regression failed before repair and passed after; edits remain nonnavigating until Enter.

Final post-fix gates all pass: default `just ci` (198 tests), `just check-shell` (159 GPUI tests and native build/lint), combined UI strict Clippy with gpui/pdfium/test-support, verified-DLL real PDFium retention (1 test) and git diff --check. Policy graph unchanged; existing coordinator all-feature deny success remains applicable. No remaining concrete product-code defect was found. No commit/push/native launch by checker.

Native acceptance remains coordinator-owned. Frozen exe: `target/debug/ferrostage.exe`, 43,604,992 bytes, UTC modified 2026-10-18T14:21:09.8241416Z, SHA256 `35D8C54E16509067FBDA1AA2E3FA75D670583C85276D9B4280304C47A4E738E0`. No further checker build planned.

Coordinator follow-up: record normal-window native dark/zh live menu effects and light/en browser/private-note integration; sync spec with repeated-Jump/modal-to-browser editor focus behavior and `page_browser_jump_refocuses_editor_after_modal_and_reopen` test. No size/DPI matrix is required.

## Coordinator closure

Final native acceptance and spec synchronization are complete. See `verification.md` for exact captures, source identities, acceptance mapping and explicit scope limits. Dark/zh and light/en test processes both exited normally with empty stderr; product/PDF hashes were rechecked. After native acceptance, the checker isolated the new focus regression's settings store with exactly three test-only lines and removed its generated working-directory TOML. Targeted1 and full GPUI159 tests plus build/lint/fmt/diff passed again. No production code changed; the relinked EXE has a separately recorded current identity. No commit/push/archive was performed.
