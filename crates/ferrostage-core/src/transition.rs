//! Owned PDF presentation metadata, independent of PDF parsers and UI clocks.
use crate::{DocumentGeneration, PhysicalPage};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionStyle {
    Replace,
    Split,
    Blinds,
    Box,
    Wipe,
    Dissolve,
    Glitter,
    Fly,
    Push,
    Cover,
    Uncover,
    Fade,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionDimension {
    Horizontal,
    Vertical,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionMotion {
    Inward,
    Outward,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransitionDirection {
    Right,
    Up,
    Left,
    Down,
    DownRight,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageTransition {
    pub style: TransitionStyle,
    pub duration_seconds: f64,
    pub dimension: TransitionDimension,
    pub motion: TransitionMotion,
    pub direction: TransitionDirection,
    pub scale: f32,
    pub opaque: bool,
}
impl Default for PageTransition {
    fn default() -> Self {
        Self {
            style: TransitionStyle::Replace,
            duration_seconds: 1.,
            dimension: TransitionDimension::Horizontal,
            motion: TransitionMotion::Inward,
            direction: TransitionDirection::Right,
            scale: 1.,
            opaque: false,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PagePresentation {
    pub transition: Option<PageTransition>,
    /// Page dwell (/Dur), distinct from transition duration (/Trans /D).
    pub dwell_seconds: Option<f64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaybackIdentity {
    pub generation: DocumentGeneration,
    pub navigation: u64,
    pub page: PhysicalPage,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudiencePlayback {
    pub identity: PlaybackIdentity,
    /// Only sequential navigation authorizes an effect. All other entries settle.
    pub transition: Option<PageTransition>,
}
