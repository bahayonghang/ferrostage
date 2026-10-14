//! Presentation state machine for FerroStage.
//!
//! This crate is the domain core. It must not depend on GPUI, PDFium, a window
//! toolkit, or file I/O. Speaker notes live in `ferrostage-document` and are
//! intentionally absent from [`AudienceProjectionSnapshot`].

mod command;
mod error;
mod geometry;
mod ids;
mod snapshot;
mod state;

pub use command::PresentationCommand;
pub use error::CoreError;
pub use geometry::{NormalizedRect, NormalizedStroke};
pub use ids::{DocumentGeneration, LogicalSlide, PhysicalPage};
pub use snapshot::{AudienceProjectionSnapshot, BlankMode};
pub use state::PresentationState;
