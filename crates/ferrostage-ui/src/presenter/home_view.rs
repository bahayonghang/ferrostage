use super::*;

pub(super) fn last_opened(
    locale: crate::locale::Locale,
    seconds: Option<i64>,
    zone: jiff::tz::TimeZone,
) -> String {
    seconds
        .filter(|&seconds| seconds >= 0)
        .and_then(|seconds| jiff::Timestamp::from_second(seconds).ok())
        .map(|time| {
            time.to_zoned_datetime(zone)
                .strftime("%Y-%m-%d %H:%M")
                .to_string()
        })
        .unwrap_or_else(|| locale.text("recent-time-unknown").into())
}

impl Presenter {
    pub(super) fn home_content(
        &self,
        p: Palette,
        has_error: bool,
        window: &Window,
    ) -> Stateful<Div> {
        let l = self.locale;
        let columns = window.viewport_size().width >= px(self.ui_text_size(1000.));
        let compact = window.viewport_size().width < px(self.ui_text_size(700.));
        let utilities = div()
            .id("home-utilities")
            .debug_selector(|| "home-utilities".into())
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(8.))
            .child(self.home_utility("settings", l.text("settings"), Intent::Settings, p))
            .child(self.home_utility("displays", l.text("displays"), Intent::Displays, p))
            .child(
                self.button(
                    "welcome-open".into(),
                    l.text(if self.data.loading { "loading" } else { "open" })
                        .into(),
                    self.data.can_open,
                    self.data.home_preview.path.is_none(),
                    Intent::Open,
                    p,
                ),
            );
        let header = div()
            .flex()
            .gap(px(16.))
            .when(compact, |view| view.flex_col().items_start())
            .when(!compact, |view| view.items_center().justify_between())
            .child(
                div()
                    .id("home-brand")
                    .debug_selector(|| "home-brand".into())
                    .font(self.data.typography.font())
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_size(px(self.ui_text_size(28.)))
                    .line_height(px(self.ui_text_size(36.)))
                    .text_color(rgb(p.ink))
                    .child("FerroStage"),
            )
            .child(utilities);
        let history = self
            .home_history(p, window)
            .when(columns, |view| {
                view.w(px(self.ui_text_size(340.))).flex_shrink_0()
            })
            .when(!columns, |view| view.w_full());
        let preview = self
            .home_preview_panel(p, window, columns, has_error)
            .when(columns, |view| view.flex_1().min_w_0())
            .when(!columns, |view| view.w_full());
        div()
            .id("home-content")
            .debug_selector(|| "home-content".into())
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(24.))
            .text_size(px(self.ui_text_size(14.)))
            .line_height(px(self.ui_text_size(22.)))
            .child(header)
            .child(
                div()
                    .flex()
                    .gap(px(32.))
                    .when(!columns, |view| view.flex_col())
                    .child(history)
                    .child(preview),
            )
            .child(
                self.material
                    .panel(p)
                    .id("home-preparation")
                    .debug_selector(|| "home-preparation".into())
                    .w_full()
                    .min_w_0()
                    .px(px(16.))
                    .py(px(10.))
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(16.))
                    .text_size(px(self.ui_text_size(12.)))
                    .text_color(rgb(p.muted))
                    .child(l.text("home-configured-target"))
                    .child(
                        div()
                            .id("home-configured-target")
                            .debug_selector(|| "home-configured-target".into())
                            .min_w_0()
                            .whitespace_normal()
                            .child(self.data.configured_target.clone()),
                    )
                    .child(l.text(super::start_mode::label(self.data.start_mode))),
            )
    }

    fn home_history(&self, p: Palette, window: &Window) -> Stateful<Div> {
        let l = self.locale;
        let mut list = self
            .material
            .panel(p)
            .id("home-history")
            .debug_selector(|| "home-history".into())
            .min_w_0()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                div()
                    .pb(px(4.))
                    .text_color(rgb(p.muted))
                    .child(l.text("recent")),
            );
        if self.data.recent.is_empty() {
            list = list.child(
                div()
                    .id("home-empty")
                    .debug_selector(|| "home-empty".into())
                    .whitespace_normal()
                    .text_color(rgb(p.muted))
                    .child(l.text("recent-empty")),
            );
        }
        let hover_bg = self.material.control_hover_bg(p);
        for (index, recent) in self.data.recent.iter().enumerate() {
            let path = std::path::Path::new(&recent.path);
            let filename = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| recent.path.clone());
            let parent = path
                .parent()
                .map(|parent| parent.to_string_lossy().into_owned())
                .unwrap_or_default();
            let selected = self.data.home_preview.path.as_deref() == Some(path);
            let row_bg = if selected {
                rgb(p.accent_wash)
            } else {
                self.material.well_bg(p)
            };
            list = list.child(
                self.activation_target(
                    format!("recent-{index}"),
                    l.format(
                        "home-preview-select",
                        &[
                            ("path", recent.path.clone()),
                            ("time", recent.last_opened.clone()),
                        ],
                    ),
                    self.data.can_open,
                    Intent::HomePreview(path.into()),
                    p,
                )
                .aria_toggled(selected.into())
                .tooltip(self.text_tooltip(recent.path.clone(), p))
                .child(
                    div()
                        .id(SharedString::from(format!("recent-{index}-face")))
                        .debug_selector(move || format!("recent-{index}-face"))
                        .p(px(12.))
                        .rounded(px(theme::WELL_RADIUS))
                        .bg(row_bg)
                        .flex()
                        .items_start()
                        .gap(px(8.))
                        .min_w_0()
                        .text_color(rgb(p.ink))
                        .child(
                            div()
                                .w(px(16.))
                                .flex_shrink_0()
                                .pt(px(1.))
                                .text_color(rgb(p.accent_text))
                                .child(if selected { "✓" } else { "" }),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .id(SharedString::from(format!("recent-{index}-name")))
                                        .debug_selector(move || format!("recent-{index}-name"))
                                        .min_w_0()
                                        .truncate()
                                        .font_weight(gpui::FontWeight::SEMIBOLD)
                                        .child(filename),
                                )
                                .child(
                                    div()
                                        .id(SharedString::from(format!("recent-{index}-parent")))
                                        .debug_selector(move || format!("recent-{index}-parent"))
                                        .truncate()
                                        .text_size(px(self.ui_text_size(12.)))
                                        .text_color(rgb(p.muted))
                                        .child(parent),
                                )
                                .child(
                                    div()
                                        .id(SharedString::from(format!("recent-{index}-time")))
                                        .debug_selector(move || format!("recent-{index}-time"))
                                        .text_size(px(self.ui_text_size(12.)))
                                        .text_color(rgb(p.muted))
                                        .child(recent.last_opened.clone()),
                                ),
                        )
                        .when(self.data.can_open, |view| {
                            view.hover(move |style| style.bg(hover_bg))
                        }),
                ),
            );
        }
        if let Some(recent) = self
            .data
            .recent
            .iter()
            .enumerate()
            .find_map(|(index, recent)| {
                self.home_focus[index + 3]
                    .is_focused(window)
                    .then_some(recent)
            })
        {
            list = list.child(
                div()
                    .id("recent-full-path")
                    .debug_selector(|| "recent-full-path".into())
                    .max_w_full()
                    .whitespace_normal()
                    .text_size(px(self.ui_text_size(12.)))
                    .text_color(rgb(p.muted))
                    .child(recent.path.clone()),
            );
        }
        list
    }

    fn home_preview_panel(
        &self,
        p: Palette,
        window: &Window,
        columns: bool,
        has_error: bool,
    ) -> Stateful<Div> {
        let l = self.locale;
        let preview = &self.data.home_preview;
        let selected = preview.path.as_ref();
        // Available workspace height, with space reserved for header, metadata,
        // output summary and any pinned top error banner. Narrow windows keep a
        // compact scrollable preview.
        let error_reserve = if has_error { 64. } else { 0. };
        let height = if columns {
            (f32::from(window.viewport_size().height) - self.ui_text_size(440. + error_reserve))
                .max(220.)
        } else {
            160.
        };
        let mut picture = div()
            .id("home-preview-image")
            .debug_selector(|| "home-preview-image".into())
            .h(px(height))
            .w_full()
            .min_w_0()
            .p(px(16.))
            .rounded(px(theme::WELL_RADIUS))
            .bg(self.material.well_bg(p))
            .overflow_hidden()
            .flex()
            .items_center()
            .justify_center();
        let cache_error = self.home_textures.borrow().error().is_some();
        if preview.frame.is_some() && !cache_error {
            let cache = self.home_textures.clone();
            let token = preview.token;
            picture = picture.child(
                gpui::canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| {
                        cache.borrow_mut().paint(
                            token,
                            None,
                            window,
                            cx,
                            |frame, image, _, window| {
                                crate::slide_surface::paint(bounds, frame, image, None, window)
                            },
                        );
                    },
                )
                .size_full(),
            );
        } else {
            picture = picture.child(
                div()
                    .id("home-preview-status")
                    .debug_selector(|| "home-preview-status".into())
                    .max_w_full()
                    .whitespace_normal()
                    .text_color(rgb(p.muted))
                    .child(l.text(if cache_error {
                        "wb-render-failed"
                    } else if let Some(error) = preview.error {
                        error
                    } else if selected.is_some() {
                        "wb-rendering"
                    } else if self.data.recent.is_empty() {
                        "home-preview-empty"
                    } else {
                        "home-preview-choose"
                    })),
            );
        }
        self.material
            .panel(p)
            .id("home-preview")
            .debug_selector(|| "home-preview".into())
            .min_w_0()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                div()
                    .text_color(rgb(p.muted))
                    .child(l.text("home-preview-title")),
            )
            .child(picture)
            .when_some(selected, |view, path| {
                view.child(
                    div()
                        .text_size(px(self.ui_text_size(20.)))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(p.ink))
                        .whitespace_normal()
                        .child(
                            path.file_name()
                                .map(|name| name.to_string_lossy().into_owned())
                                .unwrap_or_default(),
                        ),
                )
                .child(
                    div()
                        .id("home-preview-path")
                        .debug_selector(|| "home-preview-path".into())
                        .max_w_full()
                        .whitespace_normal()
                        .text_size(px(self.ui_text_size(12.)))
                        .text_color(rgb(p.muted))
                        .child(path.to_string_lossy().into_owned()),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(8.))
                        .child(self.button(
                            "home-open-selected".into(),
                            l.text("home-open-selected").into(),
                            self.data.can_open,
                            true,
                            Intent::Import(path.clone()),
                            p,
                        ))
                        .when(preview.error.is_some() || cache_error, |view| {
                            view.child(self.button(
                                "home-preview-retry".into(),
                                l.text("home-preview-retry").into(),
                                self.data.can_open,
                                false,
                                Intent::HomePreview(path.clone()),
                                p,
                            ))
                        }),
                )
            })
    }

    fn home_utility(&self, id: &str, label: &str, intent: Intent, p: Palette) -> Stateful<Div> {
        self.button(id.into(), label.into(), !self.data.busy, false, intent, p)
    }
}

#[cfg(test)]
mod tests {
    use super::last_opened;
    use crate::locale::Locale;
    #[test]
    fn recent_dates_use_local_minutes_and_localized_unknown_values() {
        let zone = jiff::tz::TimeZone::get("America/New_York").unwrap();
        for language in ["zh-CN", "en-US"] {
            let locale = Locale::new(language);
            for timestamp in [None, Some(-1), Some(i64::MAX)] {
                assert_eq!(
                    last_opened(locale, timestamp, zone.clone()),
                    locale.text("recent-time-unknown")
                );
            }
            for (seconds, expected) in [
                (0, "1969-12-31 19:00"),
                (1_772_953_140, "2026-03-08 01:59"),
                (1_772_953_200, "2026-03-08 03:00"),
            ] {
                assert_eq!(last_opened(locale, Some(seconds), zone.clone()), expected);
            }
        }
    }
}
