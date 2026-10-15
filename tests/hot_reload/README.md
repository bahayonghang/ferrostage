# Hot reload

Debounce and the `%%EOF` check live in `ferrostage-platform` (`HOT_RELOAD_DEBOUNCE`, `has_pdf_eof`). `SHADOW_TMP_SUFFIX` is still defined in that crate. It is not a user-PDF write path. Task 10 removes the constant. FerroStage opens the user's PDF read-only. An end-to-end fixture test will go in this directory once the watcher loop exists.
