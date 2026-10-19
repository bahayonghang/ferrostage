use super::*;

#[test]
fn getdc_v15_real_owner_preserves_topology_and_enumeration_releases_gdi() {
    // Keep a real-owner DC alive so GetGuiResources has a nonzero baseline;
    // winsafe treats a zero count as an error. No window is created or moved.
    let desktop_window = HWND::GetDesktopWindow();
    let sentinel = desktop_window.GetDC().unwrap();
    let process = winsafe::HPROCESS::GetCurrentProcess();
    let count = || process.GetGuiResources(co::GR::GDIOBJECTS).unwrap();
    let expected = enumerate().unwrap();
    assert!(
        !expected.is_empty(),
        "this Windows resource regression needs a desktop"
    );

    let before_candidate = count();
    for _ in 0..16 {
        let devices = devices().unwrap();
        let geometry = {
            let dc = desktop_window.GetDC().unwrap();
            let mut result = Vec::new();
            dc.EnumDisplayMonitors(None, |monitor, dc, _| {
                result.push(read_geometry(monitor, dc));
                true
            })
            .unwrap();
            result.into_iter().collect::<Result<Vec<_>, _>>().unwrap()
        };
        let candidate = combine(devices, geometry).unwrap();
        assert_eq!(
            candidate, expected,
            "real owner must retain every monitor/geometry/scale field"
        );
    }
    let after_candidate = count();
    eprintln!(
        "real-owner candidate: monitors={} GDI before={before_candidate} after={after_candidate}",
        expected.len()
    );
    assert_eq!(
        after_candidate, before_candidate,
        "real-owner DC must release on each scope exit"
    );

    let before = count();
    for _ in 0..64 {
        assert_eq!(enumerate().unwrap(), expected);
    }
    let after = count();
    eprintln!("production enumeration64: GDI before={before} after={after}");
    assert_eq!(
        after, before,
        "monitor polling must not accumulate desktop DCs"
    );
    drop(sentinel);
}

fn device(path: &str) -> BTreeMap<String, Device> {
    BTreeMap::from([(
        "DISPLAY-A".into(),
        Device {
            connection: "DISPLAY-A".into(),
            path: path.into(),
            name: "same-model".into(),
            primary: true,
        },
    )])
}

fn geometry() -> Geometry {
    Geometry {
        connection: "DISPLAY-A".into(),
        position: (-1920, -100),
        size: (1920, 1080),
        rotation: 0,
        scale: 1.5,
        primary: true,
    }
}

#[test]
fn changing_device_metadata_is_not_a_geometry_mismatch() {
    let before = device("path-before");
    for field in ["path", "name", "primary"] {
        let mut after = before.clone();
        let changed = after.get_mut("DISPLAY-A").unwrap();
        match field {
            "path" => changed.path = "path-after".into(),
            "name" => changed.name = "renamed-model".into(),
            "primary" => changed.primary = false,
            _ => unreachable!(),
        }
        assert!(before.keys().eq(after.keys()));
        assert_eq!(
            combine_snapshots(before.clone(), vec![geometry()], after),
            Err(PlatformError::TopologyChanging {
                before: vec!["DISPLAY-A".into()],
                after: vec!["DISPLAY-A".into()],
            }),
            "{field} change must not be reported as missing geometry"
        );
    }
}

#[test]
fn stable_device_snapshots_still_require_complete_geometry() {
    assert_eq!(
        combine_snapshots(device("path"), vec![], device("path")),
        Err(PlatformError::IncompleteTopology {
            devices: vec!["DISPLAY-A".into()],
            geometry: vec![],
        })
    );
    assert_eq!(
        combine_snapshots(device("path"), vec![geometry()], device("path")),
        combine(device("path"), vec![geometry()])
    );
}

#[test]
fn every_required_native_mode_field_must_be_present() {
    let required =
        co::DM::POSITION | co::DM::DISPLAYORIENTATION | co::DM::PELSWIDTH | co::DM::PELSHEIGHT;
    let mut mode = DEVMODE::default();
    mode.dmFields = required | co::DM::BITSPERPEL;
    mode.dmPelsWidth = 1920;
    mode.dmPelsHeight = 1080;
    assert!(geometry_from_mode("DISPLAY-A".into(), &mode, 1920, 1280, true).is_ok());

    for missing in [
        co::DM::POSITION,
        co::DM::DISPLAYORIENTATION,
        co::DM::PELSWIDTH,
        co::DM::PELSHEIGHT,
    ] {
        mode.dmFields = required & !missing;
        assert!(
            matches!(
                geometry_from_mode("DISPLAY-A".into(), &mode, 1920, 1280, true),
                Err(PlatformError::InvalidGeometry { .. })
            ),
            "accepted native mode without {missing:?}"
        );
    }
}

#[test]
fn omitted_or_extra_or_duplicate_geometry_is_rejected() {
    assert!(matches!(
        combine(device("path"), vec![]),
        Err(PlatformError::IncompleteTopology { .. })
    ));
    assert!(matches!(
        combine(BTreeMap::new(), vec![geometry()]),
        Err(PlatformError::IncompleteTopology { .. })
    ));
    assert!(matches!(
        combine(device("path"), vec![geometry(), geometry()]),
        Err(PlatformError::AmbiguousConnection { .. })
    ));
    assert_eq!(combine(BTreeMap::new(), vec![]), Ok(vec![]));
}

#[test]
fn invalid_scale_size_or_primary_is_not_accepted() {
    for scale in [f32::NAN, f32::INFINITY, 0.0, -1.0] {
        let mut info = geometry();
        info.scale = scale;
        assert!(matches!(
            combine(device("path"), vec![info]),
            Err(PlatformError::InvalidGeometry { .. })
        ));
    }
    let mut info = geometry();
    info.size.0 = 0;
    assert!(combine(device("path"), vec![info]).is_err());
    let mut info = geometry();
    info.primary = false;
    assert!(combine(device("path"), vec![info]).is_err());
}

#[test]
fn output_owns_physical_units_and_does_not_promote_handle_or_missing_path() {
    let infos = combine(device(""), vec![geometry()]).unwrap();
    assert_eq!(infos[0].position, (-1920, -100));
    assert_eq!(infos[0].native_size, (1920, 1080));
    assert_eq!(infos[0].scale_factor, 1.5);
    assert_eq!(
        infos[0].identity_confidence,
        crate::IdentityConfidence::Degraded
    );
    assert!(!infos[0].id.0.contains("99"));
    assert_eq!(infos[0].name, "same-model");
    assert!(!infos[0].built_in);
}
