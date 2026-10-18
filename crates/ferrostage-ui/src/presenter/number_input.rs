use super::*;
use crate::page_browser::NumberEdit;
use gpui::{Bounds, EntityInputHandler, Pixels, Point, UTF16Selection};
use std::ops::Range;

pub(super) type NumberLayout =
    std::rc::Rc<std::cell::RefCell<Option<(gpui::ShapedLine, gpui::Bounds<gpui::Pixels>)>>>;

impl Presenter {
    fn number_edit(&self) -> Option<&NumberEdit> {
        self.data.browser.as_ref().map(|data| &data.browser.edit)
    }
    fn update_number(&mut self, update: impl FnOnce(&mut NumberEdit), cx: &mut Context<Self>) {
        if let Some(data) = &mut self.data.browser {
            update(&mut data.browser.edit);
            actions::dispatch(
                &self.shell,
                Intent::BrowserEdit {
                    id: data.browser.id,
                    edit: data.browser.edit.clone(),
                },
                cx,
            );
            cx.notify();
        }
    }
    pub(super) fn number_key(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.data.browser.is_none() || !self.number_focus.is_focused(window) {
            return false;
        }
        if self.data.busy && event.keystroke.key != "escape" {
            return true;
        }
        let key = event.keystroke.unparse();
        let shift = event.keystroke.modifiers.shift;
        match key.as_str() {
            "tab" | "shift-tab" => {
                if shift {
                    window.focus_prev(cx);
                } else {
                    window.focus_next(cx);
                }
            }
            "escape" => actions::dispatch(&self.shell, Intent::CloseContextual, cx),
            "enter" => actions::dispatch(
                &self.shell,
                Intent::BrowserSubmit {
                    id: self.data.browser.as_ref().unwrap().browser.id,
                },
                cx,
            ),
            "ctrl-a" | "cmd-a" => self.update_number(
                |edit| {
                    edit.anchor = 0;
                    edit.cursor = edit.text.len();
                },
                cx,
            ),
            "ctrl-v" | "cmd-v" | "shift-insert" => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    self.update_number(
                        |edit| {
                            edit.replace(None, &text);
                        },
                        cx,
                    );
                }
            }
            "ctrl-c" | "cmd-c" | "ctrl-x" | "cmd-x" => {
                if let Some(edit) = self.number_edit() {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                        edit.text[edit.selection()].into(),
                    ));
                }
                if key.ends_with('x') {
                    self.update_number(
                        |edit| {
                            edit.replace(None, "");
                        },
                        cx,
                    );
                }
            }
            "backspace" => self.update_number(|edit| edit.delete(true), cx),
            "delete" => self.update_number(|edit| edit.delete(false), cx),
            "left" | "right" | "shift-left" | "shift-right" | "home" | "end" | "shift-home"
            | "shift-end" => {
                self.update_number(
                    |edit| {
                        let selection = edit.selection();
                        let offset = match event.keystroke.key.as_str() {
                            "home" => 0,
                            "end" => edit.text.len(),
                            "left" if !shift && !selection.is_empty() => selection.start,
                            "right" if !shift && !selection.is_empty() => selection.end,
                            "left" => edit.cursor.saturating_sub(1),
                            _ => (edit.cursor + 1).min(edit.text.len()),
                        };
                        edit.select(offset, shift);
                    },
                    cx,
                );
            }
            _ => {
                if !event.keystroke.modifiers.control
                    && !event.keystroke.modifiers.platform
                    && !event.keystroke.modifiers.alt
                {
                    let text = event
                        .keystroke
                        .key_char
                        .as_deref()
                        .unwrap_or(&event.keystroke.key);
                    if text.len() == 1 && text.as_bytes()[0].is_ascii_digit() {
                        self.update_number(
                            |edit| {
                                edit.replace(None, text);
                            },
                            cx,
                        );
                    }
                }
            }
        }
        true
    }
    pub(super) fn number_field(&self, p: Palette, cx: &Context<Self>) -> gpui::AnyElement {
        let Some(data) = &self.data.browser else {
            return div().into_any_element();
        };
        let edit = data.browser.edit.clone();
        let focus = self.number_focus.clone();
        let entity = cx.entity();
        let layout = self.number_layout.clone();
        let drag_layout = layout.clone();
        let down_layout = layout.clone();
        let selecting = self.number_selecting.clone();
        let down_selecting = selecting.clone();
        let up_selecting = selecting.clone();
        div()
            .id("page-number-input")
            .debug_selector(|| "page-number-input".into())
            .role(Role::TextInput)
            .aria_label(self.locale.text(
                if data.mode == crate::workbench::PageNumberMode::Content {
                    "pb-content-number"
                } else {
                    "pb-number"
                },
            ))
            .aria_value(edit.text.clone())
            .track_focus(&self.number_focus)
            .tab_index(0)
            .key_context("FerroStagePageInput")
            .w(px(140.))
            .h(px(44.))
            .flex_shrink_0()
            .px(px(12.))
            .flex()
            .items_center()
            .rounded(px(8.))
            .border_2()
            .border_color(rgb(p.border))
            .focus(|s| s.border_color(rgb(p.focus)))
            .bg(rgb(p.soft))
            .text_color(rgb(p.ink))
            .font(gpui::font(theme::MONO))
            .text_size(px(self.data.typography.size(20.)))
            .cursor(gpui::CursorStyle::IBeam)
            .overflow_hidden()
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(move |this, event: &gpui::MouseDownEvent, window, cx| {
                    window.focus(&this.number_focus, cx);
                    down_selecting.set(true);
                    if let Some((line, bounds)) = down_layout.borrow().as_ref() {
                        let offset = line.closest_index_for_x(event.position.x - bounds.origin.x);
                        this.update_number(
                            |edit| {
                                if event.click_count >= 2 {
                                    edit.anchor = 0;
                                    edit.cursor = edit.text.len();
                                } else {
                                    edit.select(offset, event.modifiers.shift);
                                }
                            },
                            cx,
                        );
                    }
                    cx.stop_propagation();
                }),
            )
            .on_mouse_move(
                cx.listener(move |this, event: &gpui::MouseMoveEvent, _, cx| {
                    if selecting.get()
                        && event.pressed_button == Some(gpui::MouseButton::Left)
                        && let Some((line, bounds)) = drag_layout.borrow().as_ref()
                    {
                        let offset = line.closest_index_for_x(event.position.x - bounds.origin.x);
                        this.update_number(|edit| edit.select(offset, true), cx);
                    }
                }),
            )
            .on_mouse_up(gpui::MouseButton::Left, move |_, _, _| {
                up_selecting.set(false)
            })
            .child(
                gpui::canvas(
                    move |bounds, window, _| {
                        let style = window.text_style();
                        let text: SharedString = edit.text.clone().into();
                        let line = window.text_system().shape_line(
                            text.clone(),
                            style.font_size.to_pixels(window.rem_size()),
                            &[gpui::TextRun {
                                len: text.len(),
                                font: style.font(),
                                color: style.color,
                                background_color: None,
                                underline: None,
                                strikethrough: None,
                            }],
                            None,
                        );
                        (line, edit, bounds)
                    },
                    move |_, (line, edit, bounds), window, cx| {
                        window.handle_input(
                            &focus,
                            gpui::ElementInputHandler::new(bounds, entity.clone()),
                            cx,
                        );
                        if focus.is_focused(window) {
                            let range = edit.selection();
                            let mut color: gpui::Hsla = rgb(p.focus).into();
                            color.a = 0.25;
                            if !range.is_empty() {
                                window.paint_quad(gpui::fill(
                                    Bounds::from_corners(
                                        gpui::point(
                                            bounds.left() + line.x_for_index(range.start),
                                            bounds.top(),
                                        ),
                                        gpui::point(
                                            bounds.left() + line.x_for_index(range.end),
                                            bounds.bottom(),
                                        ),
                                    ),
                                    color,
                                ));
                            }
                            window.paint_quad(gpui::fill(
                                Bounds::new(
                                    gpui::point(
                                        bounds.left() + line.x_for_index(edit.cursor),
                                        bounds.top(),
                                    ),
                                    gpui::size(px(2.), bounds.size.height),
                                ),
                                rgb(p.focus),
                            ));
                        }
                        let _ = line.paint(
                            bounds.origin,
                            bounds.size.height,
                            gpui::TextAlign::Left,
                            None,
                            window,
                            cx,
                        );
                        *layout.borrow_mut() = Some((line, bounds));
                    },
                )
                .w_full()
                .h(px(self.data.typography.size(26.))),
            )
            .into_any_element()
    }
}

impl EntityInputHandler for Presenter {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let edit = self.number_edit()?;
        let start = range.start.min(edit.text.len());
        let range = start..range.end.max(start).min(edit.text.len());
        *actual = Some(range.clone());
        Some(edit.text[range].into())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        self.number_edit().map(|edit| UTF16Selection {
            range: edit.selection(),
            reversed: edit.cursor < edit.anchor,
        })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        None
    }
    fn unmark_text(&mut self, _: &mut Window, _: &mut Context<Self>) {}
    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.update_number(
            |edit| {
                edit.replace(range, text);
            },
            cx,
        );
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        _: Option<Range<usize>>,
        _: &str,
        _: Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) {
        // A page number accepts committed ASCII digits, never IME composition.
    }
    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let layout = self.number_layout.borrow();
        let (line, bounds) = layout.as_ref()?;
        let len = self.number_edit()?.text.len();
        Some(Bounds::from_corners(
            gpui::point(
                bounds.left() + line.x_for_index(range.start.min(len)),
                bounds.top(),
            ),
            gpui::point(
                bounds.left() + line.x_for_index(range.end.min(len)),
                bounds.bottom(),
            ),
        ))
    }
    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        let layout = self.number_layout.borrow();
        let (line, bounds) = layout.as_ref()?;
        Some(line.closest_index_for_x(point.x - bounds.left()))
    }
    fn set_selected_text_range(
        &mut self,
        range: Range<usize>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.update_number(
            |edit| {
                edit.select(range.start, false);
                edit.select(range.end, true);
            },
            cx,
        );
    }
    fn text_length_utf16(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        self.number_edit().map(|edit| edit.text.len())
    }
    fn accepts_text_input(&self, window: &mut Window, _: &mut Context<Self>) -> bool {
        self.data.browser.is_some() && !self.data.busy && self.number_focus.is_focused(window)
    }
}
