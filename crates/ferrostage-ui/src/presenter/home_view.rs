use super::*;

pub(super) fn last_opened(locale: crate::locale::Locale, timestamp: Option<i64>) -> String {
    timestamp
        .filter(|timestamp| *timestamp >= 0)
        .and_then(|timestamp| chrono::DateTime::from_timestamp(timestamp, 0))
        .map(|date| date.with_timezone(&chrono::Local))
        .and_then(|date| date.naive_utc().checked_add_offset(*date.offset()))
        .map(|date| date.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| locale.text("recent-time-unknown").into())
}

impl Presenter {
    pub(super) fn home_content(&self, p: Palette, window: &Window) -> Stateful<Div> {
        let l = self.locale;
        let compact = window.viewport_size().width < px(600.);
        let utilities = div()
            .id("home-utilities")
            .debug_selector(|| "home-utilities".into())
            .flex()
            .items_center()
            .gap(px(8.))
            .child(self.home_utility("settings", l.text("settings"), Intent::Settings, p))
            .child(self.home_utility("displays", l.text("displays"), Intent::Displays, p));
        let header = div()
            .flex()
            .when(compact, |view| view.flex_col().items_start().gap(px(8.)))
            .when(!compact, |view| view.items_center().justify_between())
            .child(
                div()
                    .id("home-brand")
                    .debug_selector(|| "home-brand".into())
                    .pl(px(theme::FOCUS_WIDTH + theme::FOCUS_GAP))
                    .font(theme::home_brand_font())
                    .text_size(px(32.))
                    .line_height(px(38.))
                    .text_color(rgb(p.ink))
                    .child("FerroStage"),
            )
            .child(utilities);
        let label = if self.data.loading {
            l.text("loading")
        } else {
            l.text("open")
        };
        let style = p.button(theme::ButtonKind::Primary, self.data.can_open, false);
        let open = self
            .activation_target(
                "welcome-open".into(),
                label.into(),
                self.data.can_open,
                Intent::Open,
                p,
            )
            .child(
                div()
                    .id("welcome-open-face")
                    .debug_selector(|| "welcome-open-face".into())
                    .min_w(px(120.))
                    .min_h(px(theme::CONTROL))
                    .px(px(16.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .relative()
                    .rounded(px(theme::RADIUS))
                    .bg(rgb(style.fill))
                    .text_color(rgb(style.ink))
                    .border_1()
                    .border_color(rgb(style.border))
                    .text_size(px(self.data.typography.size(14.)))
                    .line_height(px(self.data.typography.size(20.)))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .child(label)
                    .when(self.data.can_open, |view| {
                        view.hover(move |state| state.bg(rgb(style.hover)))
                            .active(move |state| state.bg(rgb(style.pressed)).top(px(1.)))
                    }),
            );
        let time_width = self.data.typography.size(if compact { 98. } else { 152. });
        let mut recents = div().flex().flex_col().child(
            div()
                .id("recent-header")
                .debug_selector(|| "recent-header".into())
                .px(px(theme::FOCUS_WIDTH + theme::FOCUS_GAP))
                .pb(px(8.))
                .mb(px(4.))
                .border_b_1()
                .border_color(rgb(p.border))
                .flex()
                .items_center()
                .gap(px(16.))
                .text_size(px(self.data.typography.size(12.)))
                .line_height(px(self.data.typography.size(18.)))
                .text_color(rgb(p.muted))
                .child(div().flex_1().min_w_0().child(l.text("recent")))
                .child(
                    div()
                        .id("recent-time-header")
                        .debug_selector(|| "recent-time-header".into())
                        .w(px(time_width))
                        .flex_shrink_0()
                        .text_right()
                        .child(l.text("recent-last-opened")),
                ),
        );
        if self.data.recent.is_empty() {
            recents = recents.child(
                div()
                    .id("home-empty")
                    .debug_selector(|| "home-empty".into())
                    .pl(px(4.))
                    .text_size(px(self.data.typography.size(14.)))
                    .line_height(px(self.data.typography.size(22.)))
                    .text_color(rgb(p.muted))
                    .child(l.text("recent-empty")),
            );
        }
        for (index, recent) in self.data.recent.iter().enumerate() {
            let path = &recent.path;
            let file = std::path::Path::new(path);
            let filename = file
                .file_name()
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.clone());
            let parent = file
                .parent()
                .map(|value| value.to_string_lossy().into_owned())
                .filter(|value| !value.is_empty());
            let tooltip_path = path.clone();
            let typography = self.data.typography.clone();
            let row = self
                .activation_target(
                    format!("recent-{index}"),
                    l.format(
                        "recent-open-time",
                        &[("path", path.clone()), ("time", recent.last_opened.clone())],
                    ),
                    self.data.can_open,
                    Intent::Import(path.into()),
                    p,
                )
                .tooltip(move |_, cx| {
                    cx.new(|_| FilenameTooltip {
                        filename: tooltip_path.clone(),
                        palette: p,
                        typography: typography.clone(),
                    })
                    .into()
                })
                .child(
                    div()
                        .id(SharedString::from(format!("recent-{index}-face")))
                        .debug_selector(move || format!("recent-{index}-face"))
                        .min_h(px(theme::CONTROL))
                        .flex()
                        .items_center()
                        .gap(px(16.))
                        .min_w_0()
                        .relative()
                        .rounded(px(4.))
                        .text_size(px(self.data.typography.size(14.)))
                        .line_height(px(self.data.typography.size(20.)))
                        .text_color(rgb(if self.data.can_open { p.ink } else { p.muted }))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .id(SharedString::from(format!("recent-{index}-name")))
                                        .debug_selector(move || format!("recent-{index}-name"))
                                        .truncate()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .child(filename),
                                )
                                .when_some(parent, |view, parent| {
                                    view.child(
                                        div()
                                            .id(SharedString::from(format!(
                                                "recent-{index}-parent"
                                            )))
                                            .debug_selector(move || {
                                                format!("recent-{index}-parent")
                                            })
                                            .truncate()
                                            .text_size(px(self.data.typography.size(12.)))
                                            .line_height(px(self.data.typography.size(18.)))
                                            .text_color(rgb(p.muted))
                                            .child(parent),
                                    )
                                }),
                        )
                        .child(
                            div()
                                .id(SharedString::from(format!("recent-{index}-time")))
                                .debug_selector(move || format!("recent-{index}-time"))
                                .w(px(time_width))
                                .flex_shrink_0()
                                .text_right()
                                .whitespace_normal()
                                .text_size(px(self.data.typography.size(12.)))
                                .line_height(px(self.data.typography.size(18.)))
                                .text_color(rgb(p.muted))
                                .child(recent.last_opened.clone()),
                        )
                        .when(self.data.can_open, |view| {
                            view.hover(move |state| state.bg(rgb(p.soft)))
                                .active(move |state| state.bg(rgb(p.card)).top(px(1.)))
                        }),
                );
            recents = recents.child(div().border_b_1().border_color(rgb(p.border)).child(row));
        }
        // Keyboard users get the same complete path as the pointer tooltip.
        if let Some(path) = self
            .data
            .recent
            .iter()
            .enumerate()
            .find_map(|(index, recent)| {
                self.home_focus[index + 3]
                    .is_focused(window)
                    .then_some(&recent.path)
            })
        {
            recents = recents.child(
                div()
                    .id("recent-full-path")
                    .debug_selector(|| "recent-full-path".into())
                    .pl(px(4.))
                    .text_size(px(self.data.typography.size(12.)))
                    .line_height(px(self.data.typography.size(18.)))
                    .text_color(rgb(p.muted))
                    .whitespace_normal()
                    .child(path.clone()),
            );
        }
        div()
            .id("home-content")
            .debug_selector(|| "home-content".into())
            .w_full()
            .max_w(px(theme::HOME_WIDTH))
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(16.))
            .text_size(px(self.data.typography.size(14.)))
            .line_height(px(self.data.typography.size(22.)))
            .child(header)
            .child(
                div()
                    .mt(px(8.))
                    .pl(px(4.))
                    .text_color(rgb(p.muted))
                    .child(l.text("welcome")),
            )
            .child(div().flex().items_start().child(open))
            .child(div().mt(px(16.)).child(recents))
    }

    fn home_utility(&self, id: &str, label: &str, intent: Intent, p: Palette) -> Stateful<Div> {
        self.activation_target(id.into(), label.into(), !self.data.busy, intent, p)
            .child(
                div()
                    .id(SharedString::from(format!("{id}-face")))
                    .min_h(px(theme::CONTROL))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .relative()
                    .rounded(px(theme::RADIUS))
                    .text_size(px(self.data.typography.size(14.)))
                    .line_height(px(self.data.typography.size(20.)))
                    .text_color(rgb(if self.data.busy { p.muted } else { p.body }))
                    .child(label.to_string())
                    .when(!self.data.busy, |view| {
                        view.hover(move |state| state.bg(rgb(p.soft)))
                            .active(move |state| state.bg(rgb(p.card)).top(px(1.)))
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::last_opened;
    use crate::locale::Locale;

    #[test]
    fn recent_dates_use_local_minutes_and_localized_unknown_values() {
        for language in ["zh-CN", "en-US"] {
            let locale = Locale::new(language);
            for timestamp in [None, Some(-1), Some(i64::MAX)] {
                assert_eq!(
                    last_opened(locale, timestamp),
                    locale.text("recent-time-unknown")
                );
            }
            let timestamp = 1_792_375_595;
            let expected = chrono::DateTime::from_timestamp(timestamp, 0)
                .unwrap()
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M")
                .to_string();
            assert_eq!(last_opened(locale, Some(timestamp)), expected);
            assert_eq!(expected.len(), 16);
        }
    }
}
