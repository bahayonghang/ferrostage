use super::*;
use crate::page_browser::ContextualSurface;

impl Shell {
    pub(crate) fn workbench_layout(&self) -> crate::workbench::WorkbenchLayout {
        if self.workbench.document.as_ref().is_some_and(|session| {
            ferrostage_document::notes_crop(session.document.layout).is_none()
                && session
                    .document
                    .notes_mapping
                    .iter()
                    .all(|mapping| mapping.notes_pages.is_empty())
        }) && !self.crop_needs_confirmation()
            && !self.crop_reload_pending_confirmation()
        {
            crate::workbench::WorkbenchLayout::Previews
        } else {
            crate::workbench::WorkbenchLayout::for_mode(self.settings.settings.display.start_mode)
        }
    }

    pub(super) fn reset_workbench(
        &mut self,
        id: u64,
        layout: crate::workbench::WorkbenchLayout,
        cx: &mut Context<Self>,
    ) {
        if self.busy()
            || !matches!(self.contextual, Some(ContextualSurface::LayoutNavigation { id: current, generation, .. }) if current == id && generation == self.state.generation())
        {
            return;
        }
        let (primary, secondary) =
            layout.ratios(&ferrostage_settings::PresenterSettings::default());
        self.dispatch(
            Intent::Split {
                layout,
                primary,
                secondary,
                save: true,
            },
            cx,
        );
        self.close_contextual();
    }

    pub(crate) fn layout_menu_open(&self) -> bool {
        matches!(
            self.contextual,
            Some(ContextualSurface::LayoutNavigation { .. })
        )
    }

    pub(super) fn reconfigure_layout(
        &mut self,
        id: u64,
        options: ferrostage_document::DocumentOverrides,
        cx: &mut Context<Self>,
    ) {
        if self.busy()
            || self.state.stage() != SessionStage::Ready
            || !matches!(self.contextual, Some(ContextualSurface::LayoutNavigation { id: current, generation, .. }) if current == id && generation == self.state.generation())
        {
            return;
        }
        let Some(session) = &self.workbench.document else {
            return;
        };
        if session.document.overrides == options {
            self.close_contextual();
            return;
        }
        match self.apply_layout_options(options) {
            Ok(()) => {
                self.close_contextual();
                self.crop_confirmed = false;
                self.error = None;
                self.publish(false, cx);
            }
            Err(error) => {
                tracing::warn!(%error, "layout reconfiguration rejected");
                if let Some(ContextualSurface::LayoutNavigation { error: local, .. }) =
                    &mut self.contextual
                {
                    *local = Some(self.locale.text("ln-apply-error").into());
                }
            }
        }
    }

    pub(super) fn apply_layout_options(
        &mut self,
        options: ferrostage_document::DocumentOverrides,
    ) -> Result<(), String> {
        let session = self.workbench.document.as_mut().ok_or("Missing document")?;
        let document = session.reconfigure(options)?;
        let descriptor = document
            .descriptor(self.state.generation())
            .map_err(|e| e.to_string())?;
        self.state
            .apply_at(Command::ReconfigureDocument(descriptor), Instant::now())
            .map_err(|e| e.to_string())?;
        session.document = document;
        self.workbench.clear_frames();
        self.error = None;
        Ok(())
    }
}
