# Notes-free preview verification

## Result

The notes-free workbench uses Current and Next only. Layout classification uses the full document mapping and pending crop review, not whether a note texture has arrived. Mixed independent-note documents keep their Notes pane when the current page has no notes. Existing Standard and NotesFirst saved proportions remain separate from the session-only preview split.

## Automated checks

Final independent review fixed disabled-row keyboard traversal/activation and a narrow NotesFirst reset that initially targeted the hidden layout. Both have real GPUI input regressions. Final `just check-shell` passed native product build, strict product Clippy and **250 GPUI tests**. Final `just ci` passed format, strict workspace Clippy, **249 workspace tests** and cargo deny. See [final review](final-review.md). No source changed after these gates.

Regression coverage includes whole-document/pending-crop classification, mixed empty/loading/error states, terminal Next placeholder, 800 logical-pixel breakpoint and minimum Next width, real divider/reset input, new-open lifetime and notes settings isolation. These are behavior/layout assertions, not claims about physical projection hardware.

## Windows native observations

Used isolated test configurations and normal presenter controls through Windows Computer Use. Input fixtures are the repository's synthetic `ordinary.pdf`, `mixed.pdf` (mixed geometry, not independent notes), and `independent.pdf` (partially paired private note pages). SHA256 values before/after are unchanged.

- [Ordinary document](native/ordinary.png): Current and Next occupy approximately 70:30 of the available width, no Notes pane; PDF content fits inside each preview.
- [Dragged split](native/dragged.png) and [restored split](native/restored.png): actual divider input changed the session ratio and the menu restored defaults. The saved Standard/NotesFirst ratios remained 0.45/0.65 and 0.55/0.62.
- [Last step](native/last-step.png): Next stays in place with the localized end message.
- [Independent notes](native/independent-notes.png) followed by [a content page without notes](native/independent-empty.png): the same Notes pane bounds remain, with a no-notes message on the latter page.
- Final-build [narrow NotesFirst before reset](native/final-narrow-before.png) and [after reset](native/final-narrow-restored.png): the visible Standard layout is restored to 0.45/0.65; saved NotesFirst values **0.59/0.71 remain unchanged**, asserted by reading the isolated settings file. Menu focus returns to its trigger; page and stopped timer remain unchanged.

The initial layout observations used executable SHA256 `cc56fae36bfda23b6bc2aa3933bc64a12cbeca3fbc74dfdd3166049945f4d6c5` before the two reviewer menu fixes. The targeted narrow-reset retest used the final checked executable `491334d74bb603457f24cd8ec824c61146f444e4b4cc2e9046f5f74987f3a1dc`. [Native data and fixture hashes](native/native-check.json). Initial hidden launches were unsuitable for interactive capture; visible isolated instances were used and all test processes were closed. Two attempted builds hit the running test executable's Windows file lock; final gates were rerun after closing it.

## Design review

The existing DESIGN.md and theme token layer are retained. The design-atlas scanner returned Floor 0 FAIL, House 0 FAIL, Reflex 0, no waivers; **it scanned zero files because it does not parse Rust**. This is not source coverage. [Scanner receipt](design-scan.json). Judged checks: task-specific composition/real content/alignment/focus pass; marketing sections, web forms and mobile layout are inapplicable to this desktop change. Independent screenshot review is recorded separately.

Independent visual review: **ship**. Eight native screenshots and the existing DESIGN.md/source were reviewed in a fresh agent context; no task-local visual issues were found.
