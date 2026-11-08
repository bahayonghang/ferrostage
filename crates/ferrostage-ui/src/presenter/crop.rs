use super::*;
use crate::{
    shell::crop::{CropPhase, CropPreview},
    workbench::Surface,
};
use ferrostage_core::BgraFrame;
use ferrostage_document::NotesLayout;

const CHOICES: [(NotesLayout, &str, &str); 5] = [
    (NotesLayout::None, "whole", "crop-whole"),
    (NotesLayout::Right, "right", "wb-right"),
    (NotesLayout::Left, "left", "wb-left"),
    (NotesLayout::Bottom, "bottom", "wb-bottom"),
    (NotesLayout::Top, "top", "wb-top"),
];

pub(super) fn frames(crop: &CropPreview) -> Vec<(Surface, BgraFrame)> {
    crop.public_frame
        .iter()
        .cloned()
        .map(|f| (Surface::CropSample(0), f))
        .chain(
            crop.notes_frames
                .iter()
                .cloned()
                .enumerate()
                .map(|(index, f)| (Surface::CropSample(index + 1), f)),
        )
        .collect()
}

impl Presenter {
    pub(super) fn crop_initial_focus(&self, crop: &CropPreview) -> &FocusHandle {
        if crop.read_only || crop.phase != CropPhase::Inspect {
            return &self.crop_cancel_focus;
        }
        &self.crop_layout_focus[CHOICES
            .iter()
            .position(|&(layout, _, _)| layout == crop.layout)
            .unwrap_or(0)]
    }

    pub(super) fn crop_key(
        &self,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(crop) = &self.data.crop else {
            return false;
        };
        if crop.phase != CropPhase::Inspect {
            return false;
        }
        let key = event.keystroke.unparse();
        if matches!(key.as_str(), "pageup" | "pagedown") {
            actions::dispatch(
                &self.shell,
                Intent::CropSample {
                    id: crop.id,
                    delta: if key == "pageup" { -1 } else { 1 },
                },
                cx,
            );
            return true;
        }
        let Some(current) = self
            .crop_layout_focus
            .iter()
            .position(|f| f.is_focused(window))
        else {
            return false;
        };
        let index = match key.as_str() {
            "left" | "up" => (current + CHOICES.len() - 1) % CHOICES.len(),
            "right" | "down" => (current + 1) % CHOICES.len(),
            "home" => 0,
            "end" => CHOICES.len() - 1,
            _ => return false,
        };
        window.focus(&self.crop_layout_focus[index], cx);
        actions::dispatch(
            &self.shell,
            Intent::CropLayout {
                id: crop.id,
                layout: CHOICES[index].0,
            },
            cx,
        );
        true
    }

    pub(super) fn crop_hint(&self, p: Palette) -> Stateful<Div> {
        let key = match (self.data.held_crop_reload, self.data.stage) {
            (true, SessionStage::Presenting) => "crop-update-stop",
            (true, _) => "crop-update-ready",
            _ => "crop-suspected",
        };
        // An in-panel notice: warning mark, status text, then the check action.
        self.material
            .notice(p, p.warning)
            .id("crop-hint")
            .debug_selector(|| "crop-hint".into())
            .flex_wrap()
            .child(
                div()
                    .id("crop-status")
                    .role(Role::Status)
                    .flex_1()
                    .min_w_0()
                    .text_color(rgb(p.ink))
                    .child(self.locale.text(key)),
            )
            .when(self.data.stage == SessionStage::Ready, |view| {
                view.child(self.button(
                    "crop-check-hint".into(),
                    self.locale.text("crop-check-entry").into(),
                    !self.data.busy,
                    false,
                    Intent::OpenCrop,
                    p,
                ))
            })
    }

    fn crop_image(
        &self,
        slot: usize,
        expected: &BgraFrame,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let cache = self.textures.clone();
        let presenter = cx.weak_entity();
        let expected = expected.clone();
        let key = Surface::CropSample(slot);
        gpui::canvas(
            |_, _, _| (),
            move |bounds, _, window, cx| {
                let mut cache = cache.borrow_mut();
                let was_ready = cache.image(key).is_some_and(|(frame, _)| {
                    frame.pixels_bgra().as_ptr() == expected.pixels_bgra().as_ptr()
                });
                let old_error = cache.error();
                let ready = cache.paint(key, None, window, cx, |frame, image, _, window| {
                    if frame.pixels_bgra().as_ptr() != expected.pixels_bgra().as_ptr() {
                        return Err("Crop preview changed".into());
                    }
                    crate::slide_surface::paint(bounds, frame, image, None, window)
                });
                if was_ready != ready || old_error != cache.error() {
                    let owner = presenter.clone();
                    cx.defer(move |cx| {
                        let _ = owner.update(cx, |_, cx| cx.notify());
                    });
                }
            },
        )
        .size_full()
    }

    pub(super) fn crop_panel(
        &self,
        p: Palette,
        window: &Window,
        cx: &Context<Self>,
    ) -> gpui::AnyElement {
        let Some(crop) = &self.data.crop else {
            return div().into_any_element();
        };
        let l = self.locale;
        let id = crop.id;
        let wide = window.viewport_size().width >= px(680.);
        let small = window.viewport_size().height < px(550.);
        let cache = self.textures.borrow();
        let upload_error = cache.error().map(|e| l.error("wb-upload-error", &e));
        // The public crop and the first available private crop must have
        // reached the atlas before these pixels can authorize a projection.
        let painted = |slot, frame: &BgraFrame| {
            cache
                .image(Surface::CropSample(slot))
                .is_some_and(|(actual, _)| {
                    actual.pixels_bgra().as_ptr() == frame.pixels_bgra().as_ptr()
                })
        };
        let shown = crop
            .public_frame
            .as_ref()
            .is_some_and(|frame| painted(0, frame))
            && crop
                .notes_frames
                .first()
                .is_none_or(|frame| painted(1, frame));
        drop(cache);
        let preview_height = if small || !wide { 152. } else { 260. };
        let prompt = if crop.phase == CropPhase::Applying {
            l.text("crop-applying").to_owned()
        } else if crop.phase == CropPhase::Retrying {
            l.text("crop-rechecking").to_owned()
        } else if crop.phase == CropPhase::Failed {
            l.text("crop-apply-failed").to_owned()
        } else if crop.read_only {
            l.text("crop-readonly").to_owned()
        } else if let Some(layout) = crop.suggestion {
            let key = CHOICES
                .iter()
                .find(|(choice, _, _)| *choice == layout)
                .map_or("crop-whole", |(_, _, k)| *k);
            l.format("crop-suggestion", &[("layout", l.text(key).into())])
        } else {
            l.text("crop-check-help").to_owned()
        };
        let choices =
            CHOICES.iter().enumerate().fold(
                self.segmented(p)
                    .id("crop-layouts")
                    .role(Role::RadioGroup)
                    .aria_label(l.text("ln-layout-group")),
                |view, (index, &(layout, name, label))| {
                    let selected = layout == crop.layout;
                    view.child(
                        self.button(
                            format!("crop-layout-{name}"),
                            l.text(label).into(),
                            !crop.read_only && crop.phase == CropPhase::Inspect,
                            selected,
                            Intent::CropLayout { id, layout },
                            p,
                        )
                        .role(Role::RadioButton)
                        .track_focus(&self.crop_layout_focus[index].clone().tab_stop(
                            !crop.read_only && crop.phase == CropPhase::Inspect && selected,
                        ))
                        .aria_toggled(if selected {
                            gpui::Toggled::True
                        } else {
                            gpui::Toggled::False
                        }),
                    )
                },
            );
        let placeholder = |key: &'static str| {
            div()
                .size_full()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(p.muted))
                .child(l.text(key))
        };
        let public = div()
            .id("crop-public-preview")
            .accessibility_id("crop-public-preview")
            .role(Role::Group)
            .aria_label(l.text("crop-public"))
            .debug_selector(|| "crop-public-preview".into())
            .w_full()
            .h(px(preview_height))
            .min_w_0()
            .overflow_hidden()
            .when_some(crop.public_frame.as_ref(), |view, frame| {
                view.child(self.crop_image(0, frame, cx))
            })
            .when(crop.public_frame.is_none(), |view| {
                view.child(placeholder(if crop.error.is_some() {
                    "crop-preview-failed"
                } else {
                    "crop-loading"
                }))
            });
        let mut notes = div()
            .id("crop-notes-preview")
            .accessibility_id("crop-notes-preview")
            .role(Role::Group)
            .aria_label(l.text("crop-private"))
            .debug_selector(|| "crop-notes-preview".into())
            .w_full()
            .h(px(preview_height))
            .overflow_y_scroll();
        if crop.notes_frames.is_empty() {
            notes = notes.child(placeholder(if crop.error.is_some() {
                "crop-preview-failed"
            } else if crop.pending {
                "crop-loading"
            } else {
                "crop-no-notes"
            }));
        } else {
            for (index, frame) in crop.notes_frames.iter().enumerate() {
                notes = notes.child(
                    div()
                        .id(SharedString::from(format!("crop-note-{index}")))
                        .w_full()
                        .h(px(preview_height))
                        .child(self.crop_image(index + 1, frame, cx)),
                );
            }
        }
        let pane = |label, face: gpui::AnyElement| {
            div()
                .flex_1()
                .min_w_0()
                .p(px(8.))
                .rounded(px(theme::WELL_RADIUS))
                .bg(rgb(p.well))
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(
                    div()
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .text_color(rgb(p.ink))
                        .child(l.text(label)),
                )
                .child(face)
        };
        let previews = div()
            .w_full()
            .flex()
            .gap(px(12.))
            .when(!wide, |v| v.flex_col())
            .child(pane("crop-public", public.into_any_element()))
            .child(pane("crop-private", notes.into_any_element()));
        let sample = div()
            .flex()
            .items_center()
            .flex_wrap()
            .gap(px(8.))
            .child(self.button(
                "crop-sample-prev".into(),
                l.text("crop-prev").into(),
                crop.phase == CropPhase::Inspect && crop.sample_index > 0,
                false,
                Intent::CropSample { id, delta: -1 },
                p,
            ))
            .child(div().child(l.format(
                "crop-sample",
                &[
                    ("page", (crop.sample_index + 1).to_string()),
                    ("total", crop.sample_count.to_string()),
                ],
            )))
            .child(self.button(
                "crop-sample-next".into(),
                l.text("crop-next").into(),
                crop.phase == CropPhase::Inspect && crop.sample_index + 1 < crop.sample_count,
                false,
                Intent::CropSample { id, delta: 1 },
                p,
            ));
        let error = crop.error.as_ref().cloned().or(upload_error);
        let contents = div()
            .id("crop-scroll")
            .flex_1()
            .min_h_0()
            .overflow_y_scroll()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .child(div().child(prompt))
                    .child(choices)
                    .child(previews)
                    .child(sample)
                    .when(
                        crop.layout == NotesLayout::None && crop.suggestion.is_some(),
                        |v| {
                            v.child(
                                div()
                                    .text_color(rgb(p.warning))
                                    .child(l.text("crop-whole-warning")),
                            )
                        },
                    )
                    .when(crop.changing_with_annotations, |v| {
                        v.child(
                            div()
                                .text_color(rgb(p.warning))
                                .child(l.text("crop-clear-annotations")),
                        )
                    })
                    .when(crop.remapping_annotations, |v| {
                        v.child(
                            div()
                                .text_color(rgb(p.muted))
                                .child(l.text("crop-remap-annotations")),
                        )
                    })
                    .when_some(error.as_ref(), |v, error| {
                        v.child(
                            div()
                                .id("crop-error")
                                .role(Role::Status)
                                .text_color(rgb(p.error))
                                .child(error.clone()),
                        )
                        .child(self.button(
                            "crop-retry".into(),
                            l.text("crop-retry").into(),
                            true,
                            false,
                            Intent::RetryCrop(id),
                            p,
                        ))
                    }),
            );
        let confirm_key = if !crop.from_start {
            "crop-apply"
        } else if crop.layout == NotesLayout::None {
            "crop-start-whole"
        } else {
            "crop-start"
        };
        let footer = div()
            .flex_shrink_0()
            .flex()
            .flex_wrap()
            .items_center()
            .justify_end()
            .gap(px(8.))
            .child(
                self.button(
                    "crop-cancel".into(),
                    l.text(if crop.read_only { "close" } else { "cancel" })
                        .into(),
                    true,
                    false,
                    Intent::CancelCrop(id),
                    p,
                )
                .track_focus(&self.crop_cancel_focus),
            )
            .when(!crop.read_only, |v| {
                v.child(self.button(
                    "crop-confirm".into(),
                    l.text(confirm_key).into(),
                    crop.can_confirm && shown && error.is_none(),
                    true,
                    Intent::ConfirmCrop(id),
                    p,
                ))
            });
        div()
            .id("crop-panel")
            .accessibility_id("crop-panel")
            .debug_selector(|| "crop-panel".into())
            .role(Role::Dialog)
            .aria_label(l.text("crop-title"))
            .size_full()
            .min_h_0()
            .p(px(if small { 8. } else { 16. }))
            .flex()
            .flex_col()
            .gap(px(12.))
            .child(
                div()
                    .flex_shrink_0()
                    .text_size(px(self.ui_text_size(22.)))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(l.text("crop-title")),
            )
            .child(contents)
            .child(footer)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::CHOICES;
    use crate::locale::Locale;

    #[test]
    fn crop_choices_actions_and_conditional_messages_exist_in_both_languages() {
        for language in ["en-US", "zh-CN"] {
            let l = Locale::new(language);
            for (_, _, key) in CHOICES {
                assert!(!l.text(key).is_empty());
            }
            for key in [
                "crop-title",
                "crop-check-entry",
                "crop-update-stop",
                "crop-update-ready",
                "crop-suspected",
                "crop-readonly",
                "crop-check-help",
                "crop-suggestion",
                "crop-public",
                "crop-private",
                "crop-loading",
                "crop-no-notes",
                "crop-preview-failed",
                "crop-preview-unavailable",
                "crop-apply-failed",
                "crop-retry",
                "crop-sample",
                "crop-prev",
                "crop-next",
                "crop-whole-warning",
                "crop-clear-annotations",
                "crop-remap-annotations",
                "crop-applying",
                "crop-rechecking",
                "crop-retry-unavailable",
                "crop-start",
                "crop-start-whole",
                "crop-apply",
                "cancel",
                "close",
            ] {
                assert!(!l.text(key).is_empty(), "{language} {key}");
            }
        }
    }
}
