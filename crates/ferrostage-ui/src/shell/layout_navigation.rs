use super::*;
use crate::page_browser::ContextualSurface;

impl Shell {
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
