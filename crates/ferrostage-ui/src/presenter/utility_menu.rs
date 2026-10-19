use super::*;
use crate::workbench::Tool;

impl Presenter {
    fn utility_rows(&self) -> Vec<(&'static str, &'static str, Intent)> {
        if self.data.utility_menu == Some(true) {
            vec![
                ("pen", "wb-pen", Intent::Tool(Tool::Pen)),
                (
                    "highlighter",
                    "wb-highlighter",
                    Intent::Tool(Tool::Highlighter),
                ),
                ("eraser", "wb-eraser", Intent::Tool(Tool::Eraser)),
                (
                    "clear-strokes",
                    "wb-clear",
                    Intent::Core(ferrostage_core::PresentationCommand::ClearStrokes),
                ),
            ]
        } else {
            vec![
                ("home", "home", Intent::Home),
                ("open", "open", Intent::Open),
                ("settings", "settings", Intent::Settings),
            ]
        }
    }

    pub(super) fn utility_row_index(&self, id: &str) -> Option<usize> {
        self.data.utility_menu.and_then(|_| {
            self.utility_rows()
                .iter()
                .position(|(row, _, _)| *row == id)
        })
    }

    pub(super) fn utility_menu_key(
        &self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.data.utility_menu.is_none() {
            return false;
        }
        let rows = self.utility_rows();
        let enabled = rows
            .iter()
            .enumerate()
            .filter_map(|(index, (id, _, _))| {
                (!self.data.busy && (*id != "open" || self.data.can_open)).then_some(index)
            })
            .collect::<Vec<_>>();
        if enabled.is_empty() {
            return false;
        }
        let current = self
            .utility_focus
            .iter()
            .position(|f| f.is_focused(window))
            .unwrap_or(0)
            .min(rows.len() - 1);
        let enabled_position = enabled
            .iter()
            .position(|index| *index == current)
            .unwrap_or(0);
        let next = match event.keystroke.unparse().as_str() {
            "up" | "shift-tab" => {
                Some(enabled[(enabled_position + enabled.len() - 1) % enabled.len()])
            }
            "down" | "tab" => Some(enabled[(enabled_position + 1) % enabled.len()]),
            "home" => enabled.first().copied(),
            "end" => enabled.last().copied(),
            "escape" => {
                actions::dispatch(&self.shell, Intent::CloseContextual, cx);
                None
            }
            "enter" | "space" => {
                if !event.is_held
                    && !self.data.busy
                    && (rows[current].0 != "open" || self.data.can_open)
                {
                    actions::dispatch(&self.shell, rows[current].2.clone(), cx);
                }
                None
            }
            "left" | "right" | "pageup" | "pagedown" => None,
            _ => return false,
        };
        if let Some(index) = next {
            window.focus(&self.utility_focus[index], cx);
        }
        true
    }

    pub(super) fn utility_menu_panel(&self, p: Palette) -> gpui::AnyElement {
        let mut panel = div()
            .id("presenter-utility-menu")
            .debug_selector(|| "presenter-utility-menu".into())
            .w(px(240.))
            .p(px(8.))
            .rounded(px(theme::PANEL_RADIUS))
            .border_1()
            .border_color(rgb(p.border))
            .bg(rgb(p.card))
            .flex()
            .flex_col()
            .occlude();
        for (id, label, intent) in self.utility_rows() {
            let selected =
                matches!(&intent, Intent::Tool(tool) if *tool == self.data.workbench.tool);
            panel = panel.child(self.button(
                id.into(),
                self.locale.text(label).into(),
                !self.data.busy && (id != "open" || self.data.can_open),
                selected,
                intent,
                p,
            ));
        }
        panel.into_any_element()
    }
}
