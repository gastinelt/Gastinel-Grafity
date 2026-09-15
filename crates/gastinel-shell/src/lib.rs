//! Embedded Desktop Shell: TopBar, Spotlight Launcher, Workspaces, Window Management,
//! Notifications, Lock Screen, Alt+Tab, Menu Contextuel, Thèmes, Chat IA et Apps Système.

pub mod alttab;
pub mod apps;
pub mod chat;
pub mod contextmenu;
pub mod launcher;
pub mod lockscreen;
pub mod notifications;
pub mod theme;
pub mod topbar;
pub mod wizard;
pub mod workspace;

pub use alttab::{AltTabSwitcher, WindowThumb};
pub use apps::{
    AiDashboardApp, Alarm, Alignment, CalcOp, Calculator, CaptureMode, ClockApp, ClockTab,
    CountdownTimer, CpuStats, FileItem, FileSort, FilesApp, GastinelMonitor, GastinelWriteApp,
    GpuStats, HeadingLevel, ListKind, MemoryStats, MonitorTab, PackageCategory, PackageItem,
    PackageManagerApp, Paragraph, ScreenshotState, ScreenshotTool, SettingItem, SettingsApp,
    SettingsCategory, Stopwatch, StyleSpan, TerminalApp, TerminalTab,
};
pub use chat::{ChatMessage, GemmaChatApp};
pub use contextmenu::{ContextMenu, ContextMenuItem};
pub use launcher::{AppEntry, Launcher};
pub use lockscreen::LockScreen;
pub use notifications::{Notification, NotificationCenter, NotificationLevel};
pub use theme::{AccentColor, ThemeManager, ThemeMode};
pub use topbar::TopBar;
pub use wizard::{SetupWizard, WizardStep};
pub use workspace::{Window, WorkspaceManager};
