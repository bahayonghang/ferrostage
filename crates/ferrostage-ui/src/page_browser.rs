//! Page-browser editing and grouping. Numeric offsets are ASCII/UTF-16 identical.
use ferrostage_core::{DocumentGeneration, PhysicalPage};
use std::ops::Range;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct NumberEdit {
    pub text: String,
    pub anchor: usize,
    pub cursor: usize,
}
impl NumberEdit {
    pub fn new(text: String) -> Self {
        let cursor = text.len();
        Self {
            text,
            anchor: 0,
            cursor,
        }
    }
    pub fn selection(&self) -> Range<usize> {
        self.anchor.min(self.cursor)..self.anchor.max(self.cursor)
    }
    pub fn select(&mut self, offset: usize, extend: bool) {
        self.cursor = offset.min(self.text.len());
        if !extend {
            self.anchor = self.cursor;
        }
    }
    pub fn replace(&mut self, range: Option<Range<usize>>, text: &str) -> bool {
        let text = text.trim();
        if !text.bytes().all(|c| c.is_ascii_digit()) {
            return false;
        }
        let range = range.unwrap_or_else(|| self.selection());
        let start = range.start.min(self.text.len());
        let end = range.end.max(start).min(self.text.len());
        if self.text.len() - (end - start) + text.len() > 10 {
            return false;
        }
        self.text.replace_range(start..end, text);
        self.select(start + text.len(), false);
        true
    }
    pub fn delete(&mut self, backwards: bool) {
        let mut range = self.selection();
        if range.is_empty() {
            if backwards {
                range.start = range.start.saturating_sub(1);
            } else {
                range.end = (range.end + 1).min(self.text.len());
            }
        }
        self.replace(Some(range), "");
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PageBrowser {
    pub id: u64,
    pub generation: DocumentGeneration,
    pub edit: NumberEdit,
    pub target: Option<PhysicalPage>,
    pub error: Option<&'static str>,
}

#[derive(Clone, Debug)]
pub(crate) enum ContextualSurface {
    PresenterMenu {
        tools: bool,
    },
    PageBrowser(PageBrowser),
    LayoutNavigation {
        id: u64,
        generation: DocumentGeneration,
        error: Option<String>,
    },
}

pub(crate) fn rectangular_rows(columns: u32, visible: u32) -> Option<u32> {
    (visible.is_multiple_of(columns) && visible / columns <= 4).then_some(visible / columns)
}

pub(crate) fn density(columns: u32, visible: u32, column: bool, value: u32) -> Option<(u32, u32)> {
    if !(1..=4).contains(&value) {
        return None;
    }
    let rows = rectangular_rows(columns, visible).unwrap_or(visible.div_ceil(columns).min(4));
    let (columns, rows) = if column {
        (value, rows)
    } else {
        (columns, value)
    };
    Some((columns, columns * rows))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn density_preserves_legacy_until_an_explicit_dimension_choice() {
        assert_eq!(rectangular_rows(4, 13), None);
        assert_eq!(rectangular_rows(1, 16), None);
        assert_eq!(density(4, 13, true, 2), Some((2, 8)));
        assert_eq!(density(1, 16, true, 2), Some((2, 8)));
        assert_eq!(density(4, 13, false, 3), Some((4, 12)));
        assert_eq!(density(2, 8, false, 3), Some((2, 6)));
        assert_eq!(density(2, 8, true, 5), None);
    }
    #[test]
    fn number_edit_replaces_selection_and_keeps_a_real_caret() {
        let mut edit = NumberEdit::new("69".into());
        assert!(edit.replace(None, "123"));
        edit.select(1, false);
        edit.select(2, true);
        assert!(edit.replace(None, "8"));
        assert_eq!(edit.text, "183");
        edit.delete(false);
        assert_eq!(edit.text, "18");
        edit.delete(true);
        assert_eq!(edit.text, "1");
        assert!(!edit.replace(None, "private"));
        assert!(edit.replace(None, " 69\n"));
        assert_eq!(edit.text, "169");
    }
}
