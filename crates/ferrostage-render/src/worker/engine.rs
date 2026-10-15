use super::{Message, Response};
use crate::{
    cache::FrameCache,
    scheduler::{Job, Scheduler},
    *,
};
use ferrostage_core::DocumentGeneration;
use ferrostage_pdf::{PdfError, raster::RasterBackend};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        mpsc::{Receiver, Sender},
    },
    time::Instant,
};

struct Batch {
    remaining: usize,
    token: CandidateReady,
    reply: Response<CandidateReady>,
}
struct Engine<'a> {
    backend: &'a mut dyn RasterBackend,
    worker_identity: Arc<()>,
    active: Option<DocumentGeneration>,
    candidate: Option<DocumentGeneration>,
    newest: Option<DocumentGeneration>,
    epochs: HashMap<DocumentGeneration, u64>,
    ready: Option<(DocumentGeneration, u64, u64)>,
    nonce: u64,
    batch: Option<Batch>,
    scheduler: Scheduler,
    l2: FrameCache,
    l3: FrameCache,
    results: Sender<RenderEvent>,
}

pub(super) fn run(
    backend: &mut dyn RasterBackend,
    budgets: CacheBudgets,
    messages: Receiver<Message>,
    results: Sender<RenderEvent>,
) {
    let mut engine = Engine {
        backend,
        worker_identity: Arc::new(()),
        active: None,
        candidate: None,
        newest: None,
        epochs: HashMap::new(),
        ready: None,
        nonce: 0,
        batch: None,
        scheduler: Scheduler::default(),
        l2: FrameCache::new(budgets.l2_bytes),
        l3: FrameCache::new(budgets.l3_bytes),
        results,
    };
    let mut completed = None;
    loop {
        // Process all control changes received during a synchronous call BEFORE
        // publishing its result. Generation/epoch changes suppress stale frames.
        while let Ok(message) = messages.try_recv() {
            if !engine.message(message) {
                return;
            }
        }
        if let Some((job, result)) = completed.take() {
            engine.complete(job, result);
        }
        if let Some(job) = engine.scheduler.pop() {
            let result = engine.render(&job);
            completed = Some((job, result));
        } else {
            match messages.recv() {
                Ok(message) => {
                    if !engine.message(message) {
                        return;
                    }
                }
                Err(_) => return,
            }
        }
    }
}

impl Engine<'_> {
    fn ack(generation: DocumentGeneration) -> GenerationAck {
        GenerationAck { generation }
    }
    fn valid(&self, job: &Job) -> bool {
        (if job.candidate {
            self.candidate
        } else {
            self.active
        }) == Some(job.request.generation)
            && self.epochs.get(&job.request.generation) == Some(&job.request.epoch)
            && (!job.candidate || job.candidate_nonce == self.nonce)
    }
    fn invalidate(&mut self, generation: DocumentGeneration) {
        for job in self
            .scheduler
            .cancel(|job| job.request.generation == generation)
        {
            self.complete(job, Err(RenderError::Cancelled));
        }
        self.l2.retire(generation);
        self.l3.retire(generation);
    }
    fn abort_batch(&mut self) {
        self.ready = None;
        if let Some(batch) = self.batch.take() {
            batch.reply.send(Err(RenderError::Cancelled)).ok();
        }
        self.scheduler.cancel(|job| job.candidate);
    }
    fn discard_candidate(&mut self) {
        self.abort_batch();
        if let Some(generation) = self.candidate.take() {
            self.invalidate(generation);
            self.epochs.remove(&generation);
            self.backend.remove(generation);
        }
    }
    fn message(&mut self, message: Message) -> bool {
        match message {
            Message::Shutdown => return false,
            Message::Probe(bytes, reply) => {
                reply
                    .send(self.backend.probe(bytes).map_err(Into::into))
                    .ok();
            }
            Message::Load(generation, bytes, candidate, reply) => {
                let result = if self.newest.is_some_and(|old| generation <= old) {
                    Err(RenderError::StaleGeneration)
                } else {
                    self.newest = Some(generation);
                    // Failed candidate preparation closes only candidate resources.
                    if candidate {
                        self.discard_candidate();
                    }
                    self.backend
                        .load(generation, bytes)
                        .inspect(|_| {
                            self.epochs.insert(generation, 0);
                            if candidate {
                                self.candidate = Some(generation);
                            } else {
                                self.discard_candidate();
                                if let Some(old) = self.active.replace(generation) {
                                    self.invalidate(old);
                                }
                            }
                        })
                        .map_err(Into::into)
                };
                reply.send(result).ok();
            }
            Message::Render(job) => {
                if self.valid(&job) {
                    self.scheduler.push(job);
                } else {
                    self.complete(job, Err(RenderError::NotActive));
                }
            }
            Message::Epoch(generation, epoch, reply) => {
                let result = match self.epochs.get_mut(&generation) {
                    Some(current) if epoch == *current => Ok(Self::ack(generation)),
                    Some(current) if epoch > *current => {
                        *current = epoch;
                        if self.candidate == Some(generation) {
                            self.abort_batch();
                        }
                        let cancelled = self.scheduler.cancel(|job| {
                            job.request.generation == generation && job.request.epoch != epoch
                        });
                        for job in cancelled {
                            self.complete(job, Err(RenderError::Cancelled));
                        }
                        Ok(Self::ack(generation))
                    }
                    _ => Err(RenderError::Cancelled),
                };
                reply.send(result).ok();
            }
            Message::PreRender(generation, jobs, reply) => {
                if self.candidate != Some(generation) {
                    reply.send(Err(RenderError::NotCandidate)).ok();
                } else if jobs.is_empty()
                    || jobs.iter().any(|job| {
                        job.request.generation != generation
                            || self.epochs.get(&generation) != Some(&job.request.epoch)
                    })
                {
                    reply.send(Err(RenderError::InvalidCandidateRequests)).ok();
                } else {
                    self.abort_batch();
                    self.nonce += 1;
                    let epoch = jobs[0].request.epoch;
                    self.batch = Some(Batch {
                        remaining: jobs.len(),
                        token: CandidateReady {
                            worker_identity: self.worker_identity.clone(),
                            generation,
                            epoch,
                            nonce: self.nonce,
                            frames: Vec::new(),
                        },
                        reply,
                    });
                    for mut job in jobs {
                        job.candidate_nonce = self.nonce;
                        self.scheduler.push(job);
                    }
                }
            }
            Message::Commit(token, reply) => {
                let result = if Arc::ptr_eq(&self.worker_identity, &token.worker_identity)
                    && self.ready == Some((token.generation, token.epoch, token.nonce))
                    && self.candidate == Some(token.generation)
                {
                    if let Some(old) = self.active.replace(token.generation) {
                        self.invalidate(old);
                    }
                    self.candidate = None;
                    self.ready = None;
                    Ok(Self::ack(token.generation))
                } else {
                    Err(RenderError::NotReady)
                };
                reply.send(result).ok();
            }
            Message::Abort(generation, reply) => {
                let result = if self.candidate == Some(generation) {
                    self.discard_candidate();
                    Ok(Self::ack(generation))
                } else {
                    Err(RenderError::NotCandidate)
                };
                reply.send(result).ok();
            }
            Message::Retire(generation, reply) => {
                let result =
                    if self.active == Some(generation) || self.candidate == Some(generation) {
                        Err(RenderError::GenerationInUse)
                    } else {
                        self.invalidate(generation);
                        self.epochs.remove(&generation);
                        self.backend.remove(generation);
                        Ok(Self::ack(generation))
                    };
                reply.send(result).ok();
            }
        }
        true
    }
    fn cache(&mut self, request: RenderRequest) -> &mut FrameCache {
        if request.priority == RenderPriority::Thumbnail {
            &mut self.l3
        } else {
            &mut self.l2
        }
    }
    fn render(&mut self, job: &Job) -> Result<RenderResult, RenderError> {
        if !self.valid(job) {
            return Err(RenderError::Cancelled);
        }
        let request = job.request;
        if !request.crop.is_inside_unit_square() {
            return Err(PdfError::InvalidCrop.into());
        }
        if request.priority == RenderPriority::Thumbnail
            && request.target_px.height != THUMBNAIL_HEIGHT
        {
            return Err(RenderError::InvalidThumbnail);
        }
        if !job.bypass_cache
            && let Some(frame) = self.cache(request).get(request.into())
        {
            return Ok(RenderResult {
                id: job.id,
                request,
                frame,
                render_duration: None,
                cache_hit: true,
            });
        }
        let start = Instant::now();
        let frame = self.backend.render(
            request.generation,
            request.page,
            request.crop,
            request.target_px,
        )?;
        if request.priority == RenderPriority::Thumbnail && frame.height() != THUMBNAIL_HEIGHT {
            return Err(RenderError::InvalidThumbnail);
        }
        Ok(RenderResult {
            id: job.id,
            request,
            frame,
            render_duration: Some(start.elapsed()),
            cache_hit: false,
        })
    }
    fn complete(&mut self, job: Job, result: Result<RenderResult, RenderError>) {
        if job.candidate
            && (job.candidate_nonce != self.nonce || self.candidate != Some(job.request.generation))
        {
            return;
        }
        let result = if self.valid(&job) {
            result
        } else {
            Err(RenderError::Cancelled)
        };
        if let Ok(result) = &result
            && !job.bypass_cache
            && !result.cache_hit
        {
            self.cache(job.request)
                .insert(job.request.into(), result.frame.clone());
        }
        if job.candidate {
            if let Some(mut batch) = self.batch.take() {
                match result {
                    Ok(result) => {
                        batch.token.frames.push(result);
                        batch.remaining -= 1;
                        if batch.remaining == 0 {
                            self.ready = Some((
                                batch.token.generation,
                                batch.token.epoch,
                                batch.token.nonce,
                            ));
                            batch.reply.send(Ok(batch.token)).ok();
                        } else {
                            self.batch = Some(batch);
                        }
                    }
                    Err(error) => {
                        batch.reply.send(Err(error)).ok();
                        self.discard_candidate();
                    }
                }
            }
        } else {
            self.results
                .send(RenderEvent {
                    id: job.id,
                    request: job.request,
                    outcome: result,
                })
                .ok();
        }
    }
}
