//! Embedded Desktop Shell: TopBar, Spotlight Launcher, Workspaces, and Window Management.

pub mod launcher;
pub mod topbar;
pub mod wizard;
pub mod workspace;

pub use launcher::{AppEntry, Launcher};
pub use topbar::TopBar;
pub use wizard::{SetupWizard, WizardStep};
pub use workspace::{Window, WorkspaceManager};
