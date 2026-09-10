//! Core abstractions, zero-allocation memory pools, geometry, and event systems for Gastinel Desktop.

pub mod event;
pub mod geom;
pub mod mem;

pub use event::{Event, EventQueue, Modifiers, MouseButton};
pub use geom::{DamageRegion, Point, Rect, Size};
pub use mem::{FrameArena, Handle, Pool};
