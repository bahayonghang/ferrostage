//! Projection input only. No controller, path, settings, worker or document.
use ferrostage_core::{AudienceProjectionSnapshot, AudienceStage, BlankMode};
use gpui::{Context, Entity, Render, Window, div, prelude::*, rgb};

pub(crate) struct Audience {
    pub snapshot: Entity<AudienceProjectionSnapshot>,
}

impl Render for Audience {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let snapshot = self.snapshot.read(cx);
        let background = if snapshot.stage == AudienceStage::Curtain {
            0x000000
        } else {
            match snapshot.blank_mode {
                Some(BlankMode::Black) => 0x000000,
                Some(BlankMode::White) => 0xFFFFFF,
                None => 0x282828,
            }
        };
        div()
            .size_full()
            .bg(rgb(background))
            .flex()
            .items_center()
            .justify_center()
            .when(
                snapshot.stage == AudienceStage::Live && snapshot.blank_mode.is_none(),
                |view| {
                    view.child(
                        div()
                            .text_color(rgb(0xFFFFFF))
                            .text_size(gpui::px(48.))
                            .child(format!("{}", snapshot.current_page.0 + 1)),
                    )
                },
            )
    }
}
