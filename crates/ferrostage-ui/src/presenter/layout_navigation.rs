use super::*;
use ferrostage_document::{DocumentOverrides, DocumentWarning, NotesLayout};

const LAYOUTS: [(&str, Option<NotesLayout>); 2] =
    [("wb-auto", None), ("wb-unsplit", Some(NotesLayout::None))];
pub(super) const ROW_COUNT: usize = LAYOUTS.len() + 1;

fn layout_label(layout: NotesLayout) -> &'static str {
    match layout {
        NotesLayout::None => "wb-unsplit",
        NotesLayout::Left => "wb-left",
        NotesLayout::Right => "wb-right",
        NotesLayout::Top => "wb-top",
        NotesLayout::Bottom => "wb-bottom",
    }
}

pub(super) struct LayoutMenuData {
    id: u64,
    options: DocumentOverrides,
    detected: NotesLayout,
    annotations: bool,
    error: Option<String>,
}
impl LayoutMenuData {
    pub fn from_shell(shell: &Shell) -> Option<Self> {
        let crate::page_browser::ContextualSurface::LayoutNavigation { id, error, .. } =
            shell.contextual.as_ref()?
        else {
            return None;
        };
        let document = &shell.workbench.document.as_ref()?.document;
        Some(Self {
            id: *id,
            options: document.overrides,
            detected: document.layout,
            annotations: shell.state.has_annotations(),
            error: error.clone(),
        })
    }
    pub fn selected_layout(&self) -> usize {
        LAYOUTS
            .iter()
            .position(|(_, layout)| *layout == self.options.layout)
            .unwrap_or(0)
    }
    fn choice(&self, index: usize) -> DocumentOverrides {
        match index {
            0..2 => DocumentOverrides {
                layout: LAYOUTS[index].1,
                ..self.options
            },
            _ => DocumentOverrides {
                identify_independent_notes: !self.options.identify_independent_notes,
                ..self.options
            },
        }
    }
}

/// Real document warnings stay in ordinary feedback, without an internal Debug dump.
pub(super) fn document_warnings(shell: &Shell) -> Vec<String> {
    let Some(session) = &shell.workbench.document else {
        return vec![];
    };
    let l = shell.locale;
    let mut warnings = session
        .document
        .warnings
        .iter()
        .map(|warning| match warning {
            DocumentWarning::AmbiguousLayout => l.text("ln-warning-ambiguous").into(),
            DocumentWarning::MixedGeometry => l.text("ln-warning-mixed").into(),
            DocumentWarning::OverlayHeuristic => l.text("ln-warning-overlays").into(),
            DocumentWarning::OrphanNotes { page } => {
                l.format("ln-warning-orphan", &[("page", (page.0 + 1).to_string())])
            }
            DocumentWarning::MediaOutsideSlide { page, .. } => {
                l.format("ln-warning-media", &[("page", (page.0 + 1).to_string())])
            }
        })
        .collect::<Vec<_>>();
    for diagnostic in &session.document.structure_diagnostics {
        use ferrostage_document::DiagnosticKind;
        let kind = match diagnostic.kind {
            DiagnosticKind::Missing => "ln-diagnostic-missing",
            DiagnosticKind::Malformed => "ln-diagnostic-malformed",
            DiagnosticKind::Unsupported => "ln-diagnostic-unsupported",
            DiagnosticKind::UnscaledTime => "ln-diagnostic-time",
        };
        warnings.push(l.format(
            "ln-warning-structure",
            &[
                ("page", (diagnostic.page.0 + 1).to_string()),
                ("reason", l.text(kind).into()),
            ],
        ));
    }
    warnings.sort();
    warnings.dedup();
    warnings
}

impl Presenter {
    #[cfg(all(test, feature = "test-support"))]
    pub(crate) fn layout_focus_probe(&self, window: &Window) -> (Option<usize>, bool) {
        (
            self.layout_row_focus
                .iter()
                .position(|focus| focus.is_focused(window)),
            self.layout_trigger_focus.is_focused(window),
        )
    }
    pub(super) fn layout_menu_key(
        &self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(menu) = &self.data.layout_menu else {
            return false;
        };
        let key = event.keystroke.unparse();
        let current = self
            .layout_row_focus
            .iter()
            .position(|focus| focus.is_focused(window))
            .unwrap_or(menu.selected_layout());
        let next = match key.as_str() {
            "up" | "shift-tab" => Some((current + ROW_COUNT - 1) % ROW_COUNT),
            "down" | "tab" => Some((current + 1) % ROW_COUNT),
            "home" => Some(0),
            "end" => Some(ROW_COUNT - 1),
            "enter" | "space" => {
                if !self.data.busy && !event.is_held {
                    actions::dispatch(
                        &self.shell,
                        Intent::Reconfigure {
                            id: menu.id,
                            options: menu.choice(current),
                        },
                        cx,
                    );
                }
                None
            }
            "escape" => {
                actions::dispatch(&self.shell, Intent::CloseContextual, cx);
                None
            }
            "left" | "right" | "pageup" | "pagedown" => None,
            _ => return false,
        };
        if let Some(index) = next {
            window.focus(&self.layout_row_focus[index], cx);
        }
        true
    }

    pub(super) fn layout_menu_panel(&self, p: Palette, window: &Window) -> gpui::AnyElement {
        let Some(menu) = &self.data.layout_menu else {
            return div().into_any_element();
        };
        let l = self.locale;
        let width = 320f32
            .min(f32::from(window.viewport_size().width) - 24.)
            .max(200.);
        let available = (f32::from(self.layout_anchor.get().top()) - 20.).max(120.);
        let detected = layout_label(menu.detected);
        let mut body = div()
            .id("layout-menu-body")
            .debug_selector(|| "layout-menu-body".into())
            .overflow_y_scroll()
            .track_scroll(&self.layout_scroll)
            .min_h_0()
            .flex()
            .flex_col()
            .gap(px(2.))
            .child(self.layout_group_heading("ln-layout-group", p));
        for (index, focus) in self.layout_row_focus.iter().enumerate() {
            if index == LAYOUTS.len() {
                body = body
                    .child(div().my(px(6.)).h(px(1.)).bg(rgb(p.border)).flex_shrink_0())
                    .child(self.layout_group_heading("ln-page-group", p));
            }
            let (label, checked, helper) = match index {
                0 => (
                    l.text("wb-auto"),
                    menu.options.layout.is_none(),
                    Some(l.format("ln-detected", &[("layout", l.text(detected).into())])),
                ),
                1 => (
                    l.text(LAYOUTS[index].0),
                    menu.selected_layout() == index,
                    None,
                ),
                _ => (
                    l.text("wb-independent"),
                    menu.options.identify_independent_notes,
                    Some(l.text("ln-independent-help").into()),
                ),
            };
            let accessible_label = helper
                .as_ref()
                .map_or_else(|| label.to_string(), |helper| format!("{label}. {helper}"));
            let id = format!("layout-row-{index}");
            body = body.child(
                self.activation_target(
                    id,
                    accessible_label,
                    !self.data.busy,
                    Intent::Reconfigure {
                        id: menu.id,
                        options: menu.choice(index),
                    },
                    p,
                )
                .track_focus(&focus.clone().tab_stop(!self.data.busy))
                .anchor_scroll(Some(self.layout_row_anchors[index].clone()))
                .aria_selected(checked)
                .role(if index < LAYOUTS.len() {
                    Role::MenuItemRadio
                } else {
                    Role::MenuItemCheckBox
                })
                .aria_toggled(if checked {
                    gpui::Toggled::True
                } else {
                    gpui::Toggled::False
                })
                .flex_shrink_0()
                .child(
                    div()
                        .id(SharedString::from(format!("layout-row-{index}-face")))
                        .relative()
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
                        .active(|style| style.bg(rgb(p.soft)).top(px(1.)))
                        .child(
                            div()
                                .w(px(16.))
                                .flex_shrink_0()
                                .when(checked, |view| {
                                    view.debug_selector(move || format!("layout-check-{index}"))
                                })
                                .child(if checked { "✓" } else { "" }),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .text_size(px(self.ui_text_size(14.)))
                                .child(label)
                                .when_some(helper, |view, helper| {
                                    view.child(
                                        div()
                                            .text_size(px(self.ui_text_size(12.)))
                                            .text_color(rgb(p.muted))
                                            .child(helper),
                                    )
                                }),
                        ),
                ),
            );
        }
        div()
            .id("layout-navigation-menu")
            .debug_selector(|| "layout-navigation-menu".into())
            .role(Role::Menu)
            .aria_label(l.text("wb-document"))
            .occlude()
            .w(px(width))
            .max_h(px(available))
            .p(px(12.))
            .rounded(px(12.))
            .border_1()
            .border_color(rgb(p.border))
            .bg(rgb(p.card))
            .flex()
            .flex_col()
            .gap(px(8.))
            .on_mouse_down(gpui::MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .child(body)
            .when(menu.annotations, |view| {
                view.child(
                    div()
                        .id("layout-annotation-notice")
                        .debug_selector(|| "layout-annotation-notice".into())
                        .flex_shrink_0()
                        .text_size(px(self.ui_text_size(12.)))
                        .text_color(rgb(p.muted))
                        .child(l.text("ln-clear-notice")),
                )
            })
            .when_some(menu.error.as_ref(), |view, error| {
                view.child(
                    div()
                        .id("layout-menu-error")
                        .debug_selector(|| "layout-menu-error".into())
                        .flex_shrink_0()
                        .text_size(px(self.ui_text_size(12.)))
                        .text_color(rgb(p.error))
                        .child(error.clone()),
                )
            })
            .into_any_element()
    }
    fn layout_group_heading(&self, key: &str, p: Palette) -> Div {
        div()
            .px(px(12.))
            .py(px(4.))
            .flex_shrink_0()
            .text_size(px(self.ui_text_size(12.)))
            .text_color(rgb(p.muted))
            .child(self.locale.text(key))
    }
}
