use super::*;
use crate::{
    page_browser::PageBrowser,
    workbench::{BrowserEntry, PageNumberMode, Surface},
};

pub(super) struct BrowserData {
    pub browser: PageBrowser,
    pub captions: Vec<String>,
    pub entries: Vec<BrowserEntry>,
    pub mode: PageNumberMode,
    pub target_context: String,
    pub failed_thumbnails: Vec<usize>,
}
impl BrowserData {
    pub fn from_shell(shell: &Shell) -> Option<Self> {
        let browser = shell.browser()?.clone();
        let mode = shell.workbench.page_number_mode;
        let entries = shell.browser_entries();
        let captions = entries
            .iter()
            .map(|entry| {
                shell.locale.format(
                    if entry.content_number.is_none() {
                        "pb-caption-notes"
                    } else if mode == PageNumberMode::Content {
                        "pb-caption-content"
                    } else {
                        "pb-caption-pdf"
                    },
                    &[
                        ("page", (entry.page.0 + 1).to_string()),
                        ("content", entry.content_number.unwrap_or(0).to_string()),
                    ],
                )
            })
            .collect::<Vec<_>>();
        let target_context = browser
            .target
            .and_then(|target| entries.iter().position(|entry| entry.page == target))
            .map(|index| captions[index].clone())
            .unwrap_or_default();
        Some(Self {
            browser,
            entries,
            mode,
            captions,
            target_context,
            failed_thumbnails: shell.workbench.failed_thumbnails(),
        })
    }
}

impl Presenter {
    /// Shared contextual host: opaque child above an undimmed, input-occluding backdrop.
    /// The layout-menu child supplies its own content and measured trigger bounds.
    pub(super) fn contextual_host(
        &self,
        content: gpui::AnyElement,
        anchor: gpui::Bounds<gpui::Pixels>,
        alignment: gpui::Anchor,
    ) -> gpui::AnyElement {
        let target = self.shell.clone();
        div()
            .id("contextual-host")
            .debug_selector(|| "contextual-host".into())
            .absolute()
            .inset_0()
            .track_focus(&self.contextual_focus)
            .child(
                div()
                    .id("contextual-backdrop")
                    .debug_selector(|| "contextual-backdrop".into())
                    .absolute()
                    .inset_0()
                    .occlude()
                    .on_mouse_down(gpui::MouseButton::Left, move |_, _, cx| {
                        actions::dispatch(&target, Intent::CloseContextual, cx);
                        cx.stop_propagation();
                    })
                    .on_mouse_up(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_scroll_wheel(|_, _, cx| cx.stop_propagation()),
            )
            .child(
                gpui::anchored()
                    .anchor(alignment)
                    .position(gpui::point(
                        if alignment == gpui::Anchor::BottomRight {
                            anchor.right()
                        } else {
                            anchor.left()
                        },
                        anchor.top() - px(8.),
                    ))
                    .snap_to_window_with_margin(px(12.))
                    .child(content),
            )
            .into_any_element()
    }
    pub(super) fn page_browser_panel(
        &self,
        p: Palette,
        window: &Window,
        cx: &Context<Self>,
    ) -> gpui::AnyElement {
        let Some(data) = &self.data.browser else {
            return div().into_any_element();
        };
        let wb = &self.data.workbench;
        let columns = wb.settings.grid_columns as usize;
        let capacity = wb.settings.grid_visible_pages as usize;
        let rows = capacity.div_ceil(columns);
        let width = 880f32
            .min(f32::from(window.viewport_size().width) - 24.)
            .max(240.);
        let viewport_height = f32::from(window.viewport_size().height);
        let anchor_top = f32::from(self.contextual_anchor.get().top());
        let available_height = if anchor_top > 20. {
            anchor_top - 20.
        } else {
            viewport_height - 24.
        };
        let control_height = theme::CONTROL + (theme::FOCUS_WIDTH + theme::FOCUS_GAP) * 2.;
        let header_height = control_height.max(self.ui_text_size(32.) * 1.5);
        let mut chrome_height =
            header_height + control_height * 4. + self.ui_text_size(20.) + 6. * 8. + 34.;
        if !wb.writable {
            chrome_height += self.ui_text_size(12.) * 1.5 + 8.;
        }
        if self.data.dirty {
            chrome_height += control_height + 8.;
        }
        // Account for caption line/padding, inner boundary and shared focus frame.
        let tile_chrome = self.ui_text_size(12.) * 1.75 + 14.;
        let fitted_rows = rows.min(4) as f32;
        let row_gaps = (fitted_rows - 1.) * 8.;
        let preferred_height = (viewport_height * 0.72)
            .max(chrome_height + fitted_rows * (38. + tile_chrome) + row_gaps);
        let height = preferred_height.min(available_height).max(240.);
        let image_height =
            ((height - chrome_height - row_gaps) / fitted_rows - tile_chrome).max(30.);
        let enabled = !self.data.busy;
        let l = self.locale;
        let mut density = div().flex().items_center().flex_wrap().gap(px(4.));
        let rectangular = crate::page_browser::rectangular_rows(
            wb.settings.grid_columns,
            wb.settings.grid_visible_pages,
        );
        for column in [true, false] {
            density = density.child(
                div()
                    .text_size(px(self.ui_text_size(14.)))
                    .child(l.text(if column { "pb-columns" } else { "pb-rows" })),
            );
            for value in 1..=4 {
                let selected = if column {
                    wb.settings.grid_columns == value
                } else {
                    rectangular == Some(value)
                };
                density = density.child(self.button(
                    format!("page-{}-{value}", if column { "columns" } else { "rows" }),
                    value.to_string(),
                    enabled && wb.writable,
                    selected,
                    Intent::GridDensity { column, value },
                    p,
                ));
            }
            if column {
                density = density.child("×");
            }
        }
        let summary = if rectangular.is_some() {
            l.format("pb-capacity", &[("count", capacity.to_string())])
        } else {
            l.format(
                "pb-custom",
                &[
                    ("columns", columns.to_string()),
                    ("count", capacity.to_string()),
                ],
            )
        };
        density = density.child(
            div()
                .text_size(px(self.ui_text_size(12.)))
                .text_color(rgb(p.muted))
                .child(summary),
        );
        let mut grid = div()
            .id("thumbnail-grid")
            .debug_selector(|| "thumbnail-grid".into())
            .flex()
            .flex_col()
            .gap(px(8.));
        let end = (wb.grid_start + capacity).min(data.entries.len());
        for first in (wb.grid_start..end).step_by(columns) {
            let mut row = div().flex().gap(px(8.));
            for index in first..(first + columns).min(end) {
                let entry = data.entries[index];
                let page = entry.page;
                let selectable = entry.content_number.is_some();
                let current = page == wb.overlay.current_page;
                let target = data.browser.target == Some(page);
                let caption = &data.captions[index];
                let label = if current {
                    format!("{caption} · {}", l.text("pb-current"))
                } else {
                    caption.clone()
                };
                let tile = self
                    .activation_target(
                        format!("thumbnail-{index}"),
                        label,
                        enabled && selectable,
                        Intent::BrowserJump {
                            id: data.browser.id,
                            page,
                        },
                        p,
                    )
                    .aria_selected(target)
                    .track_focus(
                        &self.browser_tile_focus[index - wb.grid_start]
                            .clone()
                            .tab_stop(enabled && selectable),
                    )
                    .anchor_scroll(Some(
                        self.browser_tile_anchors[index - wb.grid_start].clone(),
                    ))
                    .w(px(
                        (width - 34. - (columns as f32 - 1.) * 8.) / columns as f32
                    ))
                    .flex_shrink_0()
                    .child(
                        div()
                            .id(SharedString::from(format!("thumbnail-face-{index}")))
                            .relative()
                            .rounded(px(8.))
                            .border_1()
                            .border_color(rgb(if target { p.focus } else { p.border }))
                            .bg(rgb(p.soft))
                            .active(|s| s.top(px(1.)))
                            .overflow_hidden()
                            .child(
                                div()
                                    .id(SharedString::from(format!("thumbnail-image-{index}")))
                                    .debug_selector(|| format!("thumbnail-image-{index}"))
                                    .h(px(image_height))
                                    .w_full()
                                    .child(if !selectable {
                                        div()
                                            .size_full()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_size(px(self.ui_text_size(12.)))
                                            .text_color(rgb(p.muted))
                                            .child(l.text("pb-notes-page"))
                                            .into_any_element()
                                    } else if data.failed_thumbnails.contains(&index) {
                                        div()
                                            .id(SharedString::from(format!(
                                                "thumbnail-error-{index}"
                                            )))
                                            .debug_selector(|| format!("thumbnail-error-{index}"))
                                            .size_full()
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .text_size(px(self.ui_text_size(12.)))
                                            .text_color(rgb(p.error))
                                            .child(l.text("pb-render-failed"))
                                            .into_any_element()
                                    } else {
                                        self.picture(Surface::Thumbnail(index), false)
                                    }),
                            )
                            .child(
                                div()
                                    .id(SharedString::from(format!("thumbnail-caption-{index}")))
                                    .debug_selector(|| format!("thumbnail-caption-{index}"))
                                    .px(px(6.))
                                    .py(px(2.))
                                    .text_size(px(self.ui_text_size(12.)))
                                    .text_color(rgb(p.body))
                                    .child(caption.clone()),
                            )
                            .when(current, |view| {
                                view.child(
                                    div()
                                        .absolute()
                                        .top(px(4.))
                                        .right(px(4.))
                                        .px(px(5.))
                                        .rounded(px(4.))
                                        .bg(rgb(p.card))
                                        .text_color(rgb(p.focus))
                                        .text_size(px(self.ui_text_size(12.)))
                                        .child(l.text("pb-current")),
                                )
                            }),
                    );
                row = row.child(tile);
            }
            grid = grid.child(row);
        }
        let wheel_target = self.shell.clone();
        let scroll = div()
            .id("page-browser-images")
            .debug_selector(|| "page-browser-images".into())
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .track_scroll(&self.browser_scroll)
            .on_scroll_wheel(move |event, _, cx| {
                let delta = event.delta.pixel_delta(px(20.)).y;
                if delta != px(0.) {
                    actions::dispatch(
                        &wheel_target,
                        Intent::GridMove(if delta < px(0.) {
                            columns as isize
                        } else {
                            -(columns as isize)
                        }),
                        cx,
                    );
                }
                cx.stop_propagation();
            })
            .child(grid)
            .when(data.entries.is_empty(), |view| {
                view.child(l.text("pb-empty"))
            });
        let helper = data
            .browser
            .error
            .map(|key| l.format(key, &[("pages", data.entries.len().to_string())]))
            .unwrap_or_else(|| data.target_context.clone());
        div()
            .id("page-browser")
            .debug_selector(|| "page-browser".into())
            .role(Role::Group)
            .aria_label(l.text("pb-title"))
            .occlude()
            .w(px(width))
            .h(px(height))
            .p(px(16.))
            .rounded(px(12.))
            .border_1()
            .border_color(rgb(p.border))
            .bg(rgb(p.card))
            .flex()
            .flex_col()
            .gap(px(8.))
            .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .flex_shrink_0()
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .text_color(rgb(p.ink))
                            .text_size(px(self.ui_text_size(20.)))
                            .child(l.text("pb-title"))
                            .child(
                                div()
                                    .text_size(px(self.ui_text_size(12.)))
                                    .text_color(rgb(p.muted))
                                    .child(wb.progress.clone()),
                            ),
                    )
                    .child(self.button(
                        "close-browser".into(),
                        l.text("close").into(),
                        true,
                        false,
                        Intent::CloseContextual,
                        p,
                    )),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .flex_shrink_0()
                    .gap(px(8.))
                    .child(l.text("pb-number-mode"))
                    .children(
                        [
                            (PageNumberMode::Content, "content", "pb-mode-content"),
                            (PageNumberMode::Pdf, "pdf", "pb-mode-pdf"),
                        ]
                        .into_iter()
                        .map(|(mode, id, label)| {
                            self.button(
                                format!("page-mode-{id}"),
                                l.text(label).into(),
                                enabled,
                                data.mode == mode,
                                Intent::BrowserMode {
                                    id: data.browser.id,
                                    mode,
                                },
                                p,
                            )
                            .aria_selected(data.mode == mode)
                        }),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .flex_shrink_0()
                    .gap(px(8.))
                    .child(div().text_size(px(self.ui_text_size(14.))).child(l.text(
                        if data.mode == PageNumberMode::Content {
                            "pb-content-number"
                        } else {
                            "pb-number"
                        },
                    )))
                    .child(self.number_field(p, cx))
                    .child(format!("/ {}", data.entries.len()))
                    .child(self.button(
                        "commit-jump".into(),
                        l.text("pb-go").into(),
                        enabled && data.browser.target.is_some(),
                        true,
                        Intent::BrowserSubmit {
                            id: data.browser.id,
                        },
                        p,
                    )),
            )
            .child(
                div()
                    .id("page-target-context")
                    .debug_selector(|| "page-target-context".into())
                    .flex_shrink_0()
                    .min_h(px(self.ui_text_size(20.)))
                    .text_size(px(self.ui_text_size(12.)))
                    .text_color(rgb(if data.browser.error.is_some() {
                        p.error
                    } else {
                        p.muted
                    }))
                    .child(helper),
            )
            .child(density.flex_shrink_0())
            .when(!wb.writable, |view| {
                view.child(
                    div()
                        .text_color(rgb(p.warning))
                        .text_size(px(self.ui_text_size(12.)))
                        .child(l.text("settings-readonly")),
                )
            })
            .when(self.data.dirty, |view| {
                view.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .flex_shrink_0()
                        .child(
                            div()
                                .flex_1()
                                .text_size(px(self.ui_text_size(12.)))
                                .text_color(rgb(p.warning))
                                .child(
                                    self.data
                                        .save_error
                                        .clone()
                                        .map(|error| l.error("save-error", error))
                                        .unwrap_or_else(|| l.text("unsaved").into()),
                                ),
                        )
                        .child(self.button(
                            "page-retry-save".into(),
                            l.text("retry-save").into(),
                            enabled && wb.writable,
                            false,
                            Intent::RetrySave,
                            p,
                        )),
                )
            })
            .child(scroll)
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(self.button(
                        "grid-prev".into(),
                        l.text("wb-earlier").into(),
                        enabled && wb.grid_start > 0,
                        false,
                        Intent::GridMove(-(capacity as isize)),
                        p,
                    ))
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(self.ui_text_size(12.)))
                            .child(l.format(
                                if data.mode == PageNumberMode::Content {
                                    "pb-range"
                                } else {
                                    "pb-range-pdf"
                                },
                                &[
                                    (
                                        "first",
                                        if end == 0 {
                                            "0".into()
                                        } else {
                                            (wb.grid_start + 1).to_string()
                                        },
                                    ),
                                    ("last", end.to_string()),
                                    ("total", data.entries.len().to_string()),
                                ],
                            )),
                    )
                    .child(self.button(
                        "grid-next".into(),
                        l.text("wb-later").into(),
                        enabled && end < data.entries.len(),
                        false,
                        Intent::GridMove(capacity as isize),
                        p,
                    )),
            )
            .into_any_element()
    }
}
