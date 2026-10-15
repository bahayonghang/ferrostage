use crate::{RenderRequest, RequestId};
pub(crate) struct Job {
    pub id: RequestId,
    pub request: RenderRequest,
    pub bypass_cache: bool,
    pub candidate: bool,
    pub candidate_nonce: u64,
}
#[derive(Default)]
pub(crate) struct Scheduler {
    queue: Vec<Job>,
}
impl Scheduler {
    pub fn push(&mut self, job: Job) {
        self.queue.push(job);
    }
    pub fn pop(&mut self) -> Option<Job> {
        let index = self
            .queue
            .iter()
            .enumerate()
            .min_by_key(|(_, job)| (job.request.priority, job.id.0))
            .map(|(index, _)| index)?;
        Some(self.queue.remove(index))
    }
    pub fn cancel(&mut self, mut predicate: impl FnMut(&Job) -> bool) -> Vec<Job> {
        let mut removed = Vec::new();
        let mut index = 0;
        while index < self.queue.len() {
            if predicate(&self.queue[index]) {
                removed.push(self.queue.remove(index));
            } else {
                index += 1;
            }
        }
        removed
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::RenderPriority;
    use ferrostage_core::{DocumentGeneration, NormalizedRect, PhysicalPage};
    use ferrostage_pdf::PixelSize;
    #[test]
    fn next_dispatch_obeys_priority_and_epoch_cancels_queued_work() {
        let mut scheduler = Scheduler::default();
        let request = RenderRequest {
            generation: DocumentGeneration(1),
            page: PhysicalPage(0),
            crop: NormalizedRect::FULL,
            target_px: PixelSize {
                width: 1,
                height: 180,
            },
            priority: RenderPriority::Thumbnail,
            epoch: 0,
        };
        for id in 0..10 {
            scheduler.push(Job {
                id: RequestId(id),
                request,
                bypass_cache: false,
                candidate: false,
                candidate_nonce: 0,
            });
        }
        scheduler.push(Job {
            id: RequestId(10),
            request: RenderRequest {
                priority: RenderPriority::Audience,
                epoch: 1,
                ..request
            },
            bypass_cache: false,
            candidate: false,
            candidate_nonce: 0,
        });
        assert_eq!(scheduler.pop().unwrap().id, RequestId(10));
        assert_eq!(scheduler.cancel(|job| job.request.epoch == 0).len(), 10);
        assert!(scheduler.pop().is_none());
    }
}
