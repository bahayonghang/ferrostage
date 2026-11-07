# Return-position verification — 2026-11-07

Status: complete for the task's application-level scope. Source review and regression detail: [final-review.md](final-review.md). Independent visual verdict: [ship](visual-review.md).

## Product behavior and tests

The first successful changed-page direct jump retains the exact generation and physical page. Further jumps, sequential navigation and authored playback preserve that origin. Ready requires a successful current composite paint with the current presenter/request/private attempt capability. Presenting requires accepted current public settlement; covered readiness checks the exact underlying public target while retaining the cover. Late or failed results cannot consume the anchor.

Explicit same-page Return re-arms only the failed Current/Audience raster and texture attempt. Ordinary redraw stays latched. Failed public transition retries keep the committed outgoing fallback and diagnostic until actual recovery. There is no navigation history stack, extra public serial or audience-private control path.

Final source checks, after the two-line footer wrap fix (byte-identical log copies in `checks/gate-logs.zip`; member hashes in `checks/manifest.json`):

- Focused Return suite: 10 passed; complete GPUI suite: **269 passed**.
- Settings suite: **55 passed**; strict test-support Clippy passed.
- `just ci`: passed format, strict workspace Clippy, **251 workspace tests**, cargo deny (`runs/return-review/ci-footer-final.log`).
- `just check-shell`: passed native GPUI/PDFium build, strict product Clippy and **269 UI tests** (`runs/return-review/check-shell-footer-final.log`). It ran after CI.
- Directly affected PDFium reload tests: **9 top-level tests passed**, including isolated subprocess cases (`runs/return-review/reload-pdfium.log`). Reused after the layout-only follow-up.
- All-features cargo deny passed (`runs/return-review/deny-product-final.log`); no dependency change. PDFium SHA256 `79d4676b656cfb1abcea88f9ade3b4b0826c5200382db5f4ec72a636c598c118`.

The GPUI regressions cover real numeric/thumbnail/outline submission, same/error/cancel/preview non-capture, explicit clear, focus/modal isolation, default/rebound bindings, actual Ready paint, stale window/request/attempt rejection, public pending/settled, covered readiness, failed raster/upload retry and fallback retention, interruption, Stop/Resume/reset, Home/new-open/reload/crop lifecycle and subsequent Right/Backspace navigation. Settings tests cover absent v1 defaults, empty disable, invalid/conflicting values, future read-only policy and unknown-key roundtrip.

## Native compiled-Beamer acceptance

Isolated configuration and owned test PDF live under `runs/return-native`; no user configuration or PDF was changed. Original `fixtures/beamer-timing/overlays-timing.pdf` SHA256 `8e40489ef4fe594585b19e64c20fd22e958aeb94b801e4b5a7ad7016c59dedd9`. This is a compiled Beamer overlay/notes fixture; content-step numbers differ from logical slide numbers. Origin is logical 1/6, step 2/2, physical 3/33; B is physical 6, C physical 12.

The first native run used executable SHA256 `08138daef54be447789ecb8eebeb181e09ca737cfb49d937b8c54e068e1701ee`. It demonstrated:

- Ready A-step2 → numeric B → numeric C → default Alt+Left → exact A-step2, with no audience creation. See `native/ready-*.png` and `ready-audience-count.json`.
- Presenting A-step2 → B → C → A-step2, with public Settled and anchor consumption. Paused talk timer **44:12**, paused automatic playback and laser selection remained unchanged. See `native/public-*.png` and corresponding UIA.
- Black cover → jump → Return to A-step2 consumed the anchor from exact underlying readiness while keeping black. `native/covered-check.json` records no visible phase before unblank. Actual unblanked audience screenshot shows only public A/B and the selected laser, without private note markers or presenter status.
- Stop preserved the anchor and resumable timer. That run exposed ordinary-window footer clipping; `stopped-footer-overflow-before.png` is defect evidence, not final acceptance.

The final executable SHA256 is **`09fa7c5a1297e6d2b487f892d3ff63fe7562b2c7c7987c1c8a9cef0d89c4fd6b`**. Only navigation wrapping/max-width and its regression changed between native runs. Final-source native checks demonstrated:

- Chinese default 16px UI, ordinary **1100×760** client: stopped/resumable Resume, Restart, Return and Clear all visible; Clear wraps within the window. `native/stopped-footer-fixed.png`.
- Real Settings → Shortcuts row removal/recording persisted **Ctrl+R**, retaining unknown `vendor_return_test = "keep"`. Old Alt+Left left physical 12 and its anchor unchanged; Ctrl+R returned Ready physical 3 and consumed the anchor after paint. `settings-rebound-ctrl-r.png`, `settings-rebind-readback.json`, `old-binding-inert-uia.txt`, `ready-rebound-returned-step-2.png`.
- New Ready jump → F5 Resume retained the exact A-step2 anchor at C. Ctrl+R showed the pending public target first, then accepted A-step2 and removed the controls. `resumed-anchor-preserved.png`, `resumed-public-returned-step-2.png`. Resume runs the timer according to its existing semantics; this does not claim pause preservation across Stop/Resume.
- Replacing only the owned test copy with malformed bytes kept the old active page and anchor. Restoring valid bytes plus one newline committed generation 2 and cleared the anchor. `reload-anchor-before.png`, `invalid-replacement-anchor-retained.png`, `successful-reload-anchor-cleared.png`, `native/traces.zip` (member `final-stdout.log`). The copy was restored byte-for-byte to the original after the process closed.

## Limits and unresolved observation

Sky-injected **Alt+R did not record** in two attempts; Ctrl+R and Alt+Left recorded successfully. Read-only tracing found no Return-specific filter: Settings accepts canonical `alt-r`, Windows forwards WM_SYSKEYDOWN, and its backend ignores VK_PACKET. No raw Windows-message capture established which input path failed. The cause remains unlocated; this is not claimed as a proven tool or product defect, and Alt-letter native recording is not marked passed. Default Alt+Left and configurable Ctrl+R were verified.

Native malformed-file rejection is not a raster/upload-failure injection; those failures and same-page retry were exercised through GPUI/cache regressions. No physical projector, cable hotplug, extra DPI/window matrix or other-platform acceptance was performed. No dependency source, license waiver or user PDF write was introduced.
