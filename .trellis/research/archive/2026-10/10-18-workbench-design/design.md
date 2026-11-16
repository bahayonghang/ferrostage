# Workbench hierarchy and control design

## Evidence and structure

Parent research F1–F3 locates `presenter.rs:362` command bar, `:879` tools and `:882` standalone Start footer; `workbench_view.rs:395` lays out every tool in a flat wrapping sequence. Root DESIGN explicitly prescribes pill drawing tools, so update that contract first rather than silently violating it.

Proposed normal-window structure:

```text
filename  [quiet Ready / Presenting]           countdown  quiet clock    Home Open Settings Displays
current slide                               current notes
                                            next slide
[large play + Start] [Previous] [Next] [page / total + step] | blanking | drawing tools | grid/document
```

Retain one main split and the notes-first right column. No additional permanent sidebar, dashboard, document cards or duplicate filename. Filename truncation/full-name tooltip remains as needed; critical counters never tail-truncate with ellipsis. Top utilities use quieter faces and coherent spacing. Use existing palette; countdown retains semantic warning colors, explicit warning text and mono role. Status is a small passive label with text/optional dot; it is not a click target with a button outline.

## Action hierarchy

One session/tool composition owns Start/Stop and navigation instead of the two independent render branches. First group: 56px-high session face with an approximately 26px play/stop vector icon plus label, aligned in the same strip with >=44px navigation targets. No huge icon-only control. Start uses clay fill; Stop uses existing semantic danger styling. Keep IDs `start`, `stop`, `previous`, `next` and original Intents. Disabled Start exposes its reason locally; preparing state blocks reentry without changing geometry or timers. Handle document-open states with empty workbench steps safely so removing the old footer cannot erase the Start/Stop route.

Progress moves next to navigation rather than fighting filename in the header: localized compact logical page / total and step / steps when multiple steps exist. Full logical/step/physical information is in its accessible label and hover/focus detail. It remains recognizably clickable with a page/jump affordance and opens the existing Jump intent. Typing does not navigate; confirmation still does. Do not change source page/step indices or invent shortcuts.

Second group: black/white blank controls with small swatches, label and current-state marker derived from actual blank state. Third group: Navigate/Laser/Pen/Highlighter/Eraser use a shared segmented/quiet tool vocabulary, selected surface plus check/label, no five separate pill badges. Clear is secondary semantic danger, separated from mode selection. Grid and Document remain secondary labeled utilities. Use consistent role radius/border rules and retain keyboard/focus/accessibility routing. Existing neutral navigation can stay stronger than quiet tools but no bright white slab is needed merely to indicate a button.

Use existing geometric icon facilities if available; otherwise add minimal local play/stop/chevron vector icons, not a new icon library or emoji font. Icons are theme-colored; PDF/media/notes pixels are untouched. Preserve immediate existing press feedback without ornamental motion or keyboard animation. Tools may wrap as existing behavior permits, but at the normal target window the Start/Previous/Next group stays together. Do not shrink target size to force every secondary tool into one row.

## Data and ownership

Presenter renders existing snapshot/workbench state and dispatches existing Intents. Add only view-facing compact progress/state values if needed; core remains authority. Use typography resolver from the preceding child for all interface labels, with timer mono preserved. Do not read Shell synchronously in render/event code where the existing snapshot rule prohibits it.

Owned targets: root DESIGN workbench/controls contract, `presenter.rs`, `presenter/workbench_view.rs`, necessary semantic tokens/local icon assets, both locale files and existing shell/workbench regressions. Do not revert the preceding settings child. The display child later owns allocation data and shared display body. Child rollback is unified header/strip/style/locales/tests plus matching DESIGN, not a change to document/core behavior.
