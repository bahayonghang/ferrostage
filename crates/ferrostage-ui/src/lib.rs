//! GPUI view layer.
//!
//! The optional native feature uses the reviewed maintenance source based on
//! Zed tag `v1.23.3`, with provenance in vendor/README.md.
//!
//! Window rules:
//! - the presenter view may read notes from `ferrostage-document`
//! - the audience view may only render [`ferrostage_core::AudienceProjectionSnapshot`]
//! - neither view embeds a child HWND for video

#[cfg(feature = "gpui")]
mod actions;
#[cfg(feature = "gpui")]
mod audience;
pub mod display_settings;
#[cfg(feature = "gpui")]
mod document_load;
#[cfg(feature = "gpui")]
mod identify;
pub mod locale;
#[cfg(feature = "gpui")]
mod page_browser;
pub mod policy;
#[cfg(feature = "gpui")]
mod presenter;
pub mod settings_panel;
#[cfg(feature = "gpui")]
mod shell;
#[cfg(feature = "gpui")]
mod slide_surface;
#[cfg(feature = "gpui")]
mod texture_cache;
pub mod theme;
pub mod transactions;
pub mod workbench;
#[cfg(feature = "gpui")]
pub use shell::{ShellInputs, run};

/// Zed release that owns the GPUI revision this project will build against.
pub const ZED_GPUI_TAG: &str = "v1.23.3";

/// Allowed alternative component release, with its own distinct upstream revision.
pub const GPUI_KIT_VERSION: &str = "0.7.3";

/// Which window is being described. Swap exchanges these roles, not the snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowRole {
    Presenter,
    Audience,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpui_pin_is_the_zed_tag_not_crates_io() {
        assert_eq!(ZED_GPUI_TAG, "v1.23.3");
        assert_eq!(GPUI_KIT_VERSION, "0.7.3");
        assert_ne!(WindowRole::Presenter, WindowRole::Audience);
    }
}

#[cfg(feature = "gpui")]
mod media_playback;
