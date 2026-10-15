# v4 input-event boundary and presenter details

Root's v3 visible native Presenter confirmed display65731 and exited0 after titlebar Close. Ctrl+O produced `window not found` without a chooser. Real GPUI keyboard and mouse regressions reproduced that exact error: the input callback had leased the originating window, while Shell::request_focus immediately called its AnyWindowHandle::update.

The single actions::dispatch boundary now defers the owned intent and weak controller until the input window is returned to App. Keyboard, mouse, accessibility and drop routes share that boundary; popup focus confirmation is unchanged. Additional tests cover Audience-origin activation of Presenter and Escape invalidation. This was implemented before the later v5 Presenting admission fix; the v4 audience-open test concerned routing only, and does not prove mid-presentation chooser eligibility.

Root's requested presentation details were also implemented: default unsupported shortcuts use localized laser/pen labels; both catalogs have47 keys; Styled::font carries explicit Segoe UI→Microsoft YaHei UI and Georgia→SimSun fallbacks using installed system fonts. The truncated filename has a GPUI hover tooltip containing the full name and existing accessible label/title. No fonts were downloaded and no vendor source changed. Actual font fallback/tooltip acceptance remains in Root's native matrix.

| Receipt | Result |
| --- | --- |
| product-v4-input-red.log | exit101,2 real keyboard/mouse failures with `window not found` |
| product-v4-input-green.log | exit0,26 tests |
| product-v4-headless-final.log | exit0,26 tests after fallback/tooltip additions |
| product-v4-just-ci.log | exit0,145 default tests and all CI gates |
| product-v4-clippy.log | exit0, strict app/UI all-features/all-targets |
| product-v4-build.log | exit0, native gpui,pdfium build |
| product-v4-diff-check.log | exit0 |

Exact commands and actual exits are retained in product-status.txt. product-v4-source-delta.json/repair patch distinguish six implementation inputs from Root's DESIGN.md status change;595 of602 source inputs are unchanged. Lock and vendor graph are unchanged, retaining the preceding full all-features policy evidence. This report was finalized after an interruption; it does not replace Root's later v4 native findings or the superseding v5 repair.

Freeze product-native-freeze-v4.json SHA256 `6b6264ff0136d32943fcc3aec0e43268916f9430b8f80c8ee3f6389b0c4da4d2`; source aggregate `208d6c5ec1069ab5e323cb0112c2191a902509f0f0ff47a534ef2ea87114c052`; executable `0e435312372a914db92e7dcb1509c28ad6c739ab3db660514a205879bcb10615`; patch `41d9ad1cb3ee41a888b5bc8f429bb598bd82e960d4b6eceed9c39461c41a70f2`. Root subsequently observed native chooser opening, CLI import, actual Start/Stop/blank/Swap behavior, and found focused-button priority/Presenting Open inconsistencies. See the Root-owned poc-1-report.md and native receipts, not this implementation report, for those native claims.
