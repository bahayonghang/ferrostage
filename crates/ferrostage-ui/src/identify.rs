//! A short-lived display label. No presentation or document input is available.
use crate::theme;
use gpui::{Context, Render, Window, div, prelude::*, px, rgb};

pub(crate) struct Identify {
    pub number: usize,
    pub name: String,
    pub appearance: ferrostage_settings::Theme,
    pub typography: theme::Typography,
}

impl Render for Identify {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let dark = matches!(
            window.appearance(),
            gpui::WindowAppearance::Dark | gpui::WindowAppearance::VibrantDark
        );
        let palette = theme::palette(self.appearance, dark);
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(rgb(palette.layer_solid))
            .text_color(rgb(palette.ink))
            .font(self.typography.font())
            .border_2()
            .border_color(rgb(palette.focus))
            .p(px(16.))
            .child(
                div()
                    .font(self.typography.numeric_font())
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_size(px(self.typography.size(72.)))
                    .text_color(rgb(palette.accent_text))
                    .child(self.number.to_string()),
            )
            .child(
                div()
                    .w_full()
                    .text_center()
                    .text_size(px(self.typography.size(16.)))
                    .whitespace_normal()
                    .child(self.name.clone()),
            )
    }
}
