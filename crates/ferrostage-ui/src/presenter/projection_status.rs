use super::*;

impl Presenter {
    pub(super) fn projection_status(&self, p: Palette) -> Option<gpui::AnyElement> {
        let status = self.data.projection.as_ref()?;
        let mut row = div()
            .id("projection-status")
            .debug_selector(|| "projection-status".into())
            .role(Role::Status)
            .accessibility_id("projection-status")
            .aria_label(status.summary.clone())
            .px(px(16.))
            .py(px(4.))
            .flex()
            .items_center()
            .flex_wrap()
            .gap(px(8.))
            .text_size(px(self.data.typography.size(12.)))
            .child(
                div()
                    .id("projection-summary")
                    .debug_selector(|| "projection-summary".into())
                    .flex_1()
                    .min_w_0()
                    .text_color(rgb(if status.failed { p.error } else { p.body }))
                    .child(status.summary.clone()),
            )
            .child(
                self.button(
                    "projection-location".into(),
                    status.location.clone(),
                    !self.data.busy,
                    false,
                    Intent::Displays,
                    p,
                )
                .text_size(px(self.data.typography.size(12.))),
            );
        if status.restore {
            row = row.child(self.button(
                "projection-restore".into(),
                self.locale.text("projection-restore").into(),
                !self.data.busy,
                false,
                Intent::RestoreProjection,
                p,
            ));
        }
        Some(row.into_any_element())
    }
}
