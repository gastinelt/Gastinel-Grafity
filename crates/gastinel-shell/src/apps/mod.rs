//! Applications système natives intégrées pour Gastinel Desktop.

pub mod aidashboard;
pub mod calculator;
pub mod clock;
pub mod files;
pub mod packagemanager;
pub mod screenshot;
pub mod settings;
pub mod sysinfo;
pub mod terminal;
pub mod write;

pub use aidashboard::AiDashboardApp;
pub use calculator::{CalcOp, Calculator};
pub use clock::{Alarm, ClockApp, ClockTab, CountdownTimer, Stopwatch};
pub use files::{FileItem, FileSort, FilesApp};
pub use packagemanager::{PackageCategory, PackageItem, PackageManagerApp};
pub use screenshot::{CaptureMode, ScreenshotState, ScreenshotTool};
pub use settings::{SettingItem, SettingsApp, SettingsCategory};
pub use sysinfo::{CpuStats, GastinelMonitor, GpuStats, MemoryStats, MonitorTab};
pub use terminal::{TerminalApp, TerminalTab};
pub use write::{Alignment, GastinelWriteApp, HeadingLevel, ListKind, Paragraph, StyleSpan};
