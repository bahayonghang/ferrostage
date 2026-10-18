use std::collections::{BTreeMap, BTreeSet};

use ferrostage_core::{MonitorId, MonitorRole};

use crate::MonitorInfo;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResolutionSource {
    NoMonitors,
    SingleMonitor,
    Heuristic,
    Saved,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TopologyWarning {
    NoMonitors,
    SavedAudienceMissing(MonitorId),
    DuplicateAudience(MonitorId),
    DuplicatePresenter(MonitorId),
    PresenterMissingFallbackPrimary,
    DuplicateBinding(MonitorId),
    AmbiguousMonitorId(MonitorId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedTopology {
    pub presenter: Option<MonitorId>,
    pub audience: Option<MonitorId>,
    pub ignored: Vec<MonitorId>,
    pub source: ResolutionSource,
    pub warnings: Vec<TopologyWarning>,
}

/// Resolve the single settings authority. App converts settings string ids to
/// these core types; neither platform nor core depends on the settings crate.
#[must_use]
pub fn resolve_roles(
    monitors: &[MonitorInfo],
    saved: &[(MonitorId, MonitorRole)],
) -> ResolvedTopology {
    let counts = id_counts(monitors);
    let mut warnings: Vec<_> = counts
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|(id, _)| TopologyWarning::AmbiguousMonitorId(id.clone()))
        .collect();
    let ambiguous = !warnings.is_empty();
    let unique: BTreeMap<_, _> = monitors
        .iter()
        .filter(|m| counts[&m.id] == 1)
        .map(|m| (m.id.clone(), m))
        .collect();
    // First binding per identity is authoritative, including explicit None.
    let mut seen = BTreeSet::new();
    let bindings: Vec<_> = saved
        .iter()
        .filter(|(id, _)| {
            if seen.insert(id.clone()) {
                true
            } else {
                warnings.push(TopologyWarning::DuplicateBinding(id.clone()));
                false
            }
        })
        .collect();
    let mut presenter = None;
    let mut audience = None;
    for (id, role) in bindings {
        match role {
            MonitorRole::Audience if !unique.contains_key(id) => {
                warnings.push(TopologyWarning::SavedAudienceMissing(id.clone()))
            }
            MonitorRole::Audience if audience.is_some() => {
                warnings.push(TopologyWarning::DuplicateAudience(id.clone()))
            }
            MonitorRole::Audience => audience = Some(id.clone()),
            MonitorRole::Presenter if !unique.contains_key(id) => {}
            MonitorRole::Presenter if presenter.is_some() => {
                warnings.push(TopologyWarning::DuplicatePresenter(id.clone()))
            }
            MonitorRole::Presenter => presenter = Some(id.clone()),
            MonitorRole::None => {}
        }
    }
    if ambiguous {
        audience = None;
    }
    let source = if monitors.is_empty() {
        warnings.push(TopologyWarning::NoMonitors);
        ResolutionSource::NoMonitors
    } else if monitors.len() == 1 {
        presenter = Some(monitors[0].id.clone());
        audience = None;
        ResolutionSource::SingleMonitor
    } else if saved.is_empty() {
        presenter = fallback_presenter(&unique, None);
        if !ambiguous {
            // BTreeMap order breaks equal-area ties by smallest id.
            audience = unique
                .iter()
                .filter(|(id, _)| Some(*id) != presenter.as_ref())
                .min_by_key(|(id, m)| {
                    (
                        std::cmp::Reverse(u64::from(m.native_size.0) * u64::from(m.native_size.1)),
                        *id,
                    )
                })
                .map(|(id, _)| id.clone());
        }
        ResolutionSource::Heuristic
    } else {
        if presenter.is_none() {
            presenter = fallback_presenter(&unique, audience.as_ref());
            if presenter.is_some() {
                warnings.push(TopologyWarning::PresenterMissingFallbackPrimary);
            }
        }
        ResolutionSource::Saved
    };
    let ignored = counts
        .into_keys()
        .filter(|id| Some(id) != presenter.as_ref() && Some(id) != audience.as_ref())
        .collect();
    ResolvedTopology {
        presenter,
        audience,
        ignored,
        source,
        warnings,
    }
}

fn fallback_presenter(
    monitors: &BTreeMap<MonitorId, &MonitorInfo>,
    excluded: Option<&MonitorId>,
) -> Option<MonitorId> {
    monitors
        .iter()
        .filter(|(id, _)| Some(*id) != excluded)
        .min_by_key(|(id, m)| (!m.is_primary, *id))
        .map(|(id, _)| id.clone())
}

fn id_counts(monitors: &[MonitorInfo]) -> BTreeMap<MonitorId, usize> {
    let mut counts = BTreeMap::new();
    for m in monitors {
        *counts.entry(m.id.clone()).or_default() += 1;
    }
    counts
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TopologyDiff {
    pub added: Vec<MonitorId>,
    pub removed: Vec<MonitorId>,
    /// Identities that cannot identify one monitor in the next topology.
    pub ambiguous: Vec<MonitorId>,
}

#[must_use]
pub fn diff(previous: &[MonitorInfo], next: &[MonitorInfo]) -> TopologyDiff {
    let before: BTreeSet<_> = previous.iter().map(|m| m.id.clone()).collect();
    let after = id_counts(next);
    TopologyDiff {
        added: after
            .keys()
            .filter(|id| !before.contains(*id))
            .cloned()
            .collect(),
        removed: before
            .into_iter()
            .filter(|id| !after.contains_key(id))
            .collect(),
        ambiguous: after
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(id, _)| id)
            .collect(),
    }
}

#[cfg(test)]
pub(crate) mod tests;
