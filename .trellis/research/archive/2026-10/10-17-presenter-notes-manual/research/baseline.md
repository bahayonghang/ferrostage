# Baseline evidence

- Source HEAD: bf9e87cd0c2cd96df41457dae7dd7d0c09b10a48.
- Existing debug executable SHA256: 013FB4DE31189116C36C4766E02C051AFB601CEE8E1A76DCDA84B945305C9064. Existing artifact, not rebuilt in this baseline step.
- PDFium SHA256: 79D4676B656CFB1ABCEA88F9ADE3B4B0826C5200382DB5F4EC72A636C598C118.
- perf-100.pdf SHA256: 395D8BB26AD5F8F7CA5245901ABDCB8DE1FCCD2BE5FF7E30482E54F776526CB6.
- Task-owned copied settings used for shell launches, with dark theme and one-minute target.
- Sandboxed launch: process stayed alive with NoMonitors / IncompleteTopology (device names present, geometry empty); no targetable window. Process deliberately stopped, not a clean native close pass.
- Escalated launch: full monitor enumeration succeeded and presenter placement confirmed at 1100x760 logical client size. Computer Use list_windows did not expose this process/window. Process deliberately stopped after bounded observation, not a clean-close pass. See native/baseline-desktop-stdout.log and launch/exit receipts.
- Source investigation by implementation agent found thumbnail on_hover dispatching Preview, explaining unsolicited preview changes when moving the pointer across the persistent strip. Timer-driven Next was not found. Final behavior must remove hover navigation and prove time cannot advance pages.

Native visual/elapsed-time baseline is not established by the above launch logs. User-supplied screenshot remains the baseline visual reference.
