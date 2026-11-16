# Design: presentation timer resume and header timer redesign

## 1. Current behavior (evidence)

| Area | File | Fact |
|---|---|---|
| Timer bookkeeping | `crates/ferrostage-core/src/timer.rs` | `start` zeroes `accumulated`; `pause` freezes; `resume` continues; `reset` zeroes and keeps running state. |
| Stage machine | `crates/ferrostage-core/src/state.rs` `apply_at` | `StartPresentation` → `timer.start(now)` (always zero). `StopPresentation` → `timer.pause(now)`, stage `Ready`. Timer commands only allowed in `Presenting`. |
| Reset boundaries | `state.rs` `load_mapped` / `CloseDocument` | Both rebuild `Self::idle()`, so the timer is zeroed. `commit_document` (hot reload) keeps the timer. |
| Start flow | `crates/ferrostage-ui/src/shell.rs` `start()` → `Transaction { kind: TransactionKind::Start }` → on placement confirm applies `StartPresentation { audience_available: true }` | Start is a two-phase native transaction. |
| Stop paths | `shell.rs` `stop()` (Esc / Stop button), `no_play` placement commit, `return_home` | All end in `StopPresentation`; `return_home` then `CloseDocument`. |
| Header | `crates/ferrostage-ui/src/presenter.rs` (~L760–918) | `identity` (`flex_1`) + `timer` + `actions`; timer sits right of the filename, so its x position depends on title length. |
| Timer face | `presenter/workbench_view.rs` `timer_panel` | Optional warning text, muted label, 28px mono digits, Pause/Reset text buttons while presenting. |
| Bottom strip | `workbench_view.rs` `tools_panel` | `session-navigation` (Start/Stop, start-mode, Prev/Next, progress), blanking, tools, `secondary` (Clear + `document-options` = 布局与导航). |
| Keybinding | `ferrostage-settings` `start_presentation = ["F5"]`, `actions.rs` binds `Start` | |

The audit confirms the gap is narrow: Stop already freezes; only Start's unconditional zeroing loses time.

## 2. Core contract

### 2.1 State

Add to `PresentationState`:

```rust
/// Elapsed time frozen by the last Presenting → Ready stop, when resumable.
resumable: bool,
```

Public read API:

```rust
/// `Some(elapsed)` when the last live session stopped with ≥1 s and can resume.
#[must_use]
pub fn resumable_elapsed(&self, now: Instant) -> Option<Duration>;
```

(`now` is passed for symmetry with `elapsed_at`; while `Ready` the timer is paused so the value is stable.)

### 2.2 Commands

Add `PresentationCommand::ResumePresentation { audience_available: bool }`, name `"resume_presentation"`.

| Command | Allowed when | Effect |
|---|---|---|
| `StartPresentation` | `Ready` | `timer.start(now)` (zero), `resumable = false`, stage `Presenting`. Semantics unchanged; this is "Restart" when resumable. |
| `ResumePresentation` | `Ready` **and** `resumable` | `timer.resume(now)` keeping `accumulated`, `resumable = false`, stage `Presenting`. Otherwise `NotReady { stage }` (or a new `CoreError::NothingToResume` if a distinct error reads better in tests; pick one and keep it typed). `audience_available = false` → `NoAudienceDisplay`, no mutation. |
| `StopPresentation` | `Presenting` | `timer.pause(now)`; `resumable = elapsed ≥ 1 s`; stage `Ready`. |
| `LoadDocument` / `LoadMappedDocument` / `CloseDocument` | unchanged | rebuild idle → `resumable = false`. |
| `CommitDocument` / `ReconfigureDocument` | unchanged | keep `resumable` and timer. |

Resume always leaves the timer **running**, even if it was paused before the stop: "继续演示" means the clock goes again. A presenter who wants it paused can press Pause.

`property_tests.rs` adds the new command to its generator; invariant: `resumable ⇒ stage == Ready`.

### 2.3 Boundaries kept

No GPUI/PDFium types in core. `AudienceProjectionSnapshot` unchanged. No settings change.

## 3. Shell / intent wiring

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StartTimer { Restart, Resume }
```

- `TransactionKind::Start` → `TransactionKind::Start(StartTimer)`. On placement confirm apply `StartPresentation` or `ResumePresentation`. Every existing `matches!(… TransactionKind::Start)` becomes `TransactionKind::Start(_)`.
- `Shell::start(cx)` → `Shell::start(timer: StartTimer, cx)`. Failure / timeout paths are unchanged; a failed resume keeps `resumable` because core was never mutated.
- Intents: keep `Intent::Start` (= Restart / fresh start) and add `Intent::Resume`.
- Keyboard `Start` action (F5) resolves at dispatch: `Resume` if `state.resumable_elapsed(now).is_some()`, else `Restart`. Settings label stays "开始演示"; no new keybinding key.
- Presenter data gains `resumable: Option<String>` (formatted `MM:SS` using the same formatter as the timer, honoring count-up/count-down) for the button sublabel and accessibility label.

## 4. Visual design

### 4.1 Direction

Existing product is the direction (DESIGN.md: warm cream / dark ink, clay primary, surface steps over shadows, soft segmented groups, no gradient/glow, no nonessential motion). Stated read: *a calm instrument panel where slides dominate and chrome steps back*. This task only rebalances the header and the session control within that vocabulary.

- **Visual thesis**: the timer becomes the header's single centered instrument, a soft segmented capsule using the same surface step as the bottom toolbar groups, with large tabular digits and a text-plus-dot state.
- **Content plan (utility)**: orient (clock, filename, stage chip, left) → time status (center) → manage (layout, home/open/settings/displays, right).
- **Interaction thesis**: none beyond the existing 1px press-down; time-dependent motion is banned by DESIGN.md, and the timer changes every second, so anything animated would read as noise.

### 4.2 Header layout

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│ 08:43:54  defense.pdf  ●准备就绪   ┃  ● 已用时  00:00  ┃   布局与导航 ▾ │ 返回首页 打开 PDF 设置 显示器设置 │
│ └───────── left: flex_1, min_w_0 ┘  └── center capsule ┘  └──────── right: flex_1, justify_end ─────────┘ │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

- `presenter-bar` children: `identity` (`flex_1`, `min_w_0`, truncating filename) · `timer` (`flex_shrink_0`) · `actions` wrapper (`flex_1`, `min_w_0`, `justify_end`). Equal flex basis on both sides keeps the capsule centered on the window until one side's min-content exceeds half the free width; then it degrades to "between" without overlap.
- Right group order: `布局与导航 ▾`, a 1px `hairline` divider (24px tall), then the existing stage-specific utilities (Ready: 返回首页 / 打开 PDF / 设置 / 显示器设置; Presenting: 更多 / 显示器设置).
- Narrow (`< 800px`): keep the existing stacked branch; row 1 identity, row 2 centered capsule, row 3 actions (wrap). Start/Stop stays visible in the bottom strip.

### 4.3 Timer capsule (recommended, option B)

```
 Presenting, running            Presenting, paused             Ready, stopped & resumable     Ready, fresh
╭─────────────────────────────╮ ╭─────────────────────────────╮ ╭──────────────────╮         ╭──────────────────╮
│ ● 已用时  12:34 │暂停│重置   │ │ ○ 已暂停  12:34 │继续│重置   │ │ ■ 已停止  12:34  │         │   已用时  00:00  │
╰─────────────────────────────╯ ╰─────────────────────────────╯ ╰──────────────────╯         ╰──────────────────╯
```

| Part | Spec |
|---|---|
| Container | `id presenter-timer`; `p.soft` fill (same as bottom segmented groups), `PANEL_RADIUS` 12, height = `CONTROL` 44, padding-left 16, padding-right 4 (Presenting, buttons inside) or 16 (Ready); no border, no shadow. |
| State mark | 8px leading mark. Running: filled dot `p.focus`. Paused: 8px ring (1.5px stroke) `p.muted`. Stopped (resumable): 8px square `p.muted`. Fresh: no mark (reserve its 8px + 8px gap so digits don't shift between fresh and stopped). Warning overrides the running dot color with `p.warning` / `p.error`. Drawn with `canvas` like `session_icon`, never a font glyph. |
| Label | 12px medium, `p.muted`. Text by state: Running → `已用时` / `剩余` / `超时` (existing keys); Paused → `已暂停`; Stopped → `已停止`; Warning → existing `wb-timer-yellow` / `wb-timer-red` short forms (see 4.6). Label slot has a fixed `min_w` = `ui_text_size(12) * 5` and right-aligns its text toward the digits, so label changes never move the digits. |
| Digits | Mono (`theme::MONO`), weight medium, `typography.size(30.)`, `min_w` sized for the longest signed value (`−000:00` → 7 mono cells), right-aligned. Color: running `p.ink`; warning `p.warning` / `p.error`; paused / stopped / fresh `p.body` (one step quieter than live, still ≥4.5:1 on soft in both themes). |
| Inline actions (Presenting only) | 1px `hairline` divider (20px tall), then Pause/Resume and Reset as quiet text buttons (`ButtonKind::Utility` face, transparent fill at rest, `p.card` hover, 1px press-down), 44px targets, same labels as today (`wb-pause` / `wb-resume` / `wb-reset`). |
| Accessibility | Capsule `aria_label`: "{label} {digits}"; buttons keep existing names and focus order (after the capsule, before right actions). |

Contrast to verify (task check): `p.muted` on `p.soft`, `p.body` on `p.soft`, `p.ink` on `p.soft`, warning/error on `p.soft`, for LIGHT and DARK.

#### Alternatives considered

- **A. Open readout, no capsule**: label above digits stacked, 32px digits, thin 2px progress rail under digits when a target exists. Most glanceable, but the rail introduces a new component and stacked text needs ~48px height, which breaks the 0px-padding Presenting header.
- **C. Large clock, actions elsewhere**: 36–40px digits only; Pause/Reset move into 更多. Cleanest, but hides frequently needed controls in Presenting and contradicts DESIGN.md "Pause/Resume/Reset are header utilities".

B is recommended because it reuses the existing segmented-group surface, fits both header heights, and keeps the timer controls where presenters already look.

### 4.4 Bottom-left split session button

```
Ready, resumable
╭──────────────────────────┬────────────────────╮
│ ▶  继续演示              │ ↺  重新开始        │
│    从 12:34 继续         │    计时归零        │
╰──────────────────────────┴────────────────────╯
   clay primary face           neutral navigation face
```

- Container `id session-start-split`; two activation targets `start-resume` and `start-restart`, joined with a 2px gap; outer corners `RADIUS` 8, inner corners 0 (`rounded_tl/bl` vs `rounded_tr/br`).
- Height 56 (`SESSION_CONTROL`), matching the existing Ready Start. Resume min width = `SESSION_WIDTH` 208 scaled; Restart min width 144 scaled. Total grows by Restart's width; the start-mode trigger follows on the right as today.
- Resume face: `ButtonKind::Primary` (clay, white), 26px play icon, label 14/20 medium + sublabel 12px at 85% opacity of `ON_PRIMARY` (verify ≥4.5:1 on `PRIMARY`; fall back to full `ON_PRIMARY` if not).
- Restart face: `ButtonKind::Navigation` (neutral card face, `p.ink`), 20px drawn counter-clockwise arrow icon (canvas `PathBuilder::stroke`, arc + head) or, if the arc path is unreliable, no icon; label + sublabel `p.muted`.
- Busy: the pressed segment shows `preparing`; both segments disabled, geometry preserved. Disabled start reason (`start-reason`) shows below, as today.
- Fresh Ready: unchanged single Start button. Presenting: unchanged Stop.
- Accessibility labels: "继续演示，从 12:34 继续计时" / "重新开始演示，计时归零" (localized, `{time}` placeholder).
- Focus order: Resume → Restart → start-mode trigger → Previous …

### 4.5 Layout & navigation trigger in header

- Moves out of `tools_panel` `secondary`; `secondary` keeps only Clear (Ready).
- Trigger keeps `document-options` id, `ButtonKind::Navigation` face and chevron (chevron points down when closed, up when open, mirroring header placement).
- Menu anchoring changes from "upward, 8px above trigger" to "downward, 8px below trigger", right-aligned to the trigger, 320px wide, 12px viewport margin. Rows, radio/checkbox semantics, keyboard model, Escape/outside dismissal and focus restoration are unchanged.
- Enabled whenever `!busy` and a document is loaded (Ready and Presenting), as today.

### 4.6 Copy (new keys)

| Key | zh-CN | en-US |
|---|---|---|
| `resume` | 继续演示 | Resume presentation |
| `resume-detail` | 从 {time} 继续 | Continue from {time} |
| `resume-label` | 继续演示，从 {time} 继续计时 | Resume presentation, timer continues from {time} |
| `restart` | 重新开始 | Restart |
| `restart-detail` | 计时归零 | Timer resets to 00:00 |
| `restart-label` | 重新开始演示，计时归零 | Restart presentation, timer resets to zero |
| `wb-timer-paused` | 已暂停 | Paused |
| `wb-timer-stopped` | 已停止 | Stopped |
| `wb-timer-low` | 时间不多 | Running low |
| `wb-timer-limit` | 到时 | Time up |

`wb-timer-yellow` / `wb-timer-red` long forms remain as the capsule's accessible description; the short `low`/`limit` forms fit the fixed label slot. `wb-elapsed` / `wb-remaining` / `wb-overtime` unchanged.

## 5. DESIGN.md changes (edit first)

- Layout §08: replace "A labeled 28px monospace presentation readout remains before utility actions … It has no separate badge background" with the centered soft capsule contract; header becomes identity / centered timer / actions. Move "Layout" from "Presenting … Page browser and Layout stay direct" bottom strip to header actions.
- Components `button-secondary / tool-button`: Layout & navigation is a header menu trigger; describe the split Resume/Restart session control and its faces.
- Components `status / timer`: capsule, state mark shapes, fixed label slot, 30px digits, stopped/paused states.
- Components `layout-navigation-menu`: downward, right-aligned under its header trigger.
- typography `timer.fontSize`: 28px → 30px.

## 6. Risks and rollback

- **Header centering vs. long right group (Ready, 5 actions + layout)**: at 800–1279 the right group may exceed half width; the flex rule degrades without overlap. If it still collides, Ready can fold 打开 PDF / 设置 into the existing utility menu at < 1280 (not planned; only if native check fails).
- **TransactionKind churn**: tests in `shell.rs` / `reload/tests.rs` / `document_load.rs` / `media_playback.rs` construct `StartPresentation`; core command shape is unchanged, so only shell-side `TransactionKind::Start` matches change.
- **Rollback**: core change is additive (new command + flag); UI changes are view-local. Reverting the UI commit leaves a harmless unused command.

## 7. User decisions (2026-10-20)

- Timer style: option B (soft capsule) confirmed.
- F5 after an interrupted session: Resume. Only the Restart button zeroes the timer.
