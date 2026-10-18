# Baseline session receipt

Visible baseline ran with task-local baseline.toml, pre-existing executable SHA256 297814B67C8DB87F8C3F89065E14CB910B6C52940E854955F52453530159C07E. This binary is a visual reference; correspondence to current source was not independently proven. Saved baseline-zh-dark.jpg is 1102x791 capture (standard nominal 1100x760 client).

Two earlier sandbox/hidden launches did not yield a visible usable window. Task-owned hidden PID 48360 was stopped before rebuilding. The successful interactive launch was PID 52548; its returned window ID 2036064 matched process MainWindowHandle. Its app identifier inherited Codex's AppUserModelID; selection used FerroStage title and verified process/window identity, never the actual Codex window.

The successful visible window was activated, captured, then closed via Alt+F4. Its supervising process reported exit 0. Baseline stdout/stderr capture collided with files held by the earlier hidden launch, so those logs do not establish clean-close diagnostics. The shared baseline-exit.json was later overwritten by hidden-process termination and is not the visible launch's exit receipt. Final native runs must use unique log names and source-bound executable hashes.
