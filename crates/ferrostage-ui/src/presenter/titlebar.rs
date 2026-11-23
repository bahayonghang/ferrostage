use super::*;
use crate::theme::{self, Palette};
use gpui::{Decorations, WindowControlArea, WindowControls};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TitlebarPlatform {
    Windows,
    MacOs,
    Linux,
}

impl TitlebarPlatform {
    pub(crate) const fn current() -> Self {
        if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(target_os = "windows") {
            Self::Windows
        } else {
            Self::Linux
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct TitlebarLayout {
    pub visible: bool,
    pub leading_inset: f32,
    pub show_minimize: bool,
    pub show_maximize: bool,
    pub show_close: bool,
    pub native_hit_test_controls: bool,
}

impl TitlebarLayout {
    pub(crate) fn resolve(
        platform: TitlebarPlatform,
        fullscreen: bool,
        decorations: Decorations,
        controls: WindowControls,
    ) -> Self {
        if fullscreen {
            return Self::default();
        }
        match platform {
            TitlebarPlatform::Windows => Self {
                visible: true,
                leading_inset: 0.0,
                show_minimize: true,
                show_maximize: true,
                show_close: true,
                native_hit_test_controls: true,
            },
            TitlebarPlatform::MacOs => Self {
                visible: true,
                leading_inset: theme::MACOS_TRAFFIC_LIGHT_INSET,
                show_minimize: false,
                show_maximize: false,
                show_close: false,
                native_hit_test_controls: false,
            },
            TitlebarPlatform::Linux => match decorations {
                Decorations::Server => Self::default(),
                Decorations::Client { .. } => Self {
                    visible: true,
                    leading_inset: 0.0,
                    show_minimize: controls.minimize,
                    show_maximize: controls.maximize,
                    show_close: true,
                    native_hit_test_controls: false,
                },
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TitlebarColors {
    pub title_ink: u32,
    pub icon_ink: u32,
}

impl TitlebarColors {
    pub(crate) const fn resolve(p: Palette, active: bool) -> Self {
        if active {
            Self {
                title_ink: p.titlebar_ink,
                icon_ink: p.titlebar_icon,
            }
        } else {
            Self {
                title_ink: p.titlebar_inactive_ink,
                icon_ink: p.titlebar_inactive_icon,
            }
        }
    }
}

#[cfg(all(test, feature = "test-support"))]
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TitlebarProbe {
    pub title: String,
    pub active: bool,
    pub colors: TitlebarColors,
    pub layout: TitlebarLayout,
    pub field_top: u32,
    pub field_bottom: u32,
}

#[derive(Clone, Copy)]
enum CaptionIcon {
    Minimize,
    Maximize,
    Restore,
    Close,
}

pub(crate) fn presenter_window_title(
    stage: SessionStage,
    filename: &str,
    stage_label: &str,
) -> String {
    if stage == SessionStage::Idle {
        "FerroStage".into()
    } else {
        format!("{filename} — {stage_label} — FerroStage")
    }
}

fn caption_icon(icon: CaptionIcon) -> gpui::AnyElement {
    gpui::canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let color = window.text_style().color;
            let at = |x: f32, y: f32| {
                gpui::point(
                    bounds.origin.x + bounds.size.width * x,
                    bounds.origin.y + bounds.size.height * y,
                )
            };
            let mut path = gpui::PathBuilder::stroke(px(1.25));
            match icon {
                CaptionIcon::Minimize => {
                    path.move_to(at(0.05, 0.5));
                    path.line_to(at(0.95, 0.5));
                }
                CaptionIcon::Maximize => {
                    path.move_to(at(0.05, 0.05));
                    path.line_to(at(0.95, 0.05));
                    path.line_to(at(0.95, 0.95));
                    path.line_to(at(0.05, 0.95));
                    path.close();
                }
                CaptionIcon::Restore => {
                    path.move_to(at(0.05, 0.25));
                    path.line_to(at(0.75, 0.25));
                    path.line_to(at(0.75, 0.95));
                    path.line_to(at(0.05, 0.95));
                    path.close();
                    path.move_to(at(0.25, 0.25));
                    path.line_to(at(0.25, 0.05));
                    path.line_to(at(0.95, 0.05));
                    path.line_to(at(0.95, 0.75));
                    path.line_to(at(0.75, 0.75));
                }
                CaptionIcon::Close => {
                    path.move_to(at(0.1, 0.1));
                    path.line_to(at(0.9, 0.9));
                    path.move_to(at(0.9, 0.1));
                    path.line_to(at(0.1, 0.9));
                }
            }
            if let Ok(built) = path.build() {
                window.paint_path(built, color);
            }
        },
    )
    .size(px(10.))
    .flex_shrink_0()
    .into_any_element()
}

impl Presenter {
    pub(super) fn titlebar(
        &self,
        platform: TitlebarPlatform,
        layout: TitlebarLayout,
        p: Palette,
        title: &str,
        window: &Window,
    ) -> gpui::AnyElement {
        let l = self.locale;
        let active = window.is_window_active();
        let maximized = window.is_maximized();
        let can_maximize = window.window_controls().maximize;
        let colors = TitlebarColors::resolve(p, active);

        let mut drag = div()
            .id("titlebar-drag")
            .debug_selector(|| "titlebar-drag".into())
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .items_center()
            .pl(px(if layout.leading_inset > 0. { 0. } else { 12. }))
            .pr(px(12.))
            .window_control_area(WindowControlArea::Drag)
            .child(
                div()
                    .id("titlebar-title")
                    .debug_selector(|| "titlebar-title".into())
                    .min_w_0()
                    .truncate()
                    .text_size(px(12.))
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(rgb(colors.title_ink))
                    .child(title.to_string()),
            );
        if !layout.native_hit_test_controls {
            drag = drag
                .on_mouse_down(gpui::MouseButton::Left, move |event, window, _| {
                    if event.click_count == 2 {
                        match platform {
                            TitlebarPlatform::MacOs => window.titlebar_double_click(),
                            TitlebarPlatform::Linux if can_maximize => window.zoom_window(),
                            _ => {}
                        }
                    } else {
                        window.start_window_move();
                    }
                })
                .when(platform == TitlebarPlatform::Linux, |drag| {
                    drag.on_mouse_down(gpui::MouseButton::Right, |event, window, _| {
                        window.show_window_menu(event.position);
                    })
                });
        }

        let control_button = |id: &'static str,
                              label: &'static str,
                              area: WindowControlArea,
                              icon: CaptionIcon,
                              close_danger: bool| {
            let mut btn = div()
                .id(id)
                .debug_selector(move || id.into())
                .accessibility_id(id)
                .role(Role::Button)
                .aria_label(l.text(label))
                .w(px(theme::TITLEBAR_BUTTON_WIDTH))
                .h_full()
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_center()
                .window_control_area(area)
                .text_color(rgb(colors.icon_ink))
                .child(caption_icon(icon));
            if close_danger {
                btn = btn
                    .hover(move |style| {
                        style
                            .bg(rgb(p.titlebar_close_hover))
                            .text_color(rgb(p.titlebar_on_close))
                    })
                    .active(move |style| {
                        style
                            .bg(rgb(p.titlebar_close_pressed))
                            .text_color(rgb(p.titlebar_on_close))
                    });
            } else {
                btn = btn
                    .hover(move |style| {
                        style
                            .bg(gpui::rgba(p.titlebar_control_hover))
                            .text_color(rgb(p.titlebar_icon))
                    })
                    .active(move |style| {
                        style
                            .bg(gpui::rgba(p.titlebar_control_pressed))
                            .text_color(rgb(p.titlebar_icon))
                    });
            }
            btn
        };

        let mut bar = div()
            .id("presenter-titlebar")
            .debug_selector(|| "presenter-titlebar".into())
            .h(px(theme::TITLEBAR_HEIGHT))
            .w_full()
            .flex_shrink_0()
            .flex()
            .items_center();

        if layout.leading_inset > 0. {
            bar = bar.child(div().w(px(layout.leading_inset)).h_full().flex_shrink_0());
        }
        bar = bar.child(drag);

        if layout.show_minimize {
            let min_btn = control_button(
                "titlebar-minimize",
                "window-minimize",
                WindowControlArea::Min,
                CaptionIcon::Minimize,
                false,
            )
            .on_a11y_action(AccessibleAction::Click, |_, window, _| {
                window.minimize_window();
            })
            .when(!layout.native_hit_test_controls, |btn| {
                btn.on_click(|_, window, _| window.minimize_window())
            });
            bar = bar.child(min_btn);
        }

        if layout.show_maximize {
            let (label, icon) = if maximized {
                ("window-restore", CaptionIcon::Restore)
            } else {
                ("window-maximize", CaptionIcon::Maximize)
            };
            let max_btn = control_button(
                "titlebar-maximize",
                label,
                WindowControlArea::Max,
                icon,
                false,
            )
            .on_a11y_action(AccessibleAction::Click, |_, window, _| {
                window.zoom_window();
            })
            .when(!layout.native_hit_test_controls, |btn| {
                btn.on_click(|_, window, _| window.zoom_window())
            });
            bar = bar.child(max_btn);
        }

        if layout.show_close {
            let close_btn = control_button(
                "titlebar-close",
                "window-close",
                WindowControlArea::Close,
                CaptionIcon::Close,
                true,
            )
            .on_a11y_action(AccessibleAction::Click, |_, window, cx| {
                crate::shell::request_window_close(window, cx);
            })
            .when(!layout.native_hit_test_controls, |btn| {
                btn.on_click(|_, window, cx| crate::shell::request_window_close(window, cx))
            });
            bar = bar.child(close_btn);
        }

        bar.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locale::Locale;
    use gpui::Tiling;

    #[test]
    fn titlebar_window_title_matches_session_stage() {
        assert_eq!(
            presenter_window_title(SessionStage::Idle, "FerroStage", "未打开文档"),
            "FerroStage"
        );
        assert_eq!(
            presenter_window_title(SessionStage::Ready, "talk.pdf", "Ready"),
            "talk.pdf — Ready — FerroStage"
        );
        assert_eq!(
            presenter_window_title(SessionStage::Presenting, "talk.pdf", "演示中"),
            "talk.pdf — 演示中 — FerroStage"
        );
    }

    #[test]
    fn titlebar_layout_resolves_cross_platform_and_fullscreen() {
        let all_controls = WindowControls::default();
        let win = TitlebarLayout::resolve(
            TitlebarPlatform::Windows,
            false,
            Decorations::Server,
            all_controls,
        );
        assert_eq!(
            win,
            TitlebarLayout {
                visible: true,
                leading_inset: 0.0,
                show_minimize: true,
                show_maximize: true,
                show_close: true,
                native_hit_test_controls: true,
            }
        );

        let mac = TitlebarLayout::resolve(
            TitlebarPlatform::MacOs,
            false,
            Decorations::Server,
            all_controls,
        );
        assert_eq!(
            mac,
            TitlebarLayout {
                visible: true,
                leading_inset: theme::MACOS_TRAFFIC_LIGHT_INSET,
                show_minimize: false,
                show_maximize: false,
                show_close: false,
                native_hit_test_controls: false,
            }
        );

        let linux_ssd = TitlebarLayout::resolve(
            TitlebarPlatform::Linux,
            false,
            Decorations::Server,
            all_controls,
        );
        assert!(!linux_ssd.visible);

        let linux_csd = TitlebarLayout::resolve(
            TitlebarPlatform::Linux,
            false,
            Decorations::Client {
                tiling: Tiling::default(),
            },
            WindowControls {
                fullscreen: true,
                maximize: false,
                minimize: true,
                window_menu: true,
            },
        );
        assert_eq!(
            linux_csd,
            TitlebarLayout {
                visible: true,
                leading_inset: 0.0,
                show_minimize: true,
                show_maximize: false,
                show_close: true,
                native_hit_test_controls: false,
            }
        );

        for platform in [
            TitlebarPlatform::Windows,
            TitlebarPlatform::MacOs,
            TitlebarPlatform::Linux,
        ] {
            let fs = TitlebarLayout::resolve(
                platform,
                true,
                Decorations::Client {
                    tiling: Tiling::default(),
                },
                all_controls,
            );
            assert!(!fs.visible);
            assert!(!fs.show_minimize && !fs.show_maximize && !fs.show_close);
        }
    }

    #[test]
    fn titlebar_colors_distinguish_active_and_inactive_windows() {
        for p in [theme::LIGHT, theme::DARK] {
            let active = TitlebarColors::resolve(p, true);
            assert_eq!(active.title_ink, p.titlebar_ink);
            assert_eq!(active.icon_ink, p.titlebar_icon);

            let inactive = TitlebarColors::resolve(p, false);
            assert_eq!(inactive.title_ink, p.titlebar_inactive_ink);
            assert_eq!(inactive.icon_ink, p.titlebar_inactive_icon);
            assert_ne!(active, inactive);
        }
    }

    #[test]
    fn titlebar_locale_keys_exist_in_both_catalogs() {
        for (lang, expected) in [
            ("zh-CN", ["最小化", "最大化", "还原", "关闭"]),
            ("en-US", ["Minimize", "Maximize", "Restore", "Close"]),
        ] {
            let locale = Locale::new(lang);
            let actual = [
                locale.text("window-minimize"),
                locale.text("window-maximize"),
                locale.text("window-restore"),
                locale.text("window-close"),
            ];
            assert_eq!(actual, expected);
        }
    }
}
