//! Presenter-owned document and exact-key render scheduling. No GPUI/native handles.
use ferrostage_core::{
    AudienceProjectionSnapshot, AudienceStage, AudienceVisual, BgraFrame, DocumentGeneration,
    NormalizedRect, PhysicalPage, PresentationState,
};
use ferrostage_document::{DocumentOverrides, LoadedDocument};
use ferrostage_pdf::{DocumentMetadata, DocumentStructure, PixelSize};
use ferrostage_render::{RenderHandle, RenderPriority, RenderRequest, RequestId};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tool {
    #[default]
    Navigate,
    Laser,
    Pen,
    Highlighter,
    Eraser,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Surface {
    Audience,
    Current,
    Next,
    Notes(usize),
    Thumbnail(usize),
    Prefetch(usize),
    Media(u64),
}
#[derive(Clone)]
pub struct ReadyFrame {
    pub request: RenderRequest,
    pub frame: BgraFrame,
}
#[derive(Clone)]
pub struct DocumentSession {
    pub metadata: DocumentMetadata,
    pub structure: DocumentStructure,
    pub document: LoadedDocument,
}
impl DocumentSession {
    pub fn build(metadata: DocumentMetadata, structure: DocumentStructure) -> Result<Self, String> {
        let document = LoadedDocument::load(&metadata, &structure).map_err(|e| e.to_string())?;
        Ok(Self {
            metadata,
            structure,
            document,
        })
    }
    pub fn reconfigure(&mut self, options: DocumentOverrides) -> Result<LoadedDocument, String> {
        LoadedDocument::build(&self.metadata, &self.structure, options).map_err(|e| e.to_string())
    }
}

#[derive(Default)]
pub struct Workbench {
    pub document: Option<DocumentSession>,
    pub frames: Vec<(Surface, ReadyFrame)>,
    pending: HashMap<u64, (Surface, RenderRequest)>,
    failed: Vec<RenderRequest>,
    requested: Vec<(Surface, RenderRequest)>,
    epoch: u64,
    pub tool: Tool,
    pub grid: bool,
    pub grid_start: usize,
    pub audience_target: Option<PixelSize>,
    pub presenter_target: Option<PixelSize>,
    pub surface_targets: HashMap<Surface, PixelSize>,
    pub render_ms: f64,
}
impl Workbench {
    pub fn requested_surfaces(&self) -> Vec<Surface> {
        self.requested.iter().map(|(surface, _)| *surface).collect()
    }

    #[cfg(any(feature = "gpui", test))]
    pub(crate) fn failed_thumbnails(&self) -> Vec<usize> {
        self.requested
            .iter()
            .filter_map(|(surface, request)| match surface {
                Surface::Thumbnail(index)
                    if self
                        .failed
                        .iter()
                        .any(|failed| same_pixels(*request, *failed)) =>
                {
                    Some(*index)
                }
                _ => None,
            })
            .collect()
    }

    pub fn clear_frames(&mut self) {
        self.frames.clear();
        self.pending.clear();
        self.failed.clear();
        self.requested.clear();
    }
    pub fn frame(&self, surface: Surface) -> Option<&ReadyFrame> {
        self.frames
            .iter()
            .find(|(s, _)| *s == surface)
            .map(|(_, f)| f)
    }
    pub fn bytes(&self) -> usize {
        self.frames
            .iter()
            .map(|(_, f)| f.frame.allocation_bytes())
            .sum()
    }
    pub fn request(
        &mut self,
        worker: &RenderHandle,
        surface: Surface,
        request: RenderRequest,
    ) -> Result<(), String> {
        if self.frame(surface).is_some_and(|f| f.request == request)
            || self
                .pending
                .values()
                .any(|(s, r)| *s == surface && *r == request)
            || self.failed.contains(&request)
        {
            return Ok(());
        }
        let RequestId(id) = worker.submit(request).map_err(|e| e.to_string())?;
        self.pending.insert(id, (surface, request));
        Ok(())
    }
    pub fn poll(&mut self, worker: &RenderHandle) -> Vec<String> {
        let mut errors = Vec::new();
        while let Ok(event) = worker.results().try_recv() {
            let Some((surface, expected)) = self.pending.remove(&event.id.0) else {
                continue;
            };
            if event.request != expected {
                continue;
            }
            match event.outcome {
                Ok(result) if result.id == event.id && result.request == expected => {
                    if let Some(duration) = result.render_duration {
                        self.render_ms = duration.as_secs_f64() * 1000.;
                    }
                    self.frames.retain(|(s, _)| *s != surface);
                    self.frames.push((
                        surface,
                        ReadyFrame {
                            request: expected,
                            frame: result.frame,
                        },
                    ));
                }
                Ok(_) => errors.push("Raster result identity mismatch".into()),
                Err(ferrostage_render::RenderError::Cancelled) => {}
                Err(error) => {
                    self.failed.push(expected);
                    errors.push(error.to_string());
                }
            }
        }
        errors
    }
    pub fn requests(
        &mut self,
        state: &PresentationState,
        settings: &ferrostage_settings::PresenterSettings,
    ) -> Result<Vec<(Surface, RenderRequest)>, String> {
        let Some(session) = &self.document else {
            return Ok(Vec::new());
        };
        let document = &session.document;
        let page = state.current_page();
        let generation = state.generation();
        let crop = state.slide_crop();
        let target = self
            .surface_targets
            .get(&Surface::Current)
            .copied()
            .or(self.presenter_target)
            .unwrap_or(PixelSize {
                width: 960,
                height: 540,
            });
        let make = |page, crop, target_px, priority| RenderRequest {
            generation,
            page,
            crop,
            target_px,
            priority,
            epoch: 0,
        };
        let mut wanted = Vec::new();
        if let Some(target) = self.audience_target {
            wanted.push((
                Surface::Audience,
                make(state.current_page(), crop, target, RenderPriority::Audience),
            ));
        }
        wanted.push((
            Surface::Current,
            make(page, crop, target, RenderPriority::Presenter),
        ));
        if let Some(next) = document.slide_map.next_step(page) {
            wanted.push((
                Surface::Next,
                make(
                    next,
                    crop,
                    self.surface_targets
                        .get(&Surface::Next)
                        .copied()
                        .unwrap_or(target),
                    RenderPriority::Presenter,
                ),
            ));
        }
        let notes_target = PixelSize {
            width: ((target.width as f64 * settings.notes_font_scale).round() as u32)
                .clamp(1, 8192),
            height: ((target.height as f64 * settings.notes_font_scale).round() as u32)
                .clamp(1, 8192),
        };
        if let Some(notes_crop) = ferrostage_document::notes_crop(document.layout) {
            wanted.push((
                Surface::Notes(0),
                make(
                    page,
                    notes_crop,
                    self.surface_targets
                        .get(&Surface::Notes(0))
                        .copied()
                        .unwrap_or(notes_target),
                    RenderPriority::Presenter,
                ),
            ));
        }
        for (index, note) in document.notes_pages_for(page).iter().enumerate() {
            wanted.push((
                Surface::Notes(index + 1),
                make(
                    *note,
                    NormalizedRect::FULL,
                    self.surface_targets
                        .get(&Surface::Notes(index + 1))
                        .copied()
                        .unwrap_or(notes_target),
                    RenderPriority::Presenter,
                ),
            ));
        }
        let steps = document.slide_map.steps();
        let position = steps.iter().position(|p| *p == page).unwrap_or(0);
        for neighbour in steps.iter().skip(position.saturating_sub(2)).take(5) {
            wanted.push((
                Surface::Prefetch(neighbour.0),
                make(*neighbour, crop, target, RenderPriority::Prefetch),
            ));
        }
        if self.grid {
            self.grid_start = self.grid_start.min(steps.len().saturating_sub(1));
            for (index, p) in steps
                .iter()
                .enumerate()
                .skip(self.grid_start)
                .take(settings.grid_visible_pages as usize)
            {
                wanted.push((
                    Surface::Thumbnail(index),
                    RenderRequest::thumbnail(generation, &session.metadata.pages[p.0], crop, 0)
                        .map_err(|e| e.to_string())?,
                ));
            }
        }
        Ok(wanted)
    }
    pub fn schedule(
        &mut self,
        worker: &RenderHandle,
        state: &PresentationState,
        settings: &ferrostage_settings::PresenterSettings,
    ) -> Result<(), String> {
        let wanted = self.requests(state, settings)?;
        if self.requested != wanted {
            self.epoch += 1;
            // FIFO worker command ordering cancels stale queued work before
            // replacement jobs. Executing PDFium calls remain non-preemptible.
            let _ack = worker
                .advance_epoch(state.generation(), self.epoch)
                .map_err(|e| e.to_string())?;
            self.pending.clear();
            self.requested = wanted.clone();
        }
        self.frames.retain(|(surface, frame)| {
            wanted
                .iter()
                .any(|(s, r)| s == surface && same_pixels(*r, frame.request))
        });
        self.failed
            .retain(|request| wanted.iter().any(|(_, r)| same_pixels(*r, *request)));
        for (surface, mut request) in wanted {
            request.epoch = self.epoch;
            if !self
                .frame(surface)
                .is_some_and(|frame| same_pixels(frame.request, request))
            {
                self.request(worker, surface, request)?;
            }
        }
        Ok(())
    }
}

fn same_pixels(a: RenderRequest, b: RenderRequest) -> bool {
    a.generation == b.generation
        && a.page == b.page
        && a.crop == b.crop
        && a.target_px == b.target_px
        && a.priority == b.priority
}

/// Only this exact Audience-priority key may contribute pixels. Old metadata
/// travels with old pixels; safety covers take effect immediately.
pub fn assemble(
    mut desired: AudienceProjectionSnapshot,
    previous: &AudienceProjectionSnapshot,
    frame: Option<&ReadyFrame>,
    target: Option<PixelSize>,
    texture_budget_bytes: usize,
) -> AudienceProjectionSnapshot {
    if desired.stage == AudienceStage::Curtain {
        return desired;
    }
    if let Some(frame) = frame.filter(|f| {
        f.request.generation == desired.generation
            && f.request.page == desired.current_page
            && f.request.crop == desired.slide_crop
            && Some(f.request.target_px) == target
            && f.request.priority == RenderPriority::Audience
    }) {
        desired.visual = Some(AudienceVisual {
            texture_budget_bytes,
            page: frame.frame.clone(),
            media: Vec::new(),
        });
        return desired;
    }
    let mut held = previous.clone();
    held.stage = desired.stage;
    held.blank_mode = desired.blank_mode;
    if let Some(visual) = &mut held.visual {
        visual.texture_budget_bytes = texture_budget_bytes;
    }
    held.laser_pos_norm = None;
    held.strokes = Default::default();
    if previous.visual.is_none() {
        desired.laser_pos_norm = None;
        desired.strokes = Default::default();
        desired
    } else {
        held
    }
}

pub fn request_for(
    generation: DocumentGeneration,
    page: PhysicalPage,
    crop: NormalizedRect,
) -> RenderRequest {
    RenderRequest {
        generation,
        page,
        crop,
        target_px: PixelSize {
            width: 960,
            height: 540,
        },
        priority: RenderPriority::Presenter,
        epoch: 0,
    }
}

#[cfg(test)]
mod tests;

/// Bound the active audience allocation to its share of the configured L1
/// budget. The Presenter uses the other half; image aspect ratio stays intact.
pub fn budgeted_target(target: PixelSize, budget: usize) -> PixelSize {
    let bytes = f64::from(target.width) * f64::from(target.height) * 4.;
    let scale = ((budget as f64 / bytes).sqrt()).min(1.);
    PixelSize {
        width: (f64::from(target.width) * scale).floor().max(1.) as u32,
        height: (f64::from(target.height) * scale).floor().max(1.) as u32,
    }
}
