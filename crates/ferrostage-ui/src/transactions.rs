//! Decisions consumed by the native coordinator, testable without claiming OS evidence.
use ferrostage_core::SessionStage;
use ferrostage_core::{CoreError, DocumentGeneration, PresentationCommand, PresentationState};
use std::{path::PathBuf, time::Instant};

/// Commit identity metadata and its presenter-only path together. Failed loads
/// leave both the existing core state and its document label untouched.
pub fn commit_import(
    state: &mut PresentationState,
    path: &mut Option<PathBuf>,
    candidate: PathBuf,
    pages: usize,
    now: Instant,
) -> Result<(), CoreError> {
    state.apply_at(
        PresentationCommand::LoadDocument {
            generation: DocumentGeneration(state.generation().0 + 1),
            page_count: pages,
        },
        now,
    )?;
    *path = Some(candidate);
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Start,
    Swap,
    RecoverPresenter,
}

pub fn import_allowed(stage: SessionStage, operation: Option<Operation>) -> bool {
    stage != SessionStage::Presenting
        && matches!(operation, None | Some(Operation::RecoverPresenter))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Escape {
    CloseModal,
    Stop,
    Nothing,
}
pub fn escape(modal: bool, stop: bool) -> Escape {
    if modal {
        Escape::CloseModal
    } else if stop {
        Escape::Stop
    } else {
        Escape::Nothing
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shutdown {
    Curtain,
    RetirePending,
    StopCore,
    RetireAudience,
}
/// The native coordinator executes this sequence, including every publication.
pub fn shutdown() -> &'static [Shutdown] {
    &[
        Shutdown::Curtain,
        Shutdown::RetirePending,
        Shutdown::StopCore,
        Shutdown::RetireAudience,
    ]
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Closed {
    Audience,
    Presenter,
    Pending,
    Retired,
}
pub fn closed<T: Copy + Eq>(
    id: T,
    presenter: Option<T>,
    audience: Option<T>,
    pending: &[T],
) -> Closed {
    if audience == Some(id) {
        Closed::Audience
    } else if presenter == Some(id) {
        Closed::Presenter
    } else if pending.contains(&id) {
        Closed::Pending
    } else {
        Closed::Retired
    }
}

pub struct Prepared<T> {
    pub presenter: T,
    pub audience: Option<T>,
}
pub struct PreparationFailure<T, E> {
    pub created: Vec<T>,
    pub error: E,
}
/// Prepare all replacements before touching authoritative handles or settings.
pub fn prepare<T, E>(
    audience: bool,
    mut create: impl FnMut(bool) -> Result<T, E>,
) -> Result<Prepared<T>, PreparationFailure<T, E>> {
    let presenter = create(false).map_err(|error| PreparationFailure {
        created: Vec::new(),
        error,
    })?;
    let audience = if audience {
        match create(true) {
            Ok(window) => Some(window),
            Err(error) => {
                return Err(PreparationFailure {
                    created: vec![presenter],
                    error,
                });
            }
        }
    } else {
        None
    };
    Ok(Prepared {
        presenter,
        audience,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Focus {
    Wait,
    Confirmed,
    Cancel,
}
pub fn focus<T: Copy + Eq>(
    owner: T,
    current: Option<T>,
    cached_active: bool,
    active: Option<T>,
    expired: bool,
) -> Focus {
    if current != Some(owner) || expired {
        Focus::Cancel
    } else if cached_active && active == Some(owner) {
        Focus::Confirmed
    } else {
        Focus::Wait
    }
}

pub fn jump_page(text: &str, count: usize) -> Option<usize> {
    text.parse::<usize>()
        .ok()
        .filter(|page| *page > 0 && *page <= count)
        .map(|page| page - 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrostage_core::{
        AudienceStage, DocumentGeneration, PresentationCommand as Command, PresentationState,
    };
    use std::time::{Duration, Instant};

    #[test]
    fn invalid_import_preserves_document_state_and_presenter_path() {
        let now = Instant::now();
        let mut core = PresentationState::new(3).unwrap();
        core.apply_at(Command::NextStep, now).unwrap();
        let mut path = Some(PathBuf::from("original.pdf"));
        let before = core.clone();
        assert!(commit_import(&mut core, &mut path, PathBuf::from("empty.pdf"), 0, now).is_err());
        assert_eq!(core, before);
        assert_eq!(path, Some(PathBuf::from("original.pdf")));
        commit_import(&mut core, &mut path, PathBuf::from("valid.pdf"), 2, now).unwrap();
        assert_eq!(core.stage(), SessionStage::Ready);
        assert_eq!(core.generation(), DocumentGeneration(2));
        assert_eq!(
            core.elapsed_at(now + Duration::from_secs(10)),
            Duration::ZERO
        );
        assert_eq!(path, Some(PathBuf::from("valid.pdf")));
    }

    #[test]
    fn startup_import_survives_presenter_confirmation_only() {
        assert!(import_allowed(
            SessionStage::Idle,
            Some(Operation::RecoverPresenter)
        ));
        assert!(!import_allowed(SessionStage::Ready, Some(Operation::Start)));
        assert!(!import_allowed(SessionStage::Ready, Some(Operation::Swap)));
        assert!(!import_allowed(SessionStage::Presenting, None));
    }
    #[test]
    fn settings_escape_closes_modal_before_stop() {
        assert_eq!(escape(true, true), Escape::CloseModal);
        assert_eq!(escape(false, true), Escape::Stop);
    }
    #[test]
    fn shutdown_never_publishes_live_between_curtain_and_close() {
        let now = Instant::now();
        let mut core = PresentationState::idle();
        core.apply_at(
            Command::LoadDocument {
                generation: DocumentGeneration(1),
                page_count: 3,
            },
            now,
        )
        .unwrap();
        core.apply_at(
            Command::StartPresentation {
                audience_available: true,
            },
            now,
        )
        .unwrap();
        let mut published = Vec::new();
        let mut pending_retired = false;
        for step in shutdown() {
            match step {
                Shutdown::Curtain => published.push(AudienceStage::Curtain),
                Shutdown::RetirePending => pending_retired = true,
                Shutdown::StopCore => {
                    core.apply_at(Command::StopPresentation, now + Duration::from_secs(5))
                        .unwrap();
                    published.push(core.audience_snapshot().stage);
                }
                Shutdown::RetireAudience => assert_eq!(core.stage(), SessionStage::Ready),
            }
        }
        assert!(pending_retired);
        assert!(
            published
                .iter()
                .all(|stage| *stage == AudienceStage::Curtain),
            "{published:?}"
        );
    }
    #[test]
    fn failed_second_replacement_returns_only_new_resources_for_cleanup() {
        let mut old = (11, 12);
        let mut created = Vec::new();
        let result = prepare(true, |audience| {
            if audience {
                Err("creation failure")
            } else {
                created.push(21);
                Ok(21)
            }
        });
        match result {
            Ok(prepared) => old = (prepared.presenter, prepared.audience.unwrap()),
            Err(failure) => {
                assert_eq!(failure.error, "creation failure");
                assert_eq!(failure.created, created);
            }
        }
        assert_eq!(old, (11, 12));
    }
    #[test]
    fn retiring_old_windows_cannot_stop_replacements() {
        assert_eq!(closed(11, Some(21), Some(22), &[]), Closed::Retired);
        assert_eq!(closed(12, Some(21), Some(22), &[]), Closed::Retired);
        assert_eq!(closed(22, Some(21), Some(22), &[]), Closed::Audience);
        assert_eq!(closed(31, Some(21), Some(22), &[31, 32]), Closed::Pending);
    }
    #[test]
    fn focus_waits_for_both_observations_and_rejects_retired_owners() {
        assert_eq!(focus(1, Some(1), true, Some(2), false), Focus::Wait);
        assert_eq!(focus(1, Some(1), false, Some(1), false), Focus::Wait);
        assert_eq!(focus(1, Some(2), true, Some(1), false), Focus::Cancel);
        assert_eq!(focus(1, Some(1), true, Some(1), true), Focus::Cancel);
        assert_eq!(focus(1, Some(1), true, Some(1), false), Focus::Confirmed);
    }
    #[test]
    fn invalid_jump_text_cannot_resolve_to_a_content_page() {
        for text in ["", "0", "4", "999999999999999999999999", "x"] {
            assert_eq!(jump_page(text, 3), None);
        }
        assert_eq!(jump_page("3", 3), Some(2));
    }
}
