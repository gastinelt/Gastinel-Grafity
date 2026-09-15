//! Embedded Desktop Shell: TopBar, Spotlight Launcher, Workspaces, Window Management, Chat IA et Apps Système.

pub mod apps;
pub mod chat;
pub mod launcher;
pub mod topbar;
pub mod wizard;
pub mod workspace;

pub use apps::{
    Alarm, CalcOp, Calculator, CaptureMode, ClockApp, ClockTab, CountdownTimer, CpuStats,
    FileItem, FileSort, FilesApp, GastinelMonitor, GpuStats, MemoryStats, MonitorTab,
    ScreenshotState, ScreenshotTool, SettingItem, SettingsApp, SettingsCategory, Stopwatch,
};
pub use chat::{ChatMessage, GemmaChatApp};
pub use launcher::{AppEntry, Launcher};
pub use topbar::TopBar;
pub use wizard::{SetupWizard, WizardStep};
pub use workspace::{Window, WorkspaceManager};
