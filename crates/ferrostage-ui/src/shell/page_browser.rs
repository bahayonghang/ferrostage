use super::*;
use crate::page_browser::{ContextualSurface, NumberEdit, PageBrowser};

impl Shell {
    pub(crate) fn browser(&self) -> Option<&PageBrowser> {
        match &self.contextual {
            Some(ContextualSurface::PageBrowser(browser)) => Some(browser),
            _ => None,
        }
    }
    pub(crate) fn close_contextual(&mut self) {
        self.contextual = None;
        self.workbench.grid = false;
        self.workbench
            .frames
            .retain(|(surface, _)| !matches!(surface, crate::workbench::Surface::Thumbnail(_)));
        self.workbench
            .surface_targets
            .retain(|surface, _| !matches!(surface, crate::workbench::Surface::Thumbnail(_)));
    }
    pub(crate) fn open_browser(&mut self) {
        if self.busy() || self.state.page_count() == 0 {
            return;
        }
        self.abandon_modal();
        self.contextual_serial += 1;
        let current = self.state.current_page();
        let entries = self.browser_entries();
        let capacity = self.settings.settings.presenter.grid_visible_pages as usize;
        let position = entries
            .iter()
            .position(|entry| entry.page == current)
            .unwrap_or(0);
        self.workbench.grid_start = position / capacity * capacity;
        self.workbench.grid = true;
        self.contextual = Some(ContextualSurface::PageBrowser(PageBrowser {
            id: self.contextual_serial,
            generation: self.state.generation(),
            edit: NumberEdit::new((position + 1).to_string()),
            target: self
                .state
                .slide_map()
                .and_then(|map| map.logical_for(current))
                .map(|_| current),
            error: None,
        }));
    }
    fn valid_browser(&self, id: u64) -> bool {
        !self.busy()
            && self.browser().is_some_and(|browser| {
                browser.id == id && browser.generation == self.state.generation()
            })
    }
    pub(crate) fn browser_entries(&self) -> Vec<crate::workbench::BrowserEntry> {
        self.state
            .slide_map()
            .map(|map| {
                self.workbench
                    .page_number_mode
                    .entries(map, self.state.page_count())
            })
            .unwrap_or_default()
    }
    pub(super) fn browser_mode(&mut self, id: u64, mode: crate::workbench::PageNumberMode) {
        if !self.valid_browser(id) || mode == self.workbench.page_number_mode {
            return;
        }
        let target = self
            .browser()
            .and_then(|browser| browser.target)
            .unwrap_or(self.state.current_page());
        self.workbench.page_number_mode = mode;
        let entries = self.browser_entries();
        let position = entries
            .iter()
            .position(|entry| entry.page == target)
            .unwrap_or(0);
        let capacity = self.settings.settings.presenter.grid_visible_pages as usize;
        self.workbench.grid_start = position / capacity * capacity;
        // Renew interaction identity: queued old-mode edits and tile activations are inert.
        self.contextual_serial += 1;
        if let Some(ContextualSurface::PageBrowser(browser)) = &mut self.contextual {
            browser.id = self.contextual_serial;
            browser.edit = NumberEdit::new((position + 1).to_string());
            browser.target = Some(target);
            browser.error = None;
        }
        self.workbench
            .frames
            .retain(|(surface, _)| !matches!(surface, crate::workbench::Surface::Thumbnail(_)));
        self.workbench
            .surface_targets
            .retain(|surface, _| !matches!(surface, crate::workbench::Surface::Thumbnail(_)));
    }
    pub(super) fn edit_browser(&mut self, id: u64, edit: NumberEdit) {
        if !self.valid_browser(id) {
            return;
        }
        let entries = self.browser_entries();
        let entry =
            transactions::jump_page(&edit.text, entries.len()).and_then(|index| entries.get(index));
        let target = entry
            .filter(|entry| entry.content_number.is_some())
            .map(|entry| entry.page);
        let error = if entry.is_none() {
            Some("pb-invalid-range")
        } else if target.is_none() {
            Some("pb-notes-page")
        } else {
            None
        };
        let changed = self
            .browser()
            .is_some_and(|browser| browser.edit.text != edit.text);
        if changed
            && let Some(target) = target
            && let Some(position) = entries.iter().position(|entry| entry.page == target)
        {
            let capacity = self.settings.settings.presenter.grid_visible_pages as usize;
            if position < self.workbench.grid_start
                || position >= self.workbench.grid_start + capacity
            {
                self.workbench.grid_start = position / capacity * capacity;
            }
        }
        if let Some(ContextualSurface::PageBrowser(browser)) = &mut self.contextual {
            browser.edit = edit;
            browser.target = target;
            browser.error = error;
        }
    }
    pub(super) fn submit_browser(
        &mut self,
        id: u64,
        page: Option<PhysicalPage>,
        cx: &mut Context<Self>,
    ) {
        if !self.valid_browser(id) {
            return;
        }
        let page = page.or_else(|| self.browser().and_then(|browser| browser.target));
        if let Some(page) = page.filter(|p| {
            self.state
                .slide_map()
                .and_then(|map| map.logical_for(*p))
                .is_some()
        }) {
            self.apply(Command::JumpTo(page), cx);
            self.close_contextual();
        }
    }
    pub(super) fn grid_density(&mut self, column: bool, value: u32, cx: &mut Context<Self>) {
        if self.browser().is_none() || self.busy() || !self.settings.write_policy.is_writable() {
            return;
        }
        let settings = &self.settings.settings.presenter;
        let Some((columns, visible)) = crate::page_browser::density(
            settings.grid_columns,
            settings.grid_visible_pages,
            column,
            value,
        ) else {
            return;
        };
        let mut next = self.settings.settings.clone();
        next.presenter.grid_columns = columns;
        next.presenter.grid_visible_pages = visible;
        self.apply_preferences(next, cx);
    }
}
