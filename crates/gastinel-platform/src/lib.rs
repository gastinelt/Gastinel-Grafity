//! Intégrations plateforme pour Alpine Linux (musl) et développement cross-platform.

pub mod drm;
pub mod seatd;

#[cfg(target_os = "windows")]
pub mod win32;

pub use drm::{DrmConnector, DrmDevice, DrmOutputMode};
pub use seatd::SeatSession;

#[cfg(target_os = "windows")]
pub use win32::{Win32Event, Win32Window};
