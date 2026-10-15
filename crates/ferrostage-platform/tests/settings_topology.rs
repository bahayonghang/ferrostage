//! Gate A: actual TOML normalization feeds the shared pure resolver.
use ferrostage_core::{MonitorId, MonitorRole};
use ferrostage_platform::{
    IdentityConfidence, IdentitySource, MonitorInfo, ScaleSource, TopologyWarning, resolve_roles,
};

fn monitors() -> Vec<MonitorInfo> {
    ["a", "b", "c"]
        .into_iter()
        .map(|id| MonitorInfo {
            id: MonitorId(id.into()),
            name: id.into(),
            connection_name: id.into(),
            native_size: (1920, 1080),
            position: (0, 0),
            scale_factor: 1.0,
            scale_source: ScaleSource::GdiDeviceCapsRatio,
            rotation_degrees: 0,
            is_primary: id == "a",
            identity_source: IdentitySource::DeviceInterfacePath,
            identity_confidence: IdentityConfidence::ConnectionScoped,
        })
        .collect()
}

#[test]
fn parsed_invalid_and_duplicate_bindings_cannot_promote_an_audience() {
    let loaded = ferrostage_settings::from_toml(
        r#"
schema_version = 1
[[display.monitors]]
id = "b"
role = "typo"
[[display.monitors]]
id = "b"
role = "audience"
[[display.monitors]]
id = "a"
role = "presenter"
"#,
    )
    .unwrap();
    let bindings: Vec<_> = loaded
        .settings
        .display
        .monitors
        .iter()
        .map(|b| (MonitorId(b.id.clone()), b.role))
        .collect();
    assert_eq!(bindings[0].1, MonitorRole::None);
    let topology = resolve_roles(&monitors(), &bindings);
    assert_eq!(topology.presenter, Some(MonitorId("a".into())));
    assert_eq!(topology.audience, None);
    assert!(
        topology
            .warnings
            .contains(&TopologyWarning::DuplicateBinding(MonitorId("b".into())))
    );
}

#[test]
fn saved_roles_survive_enumeration_reordering_but_not_a_missing_identity() {
    let loaded = ferrostage_settings::from_toml(
        r#"
schema_version = 1
[[display.monitors]]
id = "c"
role = "audience"
[[display.monitors]]
id = "b"
role = "presenter"
"#,
    )
    .unwrap();
    let bindings: Vec<_> = loaded
        .settings
        .display
        .monitors
        .iter()
        .map(|b| (MonitorId(b.id.clone()), b.role))
        .collect();
    let mut monitors = monitors();
    let first = resolve_roles(&monitors, &bindings);
    monitors.reverse();
    assert_eq!(first, resolve_roles(&monitors, &bindings));
    monitors.retain(|m| m.id.0 != "c");
    let absent = resolve_roles(&monitors, &bindings);
    assert_eq!(absent.audience, None);
    assert_eq!(absent.presenter, Some(MonitorId("b".into())));
    assert!(
        absent
            .warnings
            .contains(&TopologyWarning::SavedAudienceMissing(MonitorId(
                "c".into()
            )))
    );
}
