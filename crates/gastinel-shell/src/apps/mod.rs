//! Applications système natives intégrées pour Gastinel Desktop.

pub mod calculator;
pub mod clock;
pub mod files;
pub mod screenshot;
pub mod settings;
pub mod sysinfo;

pub use calculator::{CalcOp, Calculator};
pub use clock::{Alarm, ClockApp, ClockTab, CountdownTimer, Stopwatch};
pub use files::{FileItem, FileSort, FilesApp};
pub use screenshot::{CaptureMode, ScreenshotState, ScreenshotTool};
pub use settings::{SettingItem, SettingsApp, SettingsCategory};
pub use sysinfo::{CpuStats, GastinelMonitor, GpuStats, MemoryStats, MonitorTab};
