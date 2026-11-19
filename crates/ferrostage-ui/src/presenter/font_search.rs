use super::*;
use crate::locale::Locale;
use gpui::{Bounds, EntityInputHandler, MouseButton, Pixels, Point, ShapedLine, UTF16Selection};
use std::{cell::RefCell, ops::Range, rc::Rc};

/// GPUI shapes text with UTF-8 byte indices. The platform input handler speaks UTF-16.
fn byte_offset(text: &str, utf16: usize) -> usize {
    let mut units = 0;
    for (byte, ch) in text.char_indices() {
        units += ch.len_utf16();
        if units > utf16 {
            return byte;
        }
    }
    text.len()
}

fn byte_range(text: &str, range: Range<usize>) -> Range<usize> {
    byte_offset(text, range.start)..byte_offset(text, range.end.max(range.start))
}

fn utf16_range(text: &str, bytes: Range<usize>) -> Range<usize> {
    let start = text[..bytes.start].encode_utf16().count();
    start..start + text[bytes].encode_utf16().count()
}

#[derive(Default)]
struct Query {
    text: String,
    anchor: usize,
    cursor: usize,
    marked: Option<Range<usize>>,
}

impl Query {
    fn selection(&self) -> Range<usize> {
        self.anchor.min(self.cursor)..self.anchor.max(self.cursor)
    }

    fn select(&mut self, offset: usize, extend: bool) {
        self.cursor = offset;
        if !extend {
            self.anchor = offset;
        }
    }

    fn adjacent(&self, backward: bool) -> usize {
        if backward {
            self.text.floor_char_boundary(self.cursor.saturating_sub(1))
        } else {
            self.text.ceil_char_boundary(self.cursor + 1)
        }
    }

    fn replace(&mut self, range: Option<Range<usize>>, input: &str) -> Option<Range<usize>> {
        let bytes = range
            .map(|range| byte_range(&self.text, range))
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection());
        if input.len() > 128 - (self.text.len() - bytes.len())
            || input.chars().any(char::is_control)
        {
            return None;
        }
        let inserted = bytes.start..bytes.start + input.len();
        self.text.replace_range(bytes, input);
        self.marked = None;
        self.select(inserted.end, false);
        Some(inserted)
    }

    fn mark(
        &mut self,
        range: Option<Range<usize>>,
        input: &str,
        selection: Option<Range<usize>>,
    ) -> bool {
        let Some(inserted) = self.replace(range, input) else {
            return false;
        };
        if let Some(selection) = selection {
            let relative = byte_range(&self.text[inserted.clone()], selection);
            self.anchor = inserted.start + relative.start;
            self.cursor = inserted.start + relative.end;
        }
        self.marked = (!inserted.is_empty()).then_some(inserted);
        true
    }
}

struct Layout {
    line: ShapedLine,
    bounds: Bounds<Pixels>,
    scroll: Pixels,
}

impl Layout {
    fn x(&self, byte: usize) -> Pixels {
        self.bounds.left() - self.scroll + self.line.x_for_index(byte)
    }

    fn range_bounds(&self, range: Range<usize>) -> Bounds<Pixels> {
        Bounds::from_corners(
            gpui::point(self.x(range.start), self.bounds.top()),
            gpui::point(self.x(range.end), self.bounds.bottom()),
        )
    }
}

pub(super) struct FontSearch {
    shell: WeakEntity<Shell>,
    focus: FocusHandle,
    locale: Locale,
    palette: Palette,
    text_size: f32,
    query: Query,
    layout: Rc<RefCell<Option<Layout>>>,
    selecting: bool,
}

impl FontSearch {
    pub(super) fn new(
        shell: WeakEntity<Shell>,
        focus: FocusHandle,
        locale: Locale,
        palette: Palette,
        text_size: f32,
    ) -> Self {
        Self {
            shell,
            focus,
            locale,
            palette,
            text_size,
            query: Query::default(),
            layout: Rc::default(),
            selecting: false,
        }
    }

    pub(super) fn composing(&self) -> bool {
        self.query.marked.is_some()
    }

    pub(super) fn update_style(&mut self, palette: Palette, size: f32, cx: &mut Context<Self>) {
        if self.palette != palette || self.text_size != size {
            self.palette = palette;
            self.text_size = size;
            cx.notify();
        }
    }

    fn changed(&self, cx: &mut Context<Self>) {
        actions::dispatch(
            &self.shell,
            Intent::FilterFonts(self.query.text.clone()),
            cx,
        );
        cx.notify();
    }

    fn byte_at(&self, position: Point<Pixels>) -> Option<usize> {
        let layout = self.layout.borrow();
        let layout = layout.as_ref()?;
        let byte = layout
            .line
            .closest_index_for_x(position.x - layout.bounds.left() + layout.scroll);
        Some(self.query.text.floor_char_boundary(byte))
    }

    fn key_down(
        &mut self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.composing() {
            return;
        }
        let key = event.keystroke.unparse();
        let shift = event.keystroke.modifiers.shift;
        let edit = &mut self.query;
        let range = edit.selection();
        match key.as_str() {
            "ctrl-a" | "cmd-a" => {
                edit.anchor = 0;
                edit.cursor = edit.text.len();
            }
            "home" | "shift-home" => edit.select(0, shift),
            "end" | "shift-end" => edit.select(edit.text.len(), shift),
            "left" | "shift-left" | "right" | "shift-right" => {
                let backward = event.keystroke.key == "left";
                let next = if shift || range.is_empty() {
                    edit.adjacent(backward)
                } else if backward {
                    range.start
                } else {
                    range.end
                };
                edit.select(next, shift);
            }
            "ctrl-c" | "cmd-c" | "ctrl-x" | "cmd-x" => {
                if !range.is_empty() {
                    cx.write_to_clipboard(gpui::ClipboardItem::new_string(edit.text[range].into()));
                    if key.ends_with('x') {
                        edit.replace(None, "");
                        self.changed(cx);
                    }
                }
            }
            "ctrl-v" | "cmd-v" | "shift-insert" => {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text())
                    && edit.replace(None, &text).is_some()
                {
                    self.changed(cx);
                }
            }
            "backspace" | "delete" => {
                if range.is_empty() {
                    edit.anchor = edit.adjacent(key == "backspace");
                }
                if !edit.selection().is_empty() {
                    edit.replace(None, "");
                    self.changed(cx);
                }
            }
            _ => return,
        }
        window.prevent_default();
        cx.stop_propagation();
        cx.notify();
    }

    fn text_line(&self, cx: &Context<Self>) -> impl IntoElement + use<> {
        let p = self.palette;
        let edit = &self.query;
        let text: SharedString = if edit.text.is_empty() {
            self.locale.text("settings-font-placeholder").into()
        } else {
            edit.text.clone().into()
        };
        let color = if edit.text.is_empty() { p.muted } else { p.ink };
        let cursor = edit.cursor;
        let selection = edit.selection();
        let marked = edit.marked.clone();
        let previous_scroll = self
            .layout
            .borrow()
            .as_ref()
            .map_or(px(0.), |layout| layout.scroll);
        let saved = self.layout.clone();
        let focus = self.focus.clone();
        let entity = cx.entity();
        gpui::canvas(
            move |bounds, window, _| {
                let style = window.text_style();
                let run = gpui::TextRun {
                    len: text.len(),
                    font: style.font(),
                    color: rgb(color).into(),
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                let line = window.text_system().shape_line(
                    text,
                    style.font_size.to_pixels(window.rem_size()),
                    &[run],
                    None,
                );
                let caret = line.x_for_index(cursor);
                let visible_width = (bounds.size.width - px(2.)).max(px(0.));
                let scroll = previous_scroll
                    .min((line.width() - visible_width).max(px(0.)))
                    .min(caret)
                    .max(caret - visible_width);
                Layout {
                    line,
                    bounds,
                    scroll,
                }
            },
            move |bounds, layout, window, cx| {
                window.handle_input(&focus, gpui::ElementInputHandler::new(bounds, entity), cx);
                window.with_content_mask(Some(gpui::ContentMask { bounds }), |window| {
                    if focus.is_focused(window) && !selection.is_empty() {
                        let mut tint: gpui::Hsla = rgb(p.focus).into();
                        tint.a = 0.25;
                        window.paint_quad(gpui::fill(layout.range_bounds(selection), tint));
                    }
                    let origin = gpui::point(bounds.left() - layout.scroll, bounds.top());
                    let _ = layout.line.paint(
                        origin,
                        bounds.size.height,
                        gpui::TextAlign::Left,
                        None,
                        window,
                        cx,
                    );
                    if let Some(marked) = marked {
                        let mut underline = layout.range_bounds(marked);
                        underline.origin.y = underline.bottom() - px(1.);
                        underline.size.height = px(1.);
                        window.paint_quad(gpui::fill(underline, rgb(p.ink)));
                    }
                    if focus.is_focused(window) {
                        let mut caret = layout.range_bounds(cursor..cursor);
                        caret.size.width = px(2.);
                        window.paint_quad(gpui::fill(caret, rgb(p.focus)));
                    }
                });
                *saved.borrow_mut() = Some(layout);
            },
        )
        .w_full()
        .h(px(self.text_size * 1.45))
    }
}

impl Render for FontSearch {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let p = self.palette;
        div()
            .id("settings-font-search")
            .accessibility_id("settings-font-search")
            .debug_selector(|| "settings-font-search".into())
            .role(Role::TextInput)
            .aria_label(self.locale.text("settings-font-search"))
            .aria_value(self.query.text.clone())
            .track_focus(&self.focus)
            .tab_index(0)
            .w_full()
            .flex_shrink_0()
            .px(px(10.))
            .h(px((self.text_size * 1.45 + 12.).max(theme::CONTROL)))
            .rounded(px(theme::WELL_RADIUS))
            .border_2()
            .border_color(rgb(p.border))
            .focus(|style| style.border_color(rgb(p.focus)))
            .bg(rgb(p.well))
            .text_size(px(self.text_size))
            .flex()
            .items_center()
            .cursor(gpui::CursorStyle::IBeam)
            .on_key_down(cx.listener(Self::key_down))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseDownEvent, window, cx| {
                    window.focus(&this.focus, cx);
                    this.selecting = true;
                    if event.click_count >= 2 {
                        this.query.anchor = 0;
                        this.query.cursor = this.query.text.len();
                    } else if let Some(byte) = this.byte_at(event.position) {
                        this.query.select(byte, event.modifiers.shift);
                    }
                    cx.notify();
                    cx.stop_propagation();
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                if this.selecting
                    && event.pressed_button == Some(MouseButton::Left)
                    && let Some(byte) = this.byte_at(event.position)
                {
                    this.query.select(byte, true);
                    cx.notify();
                }
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.selecting = false),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _, _, _| this.selecting = false),
            )
            .child(self.text_line(cx))
    }
}

impl EntityInputHandler for FontSearch {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let bytes = byte_range(&self.query.text, range);
        *actual = Some(utf16_range(&self.query.text, bytes.clone()));
        Some(self.query.text[bytes].into())
    }

    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: utf16_range(&self.query.text, self.query.selection()),
            reversed: self.query.cursor < self.query.anchor,
        })
    }

    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.query
            .marked
            .clone()
            .map(|range| utf16_range(&self.query.text, range))
    }

    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.query.marked = None;
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.query.replace(range, text).is_some() {
            self.changed(cx);
        }
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selection: Option<Range<usize>>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.query.mark(range, text, selection) {
            self.changed(cx);
        }
    }

    fn bounds_for_range(
        &mut self,
        range: Range<usize>,
        _: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let layout = self.layout.borrow();
        Some(
            layout
                .as_ref()?
                .range_bounds(byte_range(&self.query.text, range)),
        )
    }

    fn character_index_for_point(
        &mut self,
        point: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        Some(
            self.query.text[..self.byte_at(point)?]
                .encode_utf16()
                .count(),
        )
    }

    fn set_selected_text_range(
        &mut self,
        range: Range<usize>,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let range = byte_range(&self.query.text, range);
        self.query.anchor = range.start;
        self.query.cursor = range.end;
        cx.notify();
    }

    fn text_length_utf16(&mut self, _: &mut Window, _: &mut Context<Self>) -> Option<usize> {
        Some(self.query.text.encode_utf16().count())
    }

    fn accepts_text_input(&self, window: &mut Window, _: &mut Context<Self>) -> bool {
        self.focus.is_focused(window)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_updates_and_commits_inside_a_unicode_query() {
        let mut q = Query::default();
        q.replace(None, "字😀 Sans");
        q.mark(Some(1..3), "ni", Some(2..2));
        assert_eq!(q.text, "字ni Sans");
        q.mark(None, "你好", Some(1..2));
        assert_eq!(&q.text[q.selection()], "好");
        assert_eq!(utf16_range(&q.text, q.marked.clone().unwrap()), 1..3);
        q.replace(None, "你");
        assert_eq!(q.text, "字你 Sans");
        assert!(q.marked.is_none());
        assert_eq!(q.cursor, "字你".len());
    }

    #[test]
    fn rejected_paste_preserves_selection_and_byte_limit_accounts_for_replacement() {
        let mut q = Query::default();
        q.replace(None, "A😀B");
        q.cursor = q.adjacent(true);
        q.anchor = q.adjacent(true);
        assert_eq!(&q.text[q.selection()], "😀");
        assert!(q.replace(None, "宋\r\n体").is_none());
        assert_eq!(&q.text[q.selection()], "😀");
        assert!(q.replace(None, &"x".repeat(127)).is_none());
        assert_eq!(&q.text[q.selection()], "😀");
        assert!(q.replace(None, &"宋".repeat(42)).is_some());
        assert_eq!(q.text.len(), 128);
        assert!(q.replace(None, "x").is_none());
        q.select(q.text.len(), false);
        assert_eq!(q.adjacent(false), q.text.len());
    }
}
