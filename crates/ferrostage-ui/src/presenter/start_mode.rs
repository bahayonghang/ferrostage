use super::*;
use ferrostage_settings::StartMode;

/// Menu rows in display order: (mode, label key, helper key).
pub(super) const CHOICES: [(StartMode, &str, &str); 2] = [
    (StartMode::Dual, "start-mode-dual", "start-mode-dual-help"),
    (
        StartMode::Single,
        "start-mode-single",
        "start-mode-single-help",
    ),
];

pub(super) fn label(mode: StartMode) -> &'static str {
    CHOICES
        .iter()
        .find(|(choice, ..)| *choice == mode)
        .map_or("start-mode-dual", |(_, label, _)| label)
}

impl Presenter {
    pub(super) fn start_mode_index(&self) -> usize {
        CHOICES
            .iter()
            .position(|(mode, ..)| *mode == self.data.start_mode)
            .unwrap_or(0)
    }

    pub(super) fn start_mode_panel(&self, p: Palette, window: &Window) -> gpui::AnyElement {
        let l = self.locale;
        let width = 320f32
            .min(f32::from(window.viewport_size().width) - 24.)
            .max(200.);
        let mut body = div().flex().flex_col().gap(px(2.));
        for (index, (mode, key, help)) in CHOICES.into_iter().enumerate() {
            let checked = self.data.start_mode == mode;
            let label = l.text(key);
            let helper = l.text(help);
            body = body.child(
                self.activation_target(
                    format!("start-mode-row-{index}"),
                    format!("{label}. {helper}"),
                    !self.data.busy,
                    Intent::SetStartMode(mode),
                    p,
                )
                .track_focus(
                    &self.start_row_focus[index]
                        .clone()
                        .tab_stop(!self.data.busy),
                )
                .role(Role::MenuItemRadio)
                .aria_toggled(if checked {
                    gpui::Toggled::True
                } else {
                    gpui::Toggled::False
                })
                .child(
                    div()
                        .min_h(px(36.))
                        .px(px(8.))
                        .py(px(4.))
                        .rounded(px(8.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .bg(rgb(if checked { p.soft } else { p.card }))
                        .text_color(rgb(p.ink))
                        .hover(|style| style.bg(rgb(p.soft)))
                        .child(
                            div()
                                .w(px(16.))
                                .flex_shrink_0()
                                .when(checked, |view| {
                                    view.debug_selector(move || format!("start-mode-check-{index}"))
                                })
                                .child(if checked { "✓" } else { "" }),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_size(px(self.ui_text_size(14.)))
                                .child(label)
                                .child(
                                    div()
                                        .text_size(px(self.ui_text_size(12.)))
                                        .text_color(rgb(p.muted))
                                        .whitespace_normal()
                                        .child(helper),
                                ),
                        ),
                ),
            );
        }
        div()
            .id("start-mode-menu")
            .debug_selector(|| "start-mode-menu".into())
            .role(Role::Menu)
            .aria_label(l.text("start-mode"))
            .occlude()
            .w(px(width))
            .p(px(8.))
            .rounded(px(12.))
            .border_1()
            .border_color(rgb(p.border))
            .bg(rgb(p.card))
            .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(body)
            .into_any_element()
    }
}
