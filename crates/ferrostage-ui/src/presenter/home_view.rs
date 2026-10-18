use super::*;

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
        let mut recents = div().flex().flex_col().gap(px(4.)).child(
            div()
                .pl(px(4.))
                .mb(px(4.))
                .text_size(px(self.data.typography.size(12.)))
                .line_height(px(self.data.typography.size(18.)))
                .text_color(rgb(p.muted))
                .child(l.text("recent")),
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
        for (index, path) in self.data.recent.iter().enumerate() {
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
                    l.format("recent-open", &[("path", path.clone())]),
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
                        .flex_col()
                        .justify_center()
                        .min_w_0()
                        .relative()
                        .rounded(px(4.))
                        .text_size(px(self.data.typography.size(14.)))
                        .line_height(px(self.data.typography.size(20.)))
                        .text_color(rgb(if self.data.can_open { p.ink } else { p.muted }))
                        .child(
                            div()
                                .truncate()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(filename),
                        )
                        .when_some(parent, |view, parent| {
                            view.child(
                                div()
                                    .truncate()
                                    .text_size(px(self.data.typography.size(12.)))
                                    .line_height(px(self.data.typography.size(18.)))
                                    .text_color(rgb(p.muted))
                                    .child(parent),
                            )
                        })
                        .when(self.data.can_open, |view| {
                            view.hover(move |state| state.bg(rgb(p.soft)))
                                .active(move |state| state.bg(rgb(p.card)).top(px(1.)))
                        }),
                );
            recents = recents.child(row);
        }
        // Keyboard users get the same complete path as the pointer tooltip.
        if let Some(path) = self
            .data
            .recent
            .iter()
            .enumerate()
            .find_map(|(index, path)| {
                self.home_focus[index + 3]
                    .is_focused(window)
                    .then_some(path)
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
