# Beamer notes

Not implemented yet.

When a Beamer PDF uses the "show notes on second screen" layout, each physical page is a wide frame: slides on the left, notes on the right. The audience crop is `NormalizedRect::LEFT_HALF`. The presenter reads the notes text from the document model, not from the audience snapshot.

Overlay steps (`\pause`, `\only`) will become several steps inside one `LogicalSlide`. Until that parser exists, `NextLogicalSlide` and `NextStep` both move one `PhysicalPage`.
