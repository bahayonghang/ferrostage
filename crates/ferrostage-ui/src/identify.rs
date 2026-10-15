//! A short-lived display label. No presentation or document input is available.
use crate::theme;
use gpui::{Context, Render, Window, div, prelude::*, px, rgb};

pub(crate) struct Identify {
    pub number: usize,
    pub name: String,
}

impl Render for Identify {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let palette = theme::palette(ferrostage_settings::Theme::Dark, true);
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(rgb(palette.canvas))
            .text_color(rgb(palette.ink))
            .font(theme::sans_font())
            .border_2()
            .border_color(rgb(palette.focus))
            .p(px(16.))
            .child(div().text_size(px(72.)).child(self.number.to_string()))
            .child(div().text_size(px(16.)).child(self.name.clone()))
    }
}
