# Native V10 checkpoint — 2026-10-15

Independent source review allowed the unchanged V10 freeze to enter native acceptance. Root started only one real application using the prior UI-saved C-presenter/B-audience configuration. `native/v10-main.launch.json` binds PID128376,609 inputs aggregate `2ee35522a9f885cf8c62ac5007e7feecc7d5c32f28949fc2f7b65e4052383a67`, executable `783d5769785029f11fa73f631720b47f7280e5124b30801d1930ee213dee37af`, config SHA `fd9004c7b16f967a42e90bc1a6302d827bc24e0e674ff4fb47de78543b5f9d08`, PDFium and the repository fixture. No frozen input changed during this run.

## F10 — requested resize does not produce the requested Windows client (FAIL)

`native/v10-main.stdout.log` records:

-04:54:51: actual C/display65735 at scale1.5 initially has client1202.6666x912; fitted requested client is1002.6666x760.
- The new existing-GPUI resize call changes actual client to1000x754, leaving a2.6666x6 discrepancy. The client is no longer oversized, but does not meet the strict requested-size contract.
-04:54:54: `RecoverPresenter` times out and removes the pending GUI. Actual usable bounds are `[1706.6666,-761.3333,1066.6666,1658.6666]`; final client is `[1740,-297.33334,1000,754]`. Do not relax the2px tolerance to call this successful.

Sky's actual inventory shows only the executable's console, no Presenter. Root did not automate that terminal. The default sandbox process inventory could not see PID128376; an escalated narrowly scoped process lookup verified its `ferrostage` name and exact task executable path, then stopped that owned process. Runner returned exit-1 at04:55:46Z, and final Sky inventory is empty. This is explicit failed-run cleanup, not a normal-close PASS. Receipts: `v10-main-timeout-inventory.json`, `v10-main-stopped-inventory.json`, `v10-main.exit.json`.

## Continuation

V10 source review/automated gates remain their accurately bounded PASS receipts; actual native placement is FAIL. The TestPlatform resize behavior did not expose this Windows client/frame discrepancy. Root released source only after confirmed cleanup, then dispatched scoped V11 implement before a new independent check. Preserve the actual-size/containment gate, existing bounded rollback, source identity and all old evidence; correct resize semantics rather than weakening acceptance.

No remaining new-panel native matrix was executed on this failed run. Gate B remains INCOMPLETE;08–11 remain dependency-gated. Physical operations/true single or zero-screen cases remain NOT_RUN. All changes stay uncommitted.
