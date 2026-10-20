use super::*;
use crate::settings_panel::{self, Category, Field, Kind, PanelState};
use ferrostage_settings::Settings;
use gpui::{MouseButton, canvas, relative};
use std::{cell::Cell, rc::Rc};

pub(super) struct SettingsData {
    settings: Settings,
    pub panel: PanelState,
    writable: bool,
    path: String,
    fonts: std::sync::Arc<Vec<String>>,
}
impl SettingsData {
    pub fn from_shell(shell: &Shell) -> Self {
        Self {
            settings: shell.settings.settings.clone(),
            panel: shell.preferences.clone(),
            writable: shell.settings.write_policy.is_writable(),
            path: shell.config_path().display().to_string(),
            fonts: shell.ui_fonts.clone(),
        }
    }
}
impl Presenter {
    pub(super) fn settings_panel(&self, p: Palette, window: &Window) -> impl IntoElement {
        let data = &self.data.preferences;
        let l = self.locale;
        let enabled = data.writable && !self.data.busy;
        let narrow = window.viewport_size().width < px(760.);
        let mut rail = div()
            .flex()
            .gap(px(4.))
            .when(narrow, |v| v.flex_wrap())
            .when(!narrow, |v| v.flex_col().w(px(176.)).flex_shrink_0());
        for category in Category::ALL {
            rail = rail.child(
                self.button(
                    format!("settings-category-{}", category.id()),
                    l.text(&format!("settings-category-{}", category.id()))
                        .into(),
                    true,
                    data.panel.category == category,
                    Intent::SettingsCategory(category),
                    p,
                ),
            );
        }
        let category = data.panel.category;
        let mut content = div()
            .id("settings-content")
            .debug_selector(|| "settings-content".into())
            .flex_1()
            .min_h_0()
            .min_w_0()
            .max_w(px(if data.panel.category == Category::Displays {
                theme::SETTINGS_DISPLAY_WIDTH
            } else {
                theme::SETTINGS_WIDTH
            }))
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap(px(0.));
        content = content.child(
            div()
                .pb(px(12.))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_size(px(self.data.typography.size(20.)))
                .text_color(rgb(p.ink))
                .child(l.text(&format!("settings-category-{}", category.id()))),
        );
        for field in settings_panel::FIELDS
            .iter()
            .filter(|field| field.category == category)
        {
            if category == Category::General {
                let section = match field.id {
                    "general.language" => Some("settings-section-appearance"),
                    "general.reopen_last_document" => Some("settings-section-startup"),
                    "general.recent_files_limit" => Some("recent"),
                    _ => None,
                };
                if let Some(section) = section {
                    content = content.child(self.settings_section(l.text(section), p));
                }
            }
            if category == Category::Displays {
                continue;
            }
            content = content.child(self.setting_row(*field, enabled, p));
            if field.id == "general.ui_font_family" && data.panel.font_picker.is_some() {
                content = content.child(self.font_picker(enabled, p));
            }
            if field.id == "general.ui_font_size" {
                content = content.child(
                    div()
                        .px(px(12.))
                        .py(px(12.))
                        .mb(px(8.))
                        .bg(rgb(p.soft))
                        .rounded(px(theme::RADIUS))
                        .child(l.text("settings-font-preview"))
                        .child(
                            div()
                                .mt(px(4.))
                                .text_size(px(self.ui_text_size(12.)))
                                .text_color(rgb(p.muted))
                                .child(l.text("settings-font-scope")),
                        ),
                );
            }
        }
        match category {
            Category::General => {
                let files = &data.settings.general.recent_files;
                content = content.child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap(px(12.))
                        .py(px(4.))
                        .child(
                            div()
                                .text_size(px(self.ui_text_size(12.)))
                                .text_color(rgb(p.muted))
                                .child(l.format(
                                    "settings-recent-count",
                                    &[
                                        ("count", files.len().to_string()),
                                        (
                                            "limit",
                                            data.settings.general.recent_files_limit.to_string(),
                                        ),
                                    ],
                                )),
                        )
                        .child(self.button(
                            "settings-clear-recent".into(),
                            l.text("settings-clear-recent").into(),
                            enabled && !files.is_empty(),
                            false,
                            Intent::ClearRecent,
                            p,
                        )),
                );
                if files.is_empty() {
                    content = content.child(
                        div()
                            .py(px(16.))
                            .text_color(rgb(p.muted))
                            .child(l.text("recent-empty")),
                    );
                }
                for (index, path) in files.iter().enumerate() {
                    let (name, parent) = settings_panel::recent_copy(path);
                    let tooltip_path = path.clone();
                    let typography = self.data.typography.clone();
                    content = content.child(
                        div()
                            .id(SharedString::from(format!("settings-recent-{index}")))
                            .debug_selector(move || format!("settings-recent-{index}"))
                            .role(Role::ListItem)
                            .aria_label(path.clone())
                            .tab_index(0)
                            .tab_stop(true)
                            .border(px(theme::FOCUS_WIDTH))
                            .border_color(rgb(p.canvas))
                            .focus(|style| style.border_color(rgb(p.focus)))
                            .tooltip(move |_, cx| {
                                cx.new(|_| FilenameTooltip {
                                    filename: tooltip_path.clone(),
                                    palette: p,
                                    typography: typography.clone(),
                                })
                                .into()
                            })
                            .min_h(px(theme::SETTINGS_ROW))
                            .flex_shrink_0()
                            .py(px(8.))
                            .px(px(8.))
                            .flex()
                            .flex_col()
                            .gap(px(4.))
                            .child(div().text_color(rgb(p.ink)).child(name))
                            .child(
                                div()
                                    .id(SharedString::from(format!(
                                        "settings-recent-parent-{index}"
                                    )))
                                    .debug_selector(move || {
                                        format!("settings-recent-parent-{index}")
                                    })
                                    .text_size(px(self.ui_text_size(12.)))
                                    .text_color(rgb(p.muted))
                                    .whitespace_normal()
                                    .child(parent),
                            ),
                    );
                }
            }
            Category::Displays => {
                if let Some(panel) = &self.data.displays {
                    let width = (f32::from(window.viewport_size().width)
                        - if narrow { 32. } else { 220. })
                    .min(theme::SETTINGS_DISPLAY_WIDTH);
                    content = content.child(crate::display_settings::view::panel(
                        self, panel, l, p, true, width,
                    ));
                }
                for field in settings_panel::FIELDS
                    .iter()
                    .filter(|field| field.category == Category::Displays)
                {
                    content = content.child(self.setting_row(*field, enabled, p));
                }
            }
            Category::Pointer => {
                let pointer = &data.settings.pointer;
                let rgba = |color: [u8; 4], alpha: u8| {
                    gpui::rgba(
                        (u32::from(color[2]) << 24)
                            | (u32::from(color[1]) << 16)
                            | (u32::from(color[0]) << 8)
                            | u32::from(alpha),
                    )
                };
                content = content.child(
                    div()
                        .id("pointer-live-preview")
                        .h(px(64.))
                        .p(px(12.))
                        .bg(rgb(p.card))
                        .flex()
                        .items_center()
                        .gap(px(20.))
                        .child(
                            div()
                                .size(px((pointer.laser_radius_norm as f32 * 360.).max(2.)))
                                .rounded_full()
                                .bg(rgba(pointer.laser_color_bgra, pointer.laser_color_bgra[3])),
                        )
                        .child(
                            div()
                                .w(px(70.))
                                .h(px((pointer.pen_width_norm as f32 * 180.).max(1.)))
                                .bg(rgba(pointer.pen_color_bgra, pointer.pen_color_bgra[3])),
                        )
                        .child(div().w(px(70.)).h(px(24.)).bg(rgba(
                            pointer.pen_color_bgra,
                            (pointer.highlighter_alpha * 255.).round() as u8,
                        ))),
                );
            }
            Category::Keys => {
                let accepted = crate::policy::bindings(&data.settings.keybindings, l).0;
                for (action, keys) in data.settings.keybindings.actions() {
                    let mut row = div()
                        .flex()
                        .flex_wrap()
                        .justify_between()
                        .items_center()
                        .gap(px(4.))
                        .p(px(12.))
                        .border_b_1()
                        .border_color(rgb(p.card));
                    row = row.child(
                        div()
                            .text_color(rgb(p.ink))
                            .child(l.text(&format!("settings-action-{action}"))),
                    );
                    let mut controls = div().flex().flex_wrap().gap(px(4.));
                    for (index, key) in keys.iter().enumerate() {
                        let valid = crate::policy::canonical_key(key).is_some_and(|key| {
                            accepted.iter().any(|(k, a)| k == &key && *a == action)
                        });
                        controls = controls.child(self.button(
                            format!("binding-{action}-{index}"),
                            format!("{key} ×"),
                            enabled,
                            false,
                            Intent::RemoveBinding(action, index),
                            p,
                        ));
                        if !valid {
                            row = row.child(
                                div()
                                    .id(SharedString::from(format!(
                                        "binding-warning-{action}-{index}"
                                    )))
                                    .role(Role::Alert)
                                    .text_size(px(self.data.typography.size(14.)))
                                    .text_color(rgb(p.warning))
                                    .child(
                                        l.format("settings-key-conflict", &[("key", key.clone())]),
                                    ),
                            );
                        }
                    }
                    controls = controls.child(
                        self.button(
                            format!("record-{action}"),
                            l.text(if data.panel.recording == Some(action) {
                                "settings-recording"
                            } else {
                                "settings-record"
                            })
                            .into(),
                            enabled,
                            data.panel.recording == Some(action),
                            Intent::RecordBinding(action),
                            p,
                        ),
                    );
                    content = content.child(row.child(controls));
                }
                content = content
                    .child(
                        div()
                            .text_size(px(self.data.typography.size(14.)))
                            .text_color(rgb(p.muted))
                            .child(l.text("settings-key-help")),
                    )
                    .child(self.button(
                        "settings-reset-keys".into(),
                        l.text("settings-reset-keys").into(),
                        enabled,
                        false,
                        Intent::ResetBindings,
                        p,
                    ));
            }
            Category::Advanced => {
                content = content
                    .child(
                        div()
                            .text_size(px(self.data.typography.size(14.)))
                            .text_color(rgb(p.muted))
                            .child(l.text("settings-cache-help")),
                    )
                    .child(
                        div()
                            .p(px(12.))
                            .bg(rgb(p.soft))
                            .child(l.text("settings-hot-reload-help")),
                    )
                    .child(
                        div()
                            .text_size(px(self.data.typography.size(14.)))
                            .whitespace_normal()
                            .child(data.path.clone()),
                    )
                    .child(self.button(
                        "settings-open-directory".into(),
                        l.text("settings-open-directory").into(),
                        enabled,
                        false,
                        Intent::OpenConfigDirectory,
                        p,
                    ))
                    .child(
                        self.button(
                            "settings-reset-all".into(),
                            l.text(if data.panel.confirm_reset {
                                "settings-confirm-reset"
                            } else {
                                "settings-reset-all"
                            })
                            .into(),
                            enabled,
                            data.panel.confirm_reset,
                            Intent::ResetPreferences(data.panel.confirm_reset),
                            p,
                        ),
                    );
            }
            Category::About => {
                let text = |value: String| {
                    div()
                        .text_size(px(self.ui_text_size(14.)))
                        .text_color(rgb(p.muted))
                        .child(value)
                        .into_any_element()
                };
                let built_at = crate::about::build_info()
                    .and_then(|build| chrono::DateTime::from_timestamp(build.built_at, 0))
                    .filter(|time| time.timestamp() > 0)
                    .map(|time| time.with_timezone(&chrono::Local));
                let built_at = match built_at {
                    Some(time) => div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(text(
                            time.format(l.text("about-build-time-format")).to_string(),
                        ))
                        .child(
                            div()
                                .px(px(6.))
                                .rounded(px(4.))
                                .border_1()
                                .border_color(rgb(p.border))
                                .text_size(px(self.ui_text_size(12.)))
                                .text_color(rgb(p.muted))
                                .child(time.format("UTC%:z").to_string()),
                        )
                        .into_any_element(),
                    None => text(l.text("about-unknown").into()),
                };
                let repository = crate::about::REPOSITORY;
                let repository = self
                    .activation_target(
                        "about-repository-link".into(),
                        repository.into(),
                        true,
                        Intent::OpenRepository,
                        p,
                    )
                    .role(Role::Link)
                    .rounded(px(4.))
                    .text_size(px(self.ui_text_size(14.)))
                    .text_color(rgb(p.focus))
                    .hover(|style| style.underline())
                    .child(repository)
                    .into_any_element();
                for (key, value) in [
                    ("about-version", text(crate::about::VERSION.to_owned())),
                    (
                        "about-author",
                        text(crate::about::AUTHORS.replace(':', ", ")),
                    ),
                    ("about-build-time", built_at),
                    ("about-license", text(crate::about::LICENSE.to_owned())),
                    ("about-repository", repository),
                ] {
                    content = content.child(
                        div()
                            .id(SharedString::from(key))
                            .debug_selector(move || key.into())
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .justify_between()
                            .gap(px(12.))
                            .py(px(12.))
                            .border_b_1()
                            .border_color(rgb(p.border))
                            .child(
                                div()
                                    .text_size(px(self.ui_text_size(16.)))
                                    .text_color(rgb(p.ink))
                                    .child(l.text(key)),
                            )
                            .child(value),
                    );
                }
            }
            _ => {}
        }
        let mut footer = div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .border_t_1()
            .border_color(rgb(p.border))
            .pt(px(8.));
        if !data.writable {
            footer = footer.child(
                div()
                    .id("settings-readonly")
                    .role(Role::Alert)
                    .text_color(rgb(p.warning))
                    .child(l.text("settings-readonly")),
            );
        }
        if let Some(error) = &data.panel.error {
            footer = footer.child(
                div()
                    .id("settings-error")
                    .role(Role::Alert)
                    .text_color(rgb(p.error))
                    .child(l.error("settings-invalid", error)),
            );
        }
        if let Some(error) = &self.data.save_error {
            footer = footer.child(
                div()
                    .id("settings-save-error")
                    .role(Role::Alert)
                    .text_color(rgb(p.error))
                    .child(error.clone()),
            );
        }
        footer = footer.child(
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .text_size(px(self.data.typography.size(14.)))
                        .text_color(rgb(p.muted))
                        .child(l.text(if self.data.busy {
                            "settings-applying"
                        } else if self.data.dirty {
                            "settings-unsaved"
                        } else {
                            if self
                                .data
                                .displays
                                .as_ref()
                                .is_some_and(|panel| panel.roles.pending)
                            {
                                "settings-saved-display-pending"
                            } else {
                                "settings-saved"
                            }
                        })),
                )
                .when(self.data.dirty, |v| {
                    v.child(self.button(
                        "settings-retry".into(),
                        l.text("retry-save").into(),
                        enabled,
                        false,
                        Intent::RetrySave,
                        p,
                    ))
                }),
        );
        div()
            .size_full()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                div()
                    .text_size(px(self.data.typography.size(24.)))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(p.ink))
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(l.text("settings"))
                    .child(self.button(
                        "close-settings".into(),
                        l.text("close").into(),
                        true,
                        false,
                        Intent::Escape,
                        p,
                    )),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .gap(px(16.))
                    .when(narrow, |v| v.flex_col())
                    .child(rail)
                    .child(content),
            )
            .child(
                div()
                    .when(!narrow, |v| v.ml(px(192.)))
                    .max_w(px(theme::SETTINGS_WIDTH))
                    .child(footer),
            )
    }
    fn setting_row(&self, field: Field, enabled: bool, p: Palette) -> impl IntoElement {
        let data = &self.data.preferences;
        let l = self.locale;
        let value = settings_panel::value(&data.settings, field);
        let label = l.text(&format!("settings-field-{}", field.id.replace('.', "-")));
        let mut controls = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(0.))
            .bg(rgb(p.soft))
            .rounded(px(theme::RADIUS))
            .flex_shrink_0()
            .max_w_full();
        match field.kind {
            Kind::FontFamily => {
                controls = controls
                    .child(
                        self.button(
                            "setting-font-family".into(),
                            if value.is_empty() {
                                l.text("settings-font-default").into()
                            } else {
                                value.clone()
                            },
                            enabled,
                            data.panel.font_picker.is_some(),
                            Intent::ToggleFontPicker,
                            p,
                        )
                        .track_focus(&self.font_picker_focus.clone().tab_stop(enabled)),
                    )
                    .child(self.button(
                        "setting-font-default".into(),
                        l.text("settings-restore-default").into(),
                        enabled && !value.is_empty(),
                        false,
                        Intent::EditSetting(field.id, String::new()),
                        p,
                    ));
            }
            Kind::Toggle => {
                controls = controls.child(
                    self.button(
                        format!("setting-{}", field.id),
                        l.text(if value == "true" {
                            "settings-on"
                        } else {
                            "settings-off"
                        })
                        .into(),
                        enabled,
                        value == "true",
                        Intent::EditSetting(field.id, (value != "true").to_string()),
                        p,
                    ),
                )
            }
            Kind::Choice(choices) => {
                for choice in choices {
                    controls = controls.child(self.button(
                        format!("setting-{}-{choice}", field.id),
                        l.text(&format!("settings-choice-{choice}")).into(),
                        enabled,
                        value == *choice,
                        Intent::EditSetting(field.id, (*choice).into()),
                        p,
                    ));
                }
            }
            Kind::Number {
                min,
                max,
                step,
                integer,
            } => {
                let number = value.parse::<f64>().unwrap_or(min);
                controls = controls.child(self.button(
                    format!("setting-{}-minus", field.id),
                    "−".into(),
                    enabled && number > min,
                    false,
                    Intent::EditSetting(field.id, number_text((number - step).max(min), integer)),
                    p,
                ));
                controls = controls.child(self.value_editor(field, &value, enabled, p));
                controls = controls.child(self.button(
                    format!("setting-{}-plus", field.id),
                    "+".into(),
                    enabled && number < max,
                    false,
                    Intent::EditSetting(field.id, number_text((number + step).min(max), integer)),
                    p,
                ));
                if field.id == "general.ui_font_size" {
                    controls = controls.child(self.button(
                        "setting-font-size-default".into(),
                        l.text("settings-restore-default").into(),
                        enabled && number != 16.,
                        false,
                        Intent::EditSetting(field.id, "16".into()),
                        p,
                    ));
                }
                if matches!(
                    field.id,
                    "presenter.next_slide_ratio"
                        | "presenter.notes_ratio"
                        | "pointer.highlighter_alpha"
                        | "pointer.laser_radius_norm"
                        | "pointer.pen_width_norm"
                ) {
                    controls = controls.child(self.setting_slider(field, number, enabled, p));
                }
            }
            Kind::Color => {
                let rgba = u32::from_str_radix(&value, 16).unwrap_or(0);
                controls = controls
                    .child(
                        div()
                            .size(px(44.))
                            .border_1()
                            .border_color(rgb(p.border))
                            .bg(gpui::rgba(rgba)),
                    )
                    .child(self.value_editor(field, &value, enabled, p));
            }
        }
        let mut caption = div()
            .flex_1()
            .min_w(px(180.))
            .py(px(8.))
            .child(div().text_color(rgb(p.ink)).child(label));
        let help = match field.id {
            "general.language" | "display.presenter_start_fullscreen" => {
                Some("settings-restart-help")
            }
            "general.reopen_last_document" => Some("settings-startup-help"),
            "display.audience_fullscreen" => Some("settings-fullscreen-help"),
            "display.audience_click_to_advance" => Some("settings-click-to-advance-help"),
            _ => None,
        };
        if let Some(help) = help {
            let help_id = format!("setting-help-{}", field.id);
            caption = caption.child(
                div()
                    .id(SharedString::from(help_id.clone()))
                    .debug_selector(move || help_id.clone())
                    .mt(px(4.))
                    .text_size(px(self.ui_text_size(12.)))
                    .text_color(rgb(p.muted))
                    .child(l.text(help)),
            );
        }
        if field.id == "general.ui_font_family" && self.data.typography.missing {
            caption = caption.child(
                div()
                    .id("settings-font-fallback")
                    .role(Role::Alert)
                    .text_size(px(self.ui_text_size(12.)))
                    .text_color(rgb(p.warning))
                    .child(l.format("settings-font-missing", &[("family", value.clone())])),
            );
        }
        let row_id = format!("setting-row-{}", field.id);
        div()
            .id(SharedString::from(row_id.clone()))
            .debug_selector(move || row_id.clone())
            .min_h(px(theme::SETTINGS_ROW))
            .flex_shrink_0()
            .py(px(4.))
            .border_b_1()
            .border_color(rgb(p.card))
            .flex()
            .flex_wrap()
            .items_center()
            .justify_between()
            .gap(px(12.))
            .child(caption)
            .child(controls)
            .when(
                data.panel
                    .edit
                    .as_ref()
                    .is_some_and(|(id, _)| *id == field.id),
                |v| {
                    v.child(
                        div()
                            .w_full()
                            .text_size(px(self.ui_text_size(12.)))
                            .text_color(rgb(p.muted))
                            .child(l.text("settings-edit-help")),
                    )
                },
            )
    }
    fn settings_section(&self, label: &str, p: Palette) -> impl IntoElement {
        div()
            .pt(px(20.))
            .pb(px(8.))
            .text_size(px(self.ui_text_size(14.)))
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(rgb(p.muted))
            .child(label.to_string())
    }
    fn font_picker(&self, enabled: bool, p: Palette) -> impl IntoElement {
        let data = &self.data.preferences;
        let picker = data.panel.font_picker.as_ref().expect("open font picker");
        let matches = picker.matches(&data.fonts);
        let selected = picker.selected.min(matches.len() - 1);
        let start = selected / 6 * 6;
        let mut list = div()
            .id("settings-font-picker")
            .p(px(8.))
            .bg(rgb(p.soft))
            .rounded(px(theme::RADIUS))
            .border_1()
            .border_color(rgb(p.border))
            .flex()
            .flex_col()
            .items_start()
            .gap(px(0.))
            .child(
                div().px(px(12.)).py(px(8.)).child(
                    self.locale
                        .format("settings-font-filter", &[("filter", picker.filter.clone())]),
                ),
            );
        for (index, family) in matches.iter().enumerate().skip(start).take(6) {
            let family = family.unwrap_or("");
            let label = if family.is_empty() {
                self.locale.text("settings-font-default").to_string()
            } else {
                family.to_string()
            };
            list = list.child(self.button(
                format!("font-choice-{index}"),
                label,
                enabled,
                selected == index,
                Intent::EditSetting("general.ui_font_family", family.into()),
                p,
            ));
        }
        list.child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(self.button(
                    "font-previous".into(),
                    "↑".into(),
                    enabled && start > 0,
                    false,
                    Intent::FontPickerMove(-6),
                    p,
                ))
                .child(
                    div()
                        .text_size(px(self.ui_text_size(12.)))
                        .text_color(rgb(p.muted))
                        .child(format!(
                            "{}–{} / {}",
                            start + 1,
                            (start + 6).min(matches.len()),
                            matches.len()
                        )),
                )
                .child(self.button(
                    "font-next".into(),
                    "↓".into(),
                    enabled && start + 6 < matches.len(),
                    false,
                    Intent::FontPickerMove(6),
                    p,
                )),
        )
        .child(
            div()
                .px(px(12.))
                .py(px(8.))
                .text_size(px(self.ui_text_size(12.)))
                .text_color(rgb(p.muted))
                .child(self.locale.text("settings-font-key-help")),
        )
    }
    fn value_editor(
        &self,
        field: Field,
        value: &str,
        enabled: bool,
        p: Palette,
    ) -> impl IntoElement {
        let edit = self
            .data
            .preferences
            .panel
            .edit
            .as_ref()
            .filter(|(id, _)| *id == field.id);
        self.button(
            format!("setting-{}-value", field.id),
            edit.map_or_else(|| value.to_string(), |(_, text)| format!("{text} ▏")),
            enabled,
            edit.is_some(),
            Intent::BeginSettingEdit(field.id),
            p,
        )
    }
    fn setting_slider(
        &self,
        field: Field,
        value: f64,
        enabled: bool,
        p: Palette,
    ) -> impl IntoElement {
        let Kind::Number {
            min,
            max,
            step,
            integer,
        } = field.kind
        else {
            unreachable!()
        };
        let bounds = Rc::new(Cell::new(gpui::Bounds::<gpui::Pixels>::default()));
        let measured = bounds.clone();
        let target = self.shell.clone();
        let keyboard = target.clone();
        let drag_target = target.clone();
        let drag_bounds = bounds.clone();
        let dragging = self.settings_drag.clone();
        let move_dragging = dragging.clone();
        let up_dragging = dragging.clone();
        let out_dragging = dragging.clone();
        div()
            .id(SharedString::from(format!("slider-{}", field.id)))
            .debug_selector(move || format!("slider-{}", field.id))
            .role(Role::Slider)
            .key_context("FerroStageSettingsSlider")
            .aria_label(
                self.locale
                    .text(&format!("settings-field-{}", field.id.replace('.', "-"))),
            )
            .tab_index(0)
            .tab_stop(enabled)
            .w(px(200.))
            .h(px(44.))
            .relative()
            .border_1()
            .border_color(rgb(p.border))
            .rounded(px(theme::RADIUS))
            .focus(|v| v.border_color(rgb(p.focus)))
            .bg(rgb(p.card))
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top(px(19.))
                    .h(px(6.))
                    .w(relative(((value - min) / (max - min)).clamp(0., 1.) as f32))
                    .bg(rgb(theme::PRIMARY)),
            )
            .child(
                canvas(move |bounds, _, _| measured.set(bounds), |_, _, _, _| {})
                    .absolute()
                    .size_full(),
            )
            .on_mouse_down(MouseButton::Left, move |event, _, cx| {
                if enabled {
                    dragging.set(Some(field.id));
                    slider_edit(&target, field, bounds.get(), event.position.x, cx);
                }
            })
            .on_mouse_move(move |event, _, cx| {
                if enabled
                    && move_dragging.get() == Some(field.id)
                    && event.pressed_button == Some(MouseButton::Left)
                {
                    slider_edit(&drag_target, field, drag_bounds.get(), event.position.x, cx);
                }
            })
            .on_mouse_up(MouseButton::Left, move |_, _, _| up_dragging.set(None))
            .on_mouse_up_out(MouseButton::Left, move |_, _, _| out_dragging.set(None))
            .on_key_down(move |event, _, cx| {
                if !enabled {
                    return;
                }
                let next = match event.keystroke.key.as_str() {
                    "left" | "down" => (value - step).max(min),
                    "right" | "up" => (value + step).min(max),
                    "home" => min,
                    "end" => max,
                    _ => return,
                };
                actions::dispatch(
                    &keyboard,
                    Intent::EditSetting(field.id, number_text(next, integer)),
                    cx,
                );
                cx.stop_propagation();
            })
    }
}
fn number_text(value: f64, integer: bool) -> String {
    if integer {
        format!("{value:.0}")
    } else {
        format!("{value:.4}")
    }
}
fn slider_edit(
    target: &WeakEntity<Shell>,
    field: Field,
    bounds: gpui::Bounds<gpui::Pixels>,
    x: gpui::Pixels,
    cx: &mut gpui::App,
) {
    let Kind::Number {
        min,
        max,
        step,
        integer,
    } = field.kind
    else {
        return;
    };
    if bounds.size.width <= px(0.) {
        return;
    }
    let fraction = f64::from((x - bounds.origin.x) / bounds.size.width).clamp(0., 1.);
    let value = ((min + (max - min) * fraction) / step).round() * step;
    actions::dispatch(
        target,
        Intent::EditSetting(field.id, number_text(value.clamp(min, max), integer)),
        cx,
    );
}
