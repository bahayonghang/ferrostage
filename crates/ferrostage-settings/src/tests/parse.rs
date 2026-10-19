use ferrostage_core::MonitorRole;
use ferrostage_render::{L1_GPU_BUDGET_MIB, L2_CPU_BUDGET_MIB, L3_THUMB_BUDGET_MIB};

use crate::from_toml;
use crate::schema::{BindingIssue, SECTION_NAMES, Settings, SingleMonitorMode};

use super::Scratch;

#[test]
fn empty_document_is_the_default() {
    let outcome = from_toml("").expect("empty toml");
    assert_eq!(outcome.settings, Settings::default());
    assert!(outcome.warnings.is_empty());
    assert!(outcome.write_policy.is_writable());
    assert!(outcome.needs_writeback);
}

#[test]
fn default_toml_roundtrip_matches_default() {
    let text = toml::to_string(&Settings::default()).expect("serialize");
    let outcome = from_toml(&text).expect("parse");
    assert_eq!(outcome.settings, Settings::default());
    assert!(outcome.warnings.is_empty());
    assert!(!outcome.needs_writeback);
}

#[test]
fn defaults_use_render_cache_budgets() {
    let cache = Settings::default().cache;
    assert_eq!(cache.l1_gpu_mib, L1_GPU_BUDGET_MIB);
    assert_eq!(cache.l2_cpu_mib, L2_CPU_BUDGET_MIB);
    assert_eq!(cache.l3_thumb_mib, L3_THUMB_BUDGET_MIB);
    assert_eq!(Settings::default().hot_reload.debounce_ms, 300);
    assert!(!Settings::default().general.debug_overlay);
    assert!(Settings::default().general.recent_files.is_empty());
    assert_eq!(Settings::default().presenter.grid_columns, 4);
    assert_eq!(Settings::default().presenter.grid_visible_pages, 16);
    assert_eq!(
        Settings::default().display.single_monitor_mode,
        SingleMonitorMode::PresenterOnly
    );
}

#[test]
fn deleting_any_section_keeps_the_other_values() {
    let mut settings = Settings::default();
    settings.general.language = "en-US".to_owned();
    settings.cache.l1_gpu_mib = 32;
    settings.timer.target_minutes = 12;
    settings.presenter.grid_columns = 2;
    settings.display.audience_fullscreen = false;
    settings.pointer.highlighter_alpha = 0.25;
    settings.hot_reload.debounce_ms = 450;
    settings.media.max_decode_height = 720;
    settings.keybindings.jump = vec!["J".to_owned()];
    let text = toml::to_string(&settings).expect("serialize");
    let mut document: toml::Table = text.parse().expect("table");
    for name in SECTION_NAMES {
        let mut stripped = document.clone();
        stripped.remove(name);
        let parsed = from_toml(&stripped.to_string()).expect("parse");
        assert!(parsed.warnings.is_empty(), "{name}");
        if name != "media" {
            assert_eq!(parsed.settings.media.max_decode_height, 720);
        }
        match name {
            "general" => {
                assert_eq!(parsed.settings.general, Settings::default().general);
                assert_eq!(parsed.settings.cache.l1_gpu_mib, 32);
            }
            "cache" => {
                assert_eq!(parsed.settings.cache, Settings::default().cache);
                assert_eq!(parsed.settings.general.language, "en-US");
            }
            "timer" => {
                assert_eq!(parsed.settings.timer, Settings::default().timer);
                assert_eq!(parsed.settings.general.language, "en-US");
            }
            "presenter" => {
                assert_eq!(parsed.settings.presenter, Settings::default().presenter);
                assert_eq!(parsed.settings.timer.target_minutes, 12);
            }
            "display" => {
                assert_eq!(parsed.settings.display, Settings::default().display);
                assert_eq!(parsed.settings.presenter.grid_columns, 2);
            }
            "pointer" => {
                assert_eq!(parsed.settings.pointer, Settings::default().pointer);
                assert!(!parsed.settings.display.audience_fullscreen);
            }
            "hot_reload" => {
                assert_eq!(parsed.settings.hot_reload, Settings::default().hot_reload);
                assert_eq!(parsed.settings.pointer.highlighter_alpha, 0.25);
            }
            "keybindings" => {
                assert_eq!(parsed.settings.keybindings, Settings::default().keybindings);
                assert_eq!(parsed.settings.hot_reload.debounce_ms, 450);
            }
            "media" => {
                assert_eq!(parsed.settings.media, Settings::default().media);
                assert_eq!(parsed.settings.hot_reload.debounce_ms, 450);
            }
            _ => panic!("unexpected section {name}"),
        }
    }
    document.remove("timer");
    assert!(from_toml(&document.to_string()).is_ok());
}

#[test]
fn zero_target_minutes_falls_back_and_names_the_key() {
    let outcome = from_toml(
        r#"
        schema_version = 1
        [timer]
        target_minutes = 0
        warn_yellow_remaining_min = 8
        warn_red_remaining_min = 2
        "#,
    )
    .expect("parse");
    assert_eq!(outcome.settings.timer.target_minutes, 45);
    assert_eq!(outcome.settings.timer.warn_yellow_remaining_min, 8);
    assert_eq!(outcome.settings.timer.warn_red_remaining_min, 2);
    assert!(
        outcome
            .warnings
            .iter()
            .any(|warning| warning.key == "timer.target_minutes"
                && warning.reason.contains("timer.target_minutes"))
    );
}

#[test]
fn illegal_target_type_falls_back_only_that_leaf() {
    let outcome = from_toml(
        r#"
        schema_version = 1
        [timer]
        target_minutes = true
        warn_yellow_remaining_min = 8
        warn_red_remaining_min = 2
        [cache]
        l1_gpu_mib = 32
        "#,
    )
    .expect("parse");
    assert_eq!(outcome.settings.timer.target_minutes, 45);
    assert_eq!(outcome.settings.timer.warn_yellow_remaining_min, 8);
    assert_eq!(outcome.settings.cache.l1_gpu_mib, 32);
    assert!(
        outcome
            .warnings
            .iter()
            .any(|warning| warning.key == "timer.target_minutes")
    );
}

#[test]
fn wrong_section_type_falls_back_only_that_section() {
    let outcome = from_toml(
        r#"
        schema_version = 1
        timer = "nope"
        [cache]
        l1_gpu_mib = 32
        "#,
    )
    .expect("parse");
    assert_eq!(outcome.settings.timer, Settings::default().timer);
    assert_eq!(outcome.settings.cache.l1_gpu_mib, 32);
    assert!(
        outcome
            .warnings
            .iter()
            .any(|warning| warning.key == "timer")
    );
}

#[test]
fn illegal_role_becomes_none_not_audience() {
    let outcome = from_toml(
        r#"
        schema_version = 1
        [display]
        single_monitor_mode = "windowed"
        [[display.monitors]]
        id = "A"
        friendly_name = "One"
        role = "Audience"
        [[display.monitors]]
        id = "B"
        friendly_name = "Two"
        role = "audience"
        "#,
    )
    .expect("parse");
    assert_eq!(
        outcome.settings.display.single_monitor_mode,
        SingleMonitorMode::PresenterOnly
    );
    assert_eq!(outcome.settings.display.monitors[0].role, MonitorRole::None);
    assert_eq!(
        outcome.settings.display.monitors[1].role,
        MonitorRole::Audience
    );
    assert!(outcome.warnings.iter().any(
        |warning| warning.key == "display.monitors[0].role" && warning.reason.contains("none")
    ));
}

#[test]
fn legacy_flat_keys_map_and_indexes_stay_unknown() {
    let outcome = from_toml(
        r#"
        presenter_monitor = 1
        audience_monitor = 2
        cache_l1_mib = 128
        cache_l2_mib = 64
        cache_l3_mib = 32
        hot_reload_debounce_ms = 450
        "#,
    )
    .expect("parse");
    assert_eq!(outcome.settings.cache.l1_gpu_mib, 128);
    assert_eq!(outcome.settings.cache.l2_cpu_mib, 64);
    assert_eq!(outcome.settings.cache.l3_thumb_mib, 32);
    assert_eq!(outcome.settings.hot_reload.debounce_ms, 450);
    assert!(outcome.settings.display.monitors.is_empty());
    assert!(
        outcome
            .consumed_legacy_keys
            .contains(&"cache_l1_mib".to_owned())
    );
    assert!(
        outcome
            .consumed_legacy_keys
            .contains(&"hot_reload_debounce_ms".to_owned())
    );
    assert!(
        !outcome
            .consumed_legacy_keys
            .iter()
            .any(|key| key.contains("monitor"))
    );
    assert!(outcome.raw_table.contains_key("presenter_monitor"));
    assert!(outcome.warnings.iter().any(|warning| {
        warning.key == "presenter_monitor" && warning.reason.contains("rebind")
    }));
    assert!(
        outcome.warnings.iter().any(|warning| {
            warning.key == "audience_monitor" && warning.reason.contains("rebind")
        })
    );
}

#[test]
fn recent_files_are_deduped_and_truncated() {
    let outcome = from_toml(
        r#"
        schema_version = 1
        [general]
        recent_files_limit = 2
        recent_files = ["a", "b", "a", "c"]
        "#,
    )
    .expect("parse");
    assert_eq!(
        outcome.settings.general.recent_files,
        vec!["a".to_owned(), "b".to_owned()]
    );
}

#[test]
fn recent_opened_times_are_optional_and_validate_each_leaf() {
    let mut old: toml::Table = toml::to_string(&Settings::default())
        .unwrap()
        .parse()
        .unwrap();
    old.get_mut("general")
        .unwrap()
        .as_table_mut()
        .unwrap()
        .remove("recent_opened_at");
    let loaded = from_toml(&old.to_string()).unwrap();
    assert!(loaded.settings.general.recent_opened_at.is_empty());
    assert!(loaded.warnings.is_empty());
    assert!(!loaded.needs_writeback);

    for invalid in ["false", "42", "[]", "'yesterday'"] {
        let loaded = from_toml(&format!(
            "[general]\nrecent_files = ['a']\nrecent_opened_at = {invalid}"
        ))
        .unwrap();
        assert_eq!(loaded.settings.general.recent_files, ["a"]);
        assert!(loaded.settings.general.recent_opened_at.is_empty());
        assert!(
            loaded
                .warnings
                .iter()
                .any(|warning| { warning.key == "general.recent_opened_at" })
        );
    }
    let loaded = from_toml(
        "[general]\nrecent_files_limit = 5\nrecent_files = ['epoch', 'large', 'negative', 'text', 'float']\n\
         [general.recent_opened_at]\nepoch = 0\nlarge = 9223372036854775807\nnegative = -1\ntext = 'today'\nfloat = 1.5",
    )
    .unwrap();
    assert_eq!(
        loaded.settings.general.recent_opened_at,
        [("epoch".to_owned(), 0), ("large".to_owned(), i64::MAX)].into()
    );
    for key in ["negative", "text", "float"] {
        assert!(
            loaded
                .warnings
                .iter()
                .any(|warning| { warning.key == format!("general.recent_opened_at.{key}") })
        );
    }
    assert_eq!(loaded.warnings.len(), 3);
}

#[test]
fn recent_opened_times_follow_effective_deduplicated_path_limit() {
    let loaded = from_toml(
        "[general]\nrecent_files_limit = 2\nrecent_files = ['a', 'b', 'a', 'c']\n\
         [general.recent_opened_at]\na = 100\nb = 200\nc = 300\norphan = 400",
    )
    .unwrap();
    assert_eq!(loaded.settings.general.recent_files, ["a", "b"]);
    assert_eq!(
        loaded.settings.general.recent_opened_at,
        [("a".to_owned(), 100), ("b".to_owned(), 200)].into()
    );
    assert!(loaded.warnings.is_empty());
}

#[test]
fn record_recent_file_updates_one_time_and_prunes_history() {
    let mut general = Settings::default().general;
    general.recent_files_limit = 3;
    general.recent_files = vec!["a".into(), "a".into(), "b".into(), "c".into()];
    general.recent_opened_at = [("a".into(), 10), ("b".into(), 20), ("orphan".into(), 30)].into();
    general.record_recent_file("b".into(), 100);
    assert_eq!(general.recent_files, ["b", "a", "c"]);
    assert_eq!(
        general.recent_opened_at,
        [("a".into(), 10), ("b".into(), 100)].into()
    );

    general.recent_files_limit = 2;
    general.record_recent_file("new".into(), 200);
    assert_eq!(general.recent_files, ["new", "b"]);
    assert_eq!(
        general.recent_opened_at,
        [("new".into(), 200), ("b".into(), 100)].into()
    );
    general.record_recent_file("new".into(), -1);
    assert!(!general.recent_opened_at.contains_key("new"));
    assert_eq!(general.recent_opened_at.get("b"), Some(&100));
}

#[test]
fn keybinding_conflicts_are_recorded_and_not_resolved() {
    let outcome = from_toml(
        r#"
        schema_version = 1
        [keybindings]
        next = ["Space", "Space", ""]
        previous = ["Space"]
        "#,
    )
    .expect("parse");
    assert_eq!(
        outcome.settings.keybindings.next,
        vec!["Space".to_owned(), "Space".to_owned(), String::new()]
    );
    assert_eq!(
        outcome.settings.keybindings.previous,
        vec!["Space".to_owned()]
    );
    let issues = outcome.settings.keybindings.issues();
    assert!(
        issues
            .iter()
            .any(|issue| matches!(issue, BindingIssue::Empty { action: "next" }))
    );
    assert!(
        issues
            .iter()
            .any(|issue| matches!(issue, BindingIssue::Duplicate { action: "next", .. }))
    );
    assert!(issues.iter().any(|issue| matches!(
        issue,
        BindingIssue::Conflict {
            actions: ["next", "previous"],
            ..
        }
    )));
    assert!(
        outcome
            .warnings
            .iter()
            .any(|warning| warning.reason.contains("must not run both actions"))
    );
}

#[test]
fn future_version_is_read_only() {
    let outcome = from_toml(
        r#"
        schema_version = 2
        [general]
        language = "en-US"
        extra = "keep"
        "#,
    )
    .expect("parse");
    assert_eq!(outcome.settings.schema_version, 2);
    assert_eq!(outcome.settings.general.language, "en-US");
    assert!(!outcome.write_policy.is_writable());
    assert!(!outcome.needs_writeback);
    assert_eq!(outcome.raw_table["general"]["extra"].as_str(), Some("keep"));
}

#[test]
fn invalid_keybinding_elements_fall_back_only_that_action() {
    for next in ["[123]", "[\"N\", false]"] {
        let outcome = from_toml(&format!(
            "[keybindings]\nnext = {next}\nprevious = [\"Q\"]\n"
        ))
        .expect("parse");
        assert_eq!(
            outcome.settings.keybindings.next,
            Settings::default().keybindings.next
        );
        assert_eq!(outcome.settings.keybindings.previous, vec!["Q".to_owned()]);
        assert!(
            outcome
                .warnings
                .iter()
                .any(|warning| warning.key == "keybindings.next")
        );
    }
}

#[test]
fn explicitly_empty_keybinding_list_stays_unbound() {
    let outcome = from_toml("[keybindings]\nnext = []\n").expect("parse");
    assert!(outcome.settings.keybindings.next.is_empty());
    assert!(outcome.warnings.is_empty());
}

#[test]
fn invalid_syntax_is_an_error_for_pure_parse() {
    assert!(from_toml("cache_l1_mib = ").is_err());
}

#[test]
fn manifest_does_not_link_platform_or_gpui() {
    let manifest = include_str!("../../Cargo.toml");
    for forbidden in ["ferrostage-platform", "gpui", "lopdf", "notify", "pdfium"] {
        assert!(
            !manifest.contains(forbidden),
            "ferrostage-settings must not depend on {forbidden}"
        );
    }
}

#[test]
fn scratch_dirs_are_not_the_user_config() {
    let scratch = Scratch::new("guard");
    let appdata = std::env::var_os("APPDATA");
    if let Some(appdata) = appdata {
        assert!(!scratch.path().starts_with(appdata));
    }
}

#[test]
fn yellow_must_exceed_red() {
    let outcome = from_toml(
        r#"
        schema_version = 1
        [timer]
        warn_yellow_remaining_min = 2
        warn_red_remaining_min = 2
        "#,
    )
    .expect("parse");
    assert_eq!(outcome.settings.timer.warn_yellow_remaining_min, 10);
    assert_eq!(outcome.settings.timer.warn_red_remaining_min, 3);
    assert!(
        outcome
            .warnings
            .iter()
            .any(|warning| warning.key == "timer.warn_yellow_remaining_min")
    );
}

#[test]
fn color_and_ratio_ranges_fall_back_one_leaf() {
    let outcome = from_toml(
        r#"
        schema_version = 1
        [pointer]
        laser_color_bgra = [0, 0, 255]
        pen_color_bgra = [1, 2, 3, 4]
        highlighter_alpha = 2
        [presenter]
        next_slide_ratio = 5
        notes_ratio = 0.4
        grid_columns = 8
        grid_visible_pages = 8
        "#,
    )
    .expect("parse");
    assert_eq!(
        outcome.settings.pointer.laser_color_bgra,
        Settings::default().pointer.laser_color_bgra
    );
    assert_eq!(outcome.settings.pointer.pen_color_bgra, [1, 2, 3, 4]);
    assert_eq!(outcome.settings.pointer.highlighter_alpha, 0.5);
    assert_eq!(outcome.settings.presenter.next_slide_ratio, 0.40);
    assert_eq!(outcome.settings.presenter.notes_ratio, 0.4);
    assert_eq!(outcome.settings.presenter.grid_columns, 4);
    assert_eq!(outcome.settings.presenter.grid_visible_pages, 8);
}

#[test]
fn media_v1_defaults_bounds_unknown_and_future_readonly() {
    assert_eq!(
        from_toml("schema_version=1")
            .unwrap()
            .settings
            .media
            .max_decode_height,
        1080
    );
    for n in [0, 2161] {
        let outcome = from_toml(&format!(
            "schema_version=1\n[media]\nmax_decode_height={n}\nunknown='keep'\n"
        ))
        .unwrap();
        assert_eq!(outcome.settings.media.max_decode_height, 1080);
        assert!(
            outcome
                .warnings
                .iter()
                .any(|w| w.key == "media.max_decode_height")
        );
        let merged =
            crate::merge::merge_document(&outcome.raw_table, &outcome.settings, &[]).unwrap();
        assert_eq!(merged["media"]["unknown"].as_str(), Some("keep"));
    }
    let future = from_toml("schema_version=99\n[media]\nmax_decode_height=720\n").unwrap();
    assert!(!future.write_policy.is_writable());
    assert_eq!(future.settings.media.max_decode_height, 720);
}
#[test]
fn legacy_notes_ratio_and_strip_are_readable_without_affecting_notes_column() {
    let parsed =
        crate::from_toml("[presenter]\nnotes_ratio=0.4\nshow_thumbnail_strip=true").unwrap();
    assert_eq!(parsed.settings.presenter.notes_ratio, 0.4);
    assert!(parsed.settings.presenter.show_thumbnail_strip);
    assert_eq!(parsed.settings.presenter.notes_column_ratio, 0.60);
    assert_eq!(
        parsed.settings.presenter.effective_notes_column_ratio(),
        0.60
    );
}

#[test]
fn notes_column_ratio_clamps_and_round_trips_with_legacy_and_unknown_leaves() {
    let parsed = crate::from_toml(
        "schema_version=1\n[presenter]\nnext_slide_ratio=0.35\nnotes_ratio=0.4\nnotes_column_ratio=0.3\nunknown='keep'\n",
    )
    .unwrap();
    assert_eq!(parsed.settings.presenter.notes_column_ratio, 0.3);
    assert_eq!(
        parsed.settings.presenter.effective_notes_column_ratio(),
        0.40
    );
    let wide = crate::PresenterSettings {
        notes_column_ratio: 0.9,
        ..crate::PresenterSettings::default()
    };
    assert_eq!(wide.effective_notes_column_ratio(), 0.75);
    let mut settings = parsed.settings.clone();
    settings.presenter.notes_column_ratio = 0.66;
    let merged = crate::merge::merge_document(&parsed.raw_table, &settings, &[]).unwrap();
    assert_eq!(
        merged["presenter"]["notes_column_ratio"].as_float(),
        Some(0.66)
    );
    assert_eq!(
        merged["presenter"]["next_slide_ratio"].as_float(),
        Some(0.35)
    );
    assert_eq!(merged["presenter"]["notes_ratio"].as_float(), Some(0.4));
    assert_eq!(merged["presenter"]["unknown"].as_str(), Some("keep"));
}

#[test]
fn interface_font_leaves_default_validate_and_preserve_future_policy() {
    let old = from_toml("schema_version = 1\n[general]\nlanguage = 'en-US'").unwrap();
    assert_eq!(old.settings.general.ui_font_family, "");
    assert_eq!(old.settings.general.ui_font_size, 16);
    for invalid in ["11", "25", "16.5", "'large'", "-1"] {
        let result = from_toml(&format!(
            "[general]\nui_font_size = {invalid}\nui_font_family = ' Missing Family '"
        ))
        .unwrap();
        assert_eq!(result.settings.general.ui_font_size, 16);
        assert_eq!(result.settings.general.ui_font_family, "Missing Family");
        assert!(
            result
                .warnings
                .iter()
                .any(|warning| warning.key == "general.ui_font_size")
        );
    }
    for invalid in ["42", "true", "\"bad\\nfamily\""] {
        let result = from_toml(&format!(
            "[general]\nui_font_family = {invalid}\nui_font_size = 24"
        ))
        .unwrap();
        assert_eq!(result.settings.general.ui_font_family, "");
        assert_eq!(result.settings.general.ui_font_size, 24);
        assert!(
            result
                .warnings
                .iter()
                .any(|warning| warning.key == "general.ui_font_family")
        );
    }
    let future =
        from_toml("schema_version = 99\n[general]\nui_font_size = 12\nui_font_family = 'Example'")
            .unwrap();
    assert!(!future.write_policy.is_writable());
    assert_eq!(future.settings.general.ui_font_size, 12);
}
#[test]
fn audience_click_leaf_defaults_validates_and_preserves_future_policy() {
    for version in [1, 999] {
        for (leaf, enabled, warns) in [
            ("", true, false),
            ("audience_click_to_advance = true", true, false),
            ("audience_click_to_advance = false", false, false),
            ("audience_click_to_advance = 'false'", true, true),
            ("audience_click_to_advance = 0", true, true),
        ] {
            let loaded = from_toml(&format!(
                "schema_version = {version}\n[display]\naudience_fullscreen = false\n{leaf}"
            ))
            .unwrap();
            assert_eq!(loaded.settings.display.audience_click_to_advance, enabled);
            assert!(!loaded.settings.display.audience_fullscreen);
            assert_eq!(loaded.write_policy.is_writable(), version == 1);
            assert_eq!(
                loaded
                    .warnings
                    .iter()
                    .any(|w| w.key == "display.audience_click_to_advance"),
                warns
            );
        }
    }
}
