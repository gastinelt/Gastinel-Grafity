//! Platform and kernel integrations for Alpine Linux (musl) and cross-platform development.

pub mod drm;
pub mod seatd;

pub use drm::{DrmConnector, DrmDevice, DrmOutputMode};
pub use seatd::SeatSession;
