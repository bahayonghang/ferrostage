# Presenter click-next, version 1.2.1, and About changelog

## Goal

While a document is open, a completed left click on the presenter current slide, next slide, or notes area advances one existing Next step. Settings > About gains a Changelog button that shows the local version history. The workspace version becomes 1.2.1.

## Background

- Presenter panes are `current-pane`, `next-pane`, and `notes-panel` in both workbench layouts (`crates/ferrostage-ui/src/presenter/workbench_view.rs:1288-1478`). Only `picture(Surface::Current, true)` handles pointer input, and that path is media and drawing (`workbench_view.rs:322-399`). Next and Notes pictures are not clickable.
- The Next button dispatches `Intent::Next` → `Command::NextStep` when no settings modal, contextual surface, or pending transaction is open and the stage is not Idle (`crates/ferrostage-ui/src/shell.rs:801-816`). Audience-window clicks use a separate policy and the Displays toggle (`shell.rs:1243-1264`). `audience_click_does_not_bind_the_presenter_slide_preview` (`shell.rs:4902`) currently forbids presenter-preview clicks.
- About already shows version, author, build time, license, and repository (`settings_view.rs:378-457`). `about::VERSION` is `CARGO_PKG_VERSION` (`about.rs:4`). Workspace version is 1.2.0 (`Cargo.toml:36`), and `preferences_tests.rs:964` asserts that string.
- Shipped user-facing deltas are the version-bump commits `746b18b` (1.0.0), `f6f5c48` (1.1.0), `e262705` (1.1.1), `487653e` (1.2.0), plus the post-1.2.0 notes-first layout, resumable timer, and document-warning flag. Details are in `research/presenter-click-and-about.md`.

## Requirements

| ID | Requirement |
| --- | --- |
| R1 | A completed left click inside `current-pane`, `next-pane`, or the notes content (including captions, letterbox, the rendering placeholder, and the empty-notes line) dispatches exactly one `Intent::Next`. Mouse-down alone, a non-left button, or a release outside the clicked pane does not advance. Two completed clicks may advance twice. Both Standard and Notes-first layouts, including the stacked narrow layout, use the same hit targets. |
| R2 | Presenter clicks follow the existing Next guards and core step rules: no advance while Settings, a contextual surface, or a pending transaction is open, or while Idle; Ready and Presenting advance; one Beamer step; notes pages stay off the audience content map; the last step stays put. Blank mode does not block this click. The Displays toggle `audience_click_to_advance` is unchanged and does not gate these panes. |
| R3 | On the current slide only, Navigate-tool clicks advance except when the point hits a media slot, which keeps the existing media action and does not also advance. Pen, highlighter, eraser, and laser clicks on the current slide keep today's drawing and pointer behavior and do not advance. Next-slide and notes clicks still advance while a drawing tool is selected. Splitter drags, toolbar buttons, the page grid, and outline rows keep their current actions. Wheel and scrollbar dragging in the notes pane scroll and do not advance. |
| R4 | Settings > About shows one button, `about-changelog`, labeled 更新日志 / Changelog. Activating it reveals the changelog in the About pane and the same button hides it. The five existing About rows stay. The flag is panel UI state: it is not written to settings, it clears when the category changes or Settings closes, and it supports the existing pointer and keyboard activation path. |
| R5 | The changelog is offline, newest first, and localized in zh-CN and en-US. Entries and order are exactly 1.2.1, 1.2.0, 1.1.1, 1.1.0, 1.0.0, with the copy in the table below. No network request and no second product-version constant. |
| R6 | `[workspace.package].version` is 1.2.1. Every inheriting FerroStage crate and the matching `Cargo.lock` package records use that version. Unrelated dependency versions stay. About, the 1.2.1 changelog heading, and `CARGO_PKG_VERSION` agree. |

### Changelog copy

| Version | zh-CN | en-US |
| --- | --- | --- |
| 1.2.1 | 演讲者视图中，单击当前页、下一页或讲稿区域可前进一页。设置的关于页可以打开更新日志。新增讲稿优先的双屏布局。中途结束演示后可以继续计时，或重新开始。文档警告改为可关闭的浮层小旗。 | In the presenter, click the current slide, the next slide, or the notes area to advance one step. Settings > About can open this changelog. Added the notes-first dual-screen layout. After stopping mid-presentation, the talk timer can resume or restart. Document warnings are a closable flag. |
| 1.2.0 | 单屏或复制显示时，可以在当前屏幕全屏放映，不显示讲稿。开始演示旁可以选择双屏或单屏。设置新增关于页。 | On a single screen or mirrored display, playback can fill the current screen without notes. The start control can choose dual-screen or single-screen. Settings gains an About page. |
| 1.1.1 | 安装程序支持 Windows 10 版本 1903（内部版本 18362）及更高版本。 | Setup supports Windows 10 version 1903 (build 18362) and later. |
| 1.1.0 | 支持全屏备注、目录跳转、页面转场和定时放映。首页显示最近文件的打开时间，并提供 Windows 安装程序。 | Adds fullscreen notes, outline jumps, page transitions, and timed playback. Home shows when recent files were opened, and Windows Setup is available. |
| 1.0.0 | 首次发布：双窗口 PDF / Beamer 演讲、演讲者备注、批注，以及外部 PDF 热重载。 | First release: dual-window PDF and Beamer presentation, speaker notes, annotations, and external PDF hot reload. |

Each version is one heading plus one paragraph. The strings above are the whole paragraph, not a list of extra sentences.

## Acceptance criteria

| ID | Observable result | Requirements |
| --- | --- | --- |
| AC1 | With Navigate selected, a completed left click on the current slide, the next slide, or the notes area advances exactly one step in both workbench layouts. Mouse-down alone, a non-left button, and a release outside the pane leave the page unchanged. The audience-window toggle is not consulted. | R1, R2 |
| AC2 | Pen, highlighter, eraser, and laser clicks on the current slide do not change the page. A media-slot click on the current slide keeps the media action and does not advance. Splitter, toolbar, grid, and outline behavior is unchanged. Scrolling the notes pane does not advance. | R3 |
| AC3 | Settings open, a contextual surface, a pending transaction, and Idle ignore the new clicks. Ready and Presenting accept them. An overlay fixture advances one content step. The final content step does not wrap. Blank mode still accepts the click, matching the Next button. | R2 |
| AC4 | About shows the changelog button in both locales. Activating it shows every version heading and the copy table, newest first, without opening a URL. Activating it again, changing category, or closing Settings hides the log. The five existing About rows and the repository link stay. | R4, R5 |
| AC5 | Workspace version, lockfile FerroStage packages, `about::VERSION`, and the first changelog heading are 1.2.1. No unrelated dependency version changes. | R6 |

## Out of scope

Audience-window click policy and its Displays toggle. Right-click or wheel navigation. Page-grid and outline jump behavior. A new persisted setting. `just release`, install, tag, or a hosted GitHub release. README feature-list edits. The packaging-script encoding fix as changelog copy.
