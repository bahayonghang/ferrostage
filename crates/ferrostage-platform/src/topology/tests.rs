use super::*;
use crate::{IdentityConfidence, IdentitySource, ScaleSource};
use proptest::prelude::*;

pub(crate) fn monitor(id: &str, primary: bool, size: (u32, u32)) -> MonitorInfo {
    MonitorInfo {
        id: MonitorId(id.into()),
        name: "same-model".into(),
        connection_name: id.into(),
        native_size: size,
        position: (0, 0),
        scale_factor: 1.0,
        scale_source: ScaleSource::GdiDeviceCapsRatio,
        rotation_degrees: 0,
        is_primary: primary,
        identity_source: IdentitySource::DeviceInterfacePath,
        identity_confidence: IdentityConfidence::ConnectionScoped,
    }
}

fn three() -> Vec<MonitorInfo> {
    vec![
        monitor("a", true, (1920, 1080)),
        monitor("b", false, (3840, 2160)),
        monitor("c", false, (2560, 1440)),
    ]
}

fn binding(id: &str, role: MonitorRole) -> (MonitorId, MonitorRole) {
    (MonitorId(id.into()), role)
}
fn id(value: &str) -> Option<MonitorId> {
    Some(MonitorId(value.into()))
}

#[test]
fn zero_and_single_screen_are_safe() {
    let zero = resolve_roles(&[], &[]);
    assert_eq!((zero.presenter, zero.audience), (None, None));
    assert_eq!(zero.warnings, vec![TopologyWarning::NoMonitors]);
    for saved in [
        vec![],
        vec![binding("a", MonitorRole::Audience)],
        vec![binding("gone", MonitorRole::Audience)],
    ] {
        let one = resolve_roles(&three()[..1], &saved);
        assert_eq!((one.presenter, one.audience), (id("a"), None));
        assert_eq!(one.source, ResolutionSource::SingleMonitor);
    }
}

#[test]
fn default_two_and_three_choose_largest_non_presenter() {
    for monitors in [&three()[..2], &three()[..]] {
        let roles = resolve_roles(monitors, &[]);
        assert_eq!((roles.presenter, roles.audience), (id("a"), id("b")));
        assert_eq!(roles.source, ResolutionSource::Heuristic);
    }
}

#[test]
fn missing_primary_and_area_ties_are_independent_of_enumeration_order() {
    let mut monitors = vec![
        monitor("c", false, (10, 10)),
        monitor("a", false, (1, 1)),
        monitor("b", false, (10, 10)),
    ];
    let first = resolve_roles(&monitors, &[]);
    assert_eq!(
        (first.presenter.clone(), first.audience.clone()),
        (id("a"), id("b"))
    );
    monitors.reverse();
    assert_eq!(first, resolve_roles(&monitors, &[]));
}

#[test]
fn saved_roles_are_authoritative_in_configuration_order() {
    let saved = [
        binding("c", MonitorRole::Presenter),
        binding("a", MonitorRole::Audience),
        binding("b", MonitorRole::None),
    ];
    let roles = resolve_roles(&three(), &saved);
    assert_eq!((roles.presenter, roles.audience), (id("c"), id("a")));
    assert_eq!(roles.ignored, vec![MonitorId("b".into())]);
    assert!(roles.warnings.is_empty());
}

#[test]
fn saved_absent_or_explicitly_unused_audience_never_falls_back() {
    for saved in [
        vec![binding("gone", MonitorRole::Audience)],
        vec![binding("b", MonitorRole::None)],
        vec![binding("a", MonitorRole::Presenter)],
    ] {
        let roles = resolve_roles(&three(), &saved);
        assert_eq!(roles.audience, None);
        assert_eq!(roles.presenter, id("a"));
    }
    assert!(
        resolve_roles(&three(), &[binding("gone", MonitorRole::Audience)])
            .warnings
            .contains(&TopologyWarning::SavedAudienceMissing(MonitorId(
                "gone".into()
            )))
    );
}

#[test]
fn saved_audience_resolves_before_missing_presenter_fallback() {
    let roles = resolve_roles(
        &three(),
        &[
            binding("gone", MonitorRole::Presenter),
            binding("a", MonitorRole::Audience),
        ],
    );
    assert_eq!((roles.presenter, roles.audience), (id("b"), id("a")));
    assert!(
        roles
            .warnings
            .contains(&TopologyWarning::PresenterMissingFallbackPrimary)
    );
}

#[test]
fn duplicate_bindings_first_none_wins_before_role_resolution() {
    let roles = resolve_roles(
        &three(),
        &[
            binding("b", MonitorRole::None),
            binding("b", MonitorRole::Audience),
            binding("b", MonitorRole::Presenter),
        ],
    );
    assert_eq!((roles.presenter, roles.audience), (id("a"), None));
    assert_eq!(
        roles
            .warnings
            .iter()
            .filter(|w| matches!(w, TopologyWarning::DuplicateBinding(_)))
            .count(),
        2
    );
}

#[test]
fn duplicate_roles_keep_first_valid_and_report_remaining_ids() {
    let roles = resolve_roles(
        &three(),
        &[
            binding("gone", MonitorRole::Audience),
            binding("c", MonitorRole::Audience),
            binding("b", MonitorRole::Audience),
            binding("a", MonitorRole::Presenter),
        ],
    );
    assert_eq!((roles.presenter, roles.audience), (id("a"), id("c")));
    assert!(
        roles
            .warnings
            .contains(&TopologyWarning::DuplicateAudience(MonitorId("b".into())))
    );
    let roles = resolve_roles(
        &three(),
        &[
            binding("c", MonitorRole::Presenter),
            binding("a", MonitorRole::Presenter),
        ],
    );
    assert_eq!(roles.presenter, id("c"));
    assert!(
        roles
            .warnings
            .contains(&TopologyWarning::DuplicatePresenter(MonitorId("a".into())))
    );
}

#[test]
fn any_identity_collision_disables_saved_and_heuristic_audience() {
    let mut monitors = three();
    monitors.push(monitor("b", false, (3840, 2160)));
    for saved in [
        vec![],
        vec![binding("c", MonitorRole::Audience)],
        vec![binding("b", MonitorRole::Audience)],
    ] {
        let roles = resolve_roles(&monitors, &saved);
        assert_eq!((roles.presenter, roles.audience), (id("a"), None));
        assert!(
            roles
                .warnings
                .contains(&TopologyWarning::AmbiguousMonitorId(MonitorId("b".into())))
        );
    }
    let only_collision = vec![monitor("a", true, (1, 1)), monitor("a", false, (1, 1))];
    assert_eq!(resolve_roles(&only_collision, &[]).presenter, None);
}

#[test]
fn diff_reports_set_changes_and_new_ambiguity_without_indexes() {
    let before = three();
    let next = vec![
        before[2].clone(),
        before[2].clone(),
        monitor("new", false, (1, 1)),
    ];
    assert_eq!(
        diff(&before, &next),
        TopologyDiff {
            added: vec![MonitorId("new".into())],
            removed: vec![MonitorId("a".into()), MonitorId("b".into())],
            ambiguous: vec![MonitorId("c".into())],
        }
    );
}

proptest! {
    #[test]
    fn resolved_ids_are_present_unique_disjoint_and_never_unsaved_audience(
        ids in proptest::collection::vec(0u8..8, 0..12),
        saved in proptest::collection::vec((0u8..10, 0u8..3), 0..12)
    ) {
        let monitors: Vec<_> = ids.iter().map(|id| monitor(&id.to_string(), *id == 0, (u32::from(*id) + 1, 2))).collect();
        let saved: Vec<_> = saved.into_iter().map(|(id, role)| binding(&id.to_string(), match role { 0 => MonitorRole::Presenter, 1 => MonitorRole::Audience, _ => MonitorRole::None })).collect();
        let roles = resolve_roles(&monitors, &saved);
        for resolved in [&roles.presenter, &roles.audience].into_iter().flatten() {
            prop_assert_eq!(monitors.iter().filter(|m| &m.id == resolved).count(), 1);
        }
        if let Some(audience) = &roles.audience {
            prop_assert_ne!(roles.presenter.as_ref(), Some(audience));
            if !saved.is_empty() {
                prop_assert_eq!(saved.iter().find(|(id, _)| id == audience).map(|(_, role)| *role), Some(MonitorRole::Audience));
            }
        }
    }
}
