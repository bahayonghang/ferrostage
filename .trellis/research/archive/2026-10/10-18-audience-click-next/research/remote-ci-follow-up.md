# PR CI follow-up

PR #2 / run 39128516258 / head a857213: default Linux CI, actual Linux GPUI+AT-SPI1141+portal3+oo7 original golden, and Windows media jobs passed. The PDFium job passed native fixtures, full-feature deny, product build/lint and143 GPUI tests, then failed only in its final missing-runtime negative checker.

The native test correctly failed with Pdf(LibraryUnavailable). PowerShell `Select-String native-missing.log -SimpleMatch 'LibraryUnavailable'` binds its first positional argument to Pattern and the second to Path, so it tried to open a file named LibraryUnavailable. Use explicit `-LiteralPath native-missing.log` while retaining the exit-code check and expected-error assertion. No product/test assertion is weakened.

Executed both exact workflow scripts locally with the same missing runtime and PowerShell Stop policy: old script RED exit1 with wrong-path error; corrected script GREEN exit0, with the native test still failing as required with LibraryUnavailable. Sibling scan found only this occurrence. actionlint passed. A new PR run must pass before merge; main manual dispatch remains pending until after merge.
