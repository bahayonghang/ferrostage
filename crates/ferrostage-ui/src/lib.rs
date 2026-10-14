//! GPUI view layer.
//!
//! This skeleton does not link GPUI. The pin, once the UI task starts, is
//! Zed tag `v1.23.3` or `gpui-kit` 0.7.3 (`gpui-pre` 0.3.9). Do not depend on
//! crates.io `gpui` 0.2.x.
//!
//! Window rules:
//! - the presenter view may read notes from `ferrostage-document`
//! - the audience view may only render [`ferrostage_core::AudienceProjectionSnapshot`]
//! - neither view embeds a child HWND for video

/// Zed release that owns the GPUI revision this project will build against.
pub const ZED_GPUI_TAG: &str = "v1.23.3";

/// crates.io component library release aligned with that Zed line.
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
