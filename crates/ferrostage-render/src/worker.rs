use crate::{
    CacheBudgets, CandidateReady, GenerationAck, RenderError, RenderEvent, RenderRequest,
    RequestId, scheduler::Job,
};
use ferrostage_core::DocumentGeneration;
use ferrostage_pdf::{DocumentMetadata, RuntimeConfig, raster::RasterBackend};
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

mod engine;
#[cfg(test)]
mod tests;

pub type Reply<T> = Receiver<Result<T, RenderError>>;
type Response<T> = Sender<Result<T, RenderError>>;

pub(super) enum Message {
    Probe(Vec<u8>, Response<DocumentMetadata>),
    Load(
        DocumentGeneration,
        Vec<u8>,
        bool,
        Response<DocumentMetadata>,
    ),
    PreRender(DocumentGeneration, Vec<Job>, Response<CandidateReady>),
    Render(Job),
    Epoch(DocumentGeneration, u64, Response<GenerationAck>),
    Commit(CandidateReady, Response<GenerationAck>),
    CommitReversible(CandidateReady, Response<crate::RollbackToken>),
    Rollback(crate::RollbackToken, Response<GenerationAck>),
    Abort(DocumentGeneration, Response<GenerationAck>),
    Retire(DocumentGeneration, Response<GenerationAck>),
    Budgets(CacheBudgets, Response<crate::CacheBudgetAck>),
    Shutdown,
}

/// The only production constructor creates PDFium on its named worker thread.
/// Dropping the handle joins that thread after its current synchronous call.
pub struct RenderHandle {
    commands: Sender<Message>,
    results: Receiver<RenderEvent>,
    next_id: AtomicU64,
    thread: Option<JoinHandle<()>>,
}

impl RenderHandle {
    pub fn spawn(config: RuntimeConfig, budgets: CacheBudgets) -> Result<Self, RenderError> {
        #[cfg(feature = "pdfium")]
        {
            Self::start(move |rx, tx, ready| {
                let report = ready.clone();
                if let Err(error) = ferrostage_pdf::with_native_session(config, |backend| {
                    if ready.send(Ok(())).is_ok() {
                        engine::run(backend, budgets, rx, tx);
                    }
                }) {
                    report.send(Err(error.into())).ok();
                }
            })
        }
        #[cfg(not(feature = "pdfium"))]
        {
            drop(config);
            let _budgets = budgets;
            Err(RenderError::FeatureDisabled)
        }
    }

    /// Adapter injection for deterministic worker tests. The factory executes on
    /// the worker; this cannot transport native handles from the caller.
    #[doc(hidden)]
    pub fn with_backend(
        factory: impl FnOnce() -> Box<dyn RasterBackend> + Send + 'static,
        budgets: CacheBudgets,
    ) -> Result<Self, RenderError> {
        Self::start(move |rx, tx, ready| {
            let mut backend = factory();
            if ready.send(Ok(())).is_ok() {
                engine::run(backend.as_mut(), budgets, rx, tx);
            }
        })
    }

    fn start(
        run: impl FnOnce(Receiver<Message>, Sender<RenderEvent>, Response<()>) + Send + 'static,
    ) -> Result<Self, RenderError> {
        let (commands, rx) = mpsc::channel();
        let (tx, results) = mpsc::channel();
        let (ready, status) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("ferrostage-pdf".into())
            .spawn(move || run(rx, tx, ready))
            .map_err(|_| RenderError::WorkerUnavailable)?;
        match status.recv().unwrap_or(Err(RenderError::WorkerUnavailable)) {
            Ok(()) => Ok(Self {
                commands,
                results,
                next_id: AtomicU64::new(1),
                thread: Some(thread),
            }),
            Err(error) => {
                thread.join().map_err(|_| RenderError::WorkerUnavailable)?;
                Err(error)
            }
        }
    }

    fn reply<T>(
        &self,
        message: impl FnOnce(Response<T>) -> Message,
    ) -> Result<Reply<T>, RenderError> {
        let (tx, rx) = mpsc::channel();
        self.commands
            .send(message(tx))
            .map_err(|_| RenderError::WorkerUnavailable)?;
        Ok(rx)
    }
    /// Update existing caches on the single worker without replacing its document.
    pub fn set_cache_budgets(
        &self,
        budgets: CacheBudgets,
    ) -> Result<Reply<crate::CacheBudgetAck>, RenderError> {
        self.reply(|tx| Message::Budgets(budgets, tx))
    }
    pub fn probe(&self, bytes: Vec<u8>) -> Result<Reply<DocumentMetadata>, RenderError> {
        self.reply(|tx| Message::Probe(bytes, tx))
    }
    pub fn load_active(
        &self,
        generation: DocumentGeneration,
        bytes: Vec<u8>,
    ) -> Result<Reply<DocumentMetadata>, RenderError> {
        self.reply(|tx| Message::Load(generation, bytes, false, tx))
    }
    pub fn prepare_candidate(
        &self,
        generation: DocumentGeneration,
        bytes: Vec<u8>,
    ) -> Result<Reply<DocumentMetadata>, RenderError> {
        self.reply(|tx| Message::Load(generation, bytes, true, tx))
    }
    pub fn pre_render_candidate(
        &self,
        generation: DocumentGeneration,
        requests: Vec<RenderRequest>,
    ) -> Result<Reply<CandidateReady>, RenderError> {
        let jobs = requests
            .into_iter()
            .map(|request| self.job(request, false, true))
            .collect();
        self.reply(|tx| Message::PreRender(generation, jobs, tx))
    }
    fn job(&self, request: RenderRequest, bypass_cache: bool, candidate: bool) -> Job {
        Job {
            id: RequestId(self.next_id.fetch_add(1, Ordering::Relaxed)),
            request,
            bypass_cache,
            candidate,
            candidate_nonce: 0,
        }
    }
    pub fn submit(&self, request: RenderRequest) -> Result<RequestId, RenderError> {
        self.submit_inner(request, false)
    }
    /// Used for measured renders: neither reads nor populates caches.
    pub fn submit_uncached(&self, request: RenderRequest) -> Result<RequestId, RenderError> {
        self.submit_inner(request, true)
    }
    fn submit_inner(
        &self,
        request: RenderRequest,
        bypass_cache: bool,
    ) -> Result<RequestId, RenderError> {
        let job = self.job(request, bypass_cache, false);
        let id = job.id;
        self.commands
            .send(Message::Render(job))
            .map_err(|_| RenderError::WorkerUnavailable)?;
        Ok(id)
    }
    pub fn results(&self) -> &Receiver<RenderEvent> {
        &self.results
    }
    pub fn advance_epoch(
        &self,
        generation: DocumentGeneration,
        epoch: u64,
    ) -> Result<Reply<GenerationAck>, RenderError> {
        self.reply(|tx| Message::Epoch(generation, epoch, tx))
    }
    pub fn commit_candidate(
        &self,
        ready: CandidateReady,
    ) -> Result<Reply<GenerationAck>, RenderError> {
        self.reply(|tx| Message::Commit(ready, tx))
    }
    pub fn abort_candidate(
        &self,
        generation: DocumentGeneration,
    ) -> Result<Reply<GenerationAck>, RenderError> {
        self.reply(|tx| Message::Abort(generation, tx))
    }
    pub fn commit_reversible(
        &self,
        ready: CandidateReady,
    ) -> Result<Reply<crate::RollbackToken>, RenderError> {
        self.reply(|tx| Message::CommitReversible(ready, tx))
    }
    pub fn rollback_committed(
        &self,
        token: crate::RollbackToken,
    ) -> Result<Reply<GenerationAck>, RenderError> {
        self.reply(|tx| Message::Rollback(token, tx))
    }
    pub fn retire_generation(
        &self,
        generation: DocumentGeneration,
    ) -> Result<Reply<GenerationAck>, RenderError> {
        self.reply(|tx| Message::Retire(generation, tx))
    }
    pub fn shutdown(mut self) -> Result<(), RenderError> {
        self.stop()
    }
    fn stop(&mut self) -> Result<(), RenderError> {
        if let Some(thread) = self.thread.take() {
            self.commands.send(Message::Shutdown).ok();
            thread.join().map_err(|_| RenderError::WorkerUnavailable)?;
        }
        Ok(())
    }
}
impl Drop for RenderHandle {
    fn drop(&mut self) {
        self.stop().ok();
    }
}
