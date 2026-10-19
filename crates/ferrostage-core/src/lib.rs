//! Presentation state machine for FerroStage.
//!
//! This crate is the domain core. It must not depend on GPUI, PDFium, a window
//! toolkit, or file I/O. Speaker notes live in `ferrostage-document` and are
//! intentionally absent from [`AudienceProjectionSnapshot`].

mod command;
mod display;
mod error;
mod frame;
mod geometry;
mod ids;
mod session;
mod slide_map;
mod snapshot;
mod state;
mod timer;
mod transition;
pub use transition::{
    AudiencePlayback, PagePresentation, PageTransition, PlaybackIdentity, TransitionDimension,
    TransitionDirection, TransitionMotion, TransitionStyle,
};

pub use command::PresentationCommand;
pub use display::{MonitorId, MonitorRole};
pub use error::CoreError;
pub use frame::BgraFrame;
pub use geometry::{LaserStyle, NormalizedRect, NormalizedStroke};
pub use ids::{DocumentGeneration, LogicalSlide, PhysicalPage};
pub use session::{DocumentDescriptor, SessionStage};
pub use slide_map::SlideMap;
pub use snapshot::{
    AudienceMediaFrame, AudienceProjectionSnapshot, AudienceStage, AudienceStagingFrame,
    AudienceVisual, BlankMode,
};
pub use state::PresentationState;
