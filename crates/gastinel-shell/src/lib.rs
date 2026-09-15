//! Embedded Desktop Shell: TopBar, Spotlight Launcher, Workspaces, Window Management et Chat IA.

pub mod chat;
pub mod launcher;
pub mod topbar;
pub mod wizard;
pub mod workspace;

pub use chat::{ChatMessage, GemmaChatApp};
pub use launcher::{AppEntry, Launcher};
pub use topbar::TopBar;
pub use wizard::{SetupWizard, WizardStep};
pub use workspace::{Window, WorkspaceManager};
