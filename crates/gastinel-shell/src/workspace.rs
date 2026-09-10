//! Workspaces and Window Management for Gastinel Desktop.

use gastinel_core::geom::Rect;
use gastinel_core::mem::{Handle, Pool};

#[derive(Clone, Debug)]
pub struct Window {
    pub title: String,
    pub geometry: Rect,
    pub workspace_id: u32,
    pub is_focused: bool,
    pub is_minimized: bool,
    pub is_fullscreen: bool,
}

pub struct WorkspaceManager {
    pub active_workspace: u32,
    pub workspace_count: u32,
    pub windows: Pool<Window>,
    pub focused_window: Option<Handle<Window>>,
}

impl WorkspaceManager {
    pub fn new(workspace_count: u32) -> Self {
        Self {
            active_workspace: 1,
            workspace_count,
            windows: Pool::with_capacity(32),
            focused_window: None,
        }
    }

    pub fn create_window(&mut self, title: &str, geometry: Rect) -> Handle<Window> {
        let win = Window {
            title: title.to_string(),
            geometry,
            workspace_id: self.active_workspace,
            is_focused: true,
            is_minimized: false,
            is_fullscreen: false,
        };

        let handle = self.windows.insert(win);
        self.set_focus(Some(handle));
        handle
    }

    pub fn destroy_window(&mut self, handle: Handle<Window>) -> Option<Window> {
        if self.focused_window == Some(handle) {
            self.focused_window = None;
        }
        self.windows.remove(handle)
    }

    pub fn set_focus(&mut self, handle: Option<Handle<Window>>) {
        if let Some(prev) = self.focused_window {
            if let Some(win) = self.windows.get_mut(prev) {
                win.is_focused = false;
            }
        }

        self.focused_window = handle;

        if let Some(curr) = handle {
            if let Some(win) = self.windows.get_mut(curr) {
                win.is_focused = true;
            }
        }
    }

    pub fn switch_workspace(&mut self, ws: u32) {
        if ws >= 1 && ws <= self.workspace_count {
            self.active_workspace = ws;

            // Pick a focused window in this workspace if any
            let new_focus = self
                .windows
                .iter()
                .find(|(_, win)| win.workspace_id == ws && !win.is_minimized)
                .map(|(h, _)| h);

            self.set_focus(new_focus);
        }
    }

    pub fn move_window_to_workspace(&mut self, handle: Handle<Window>, ws: u32) {
        if let Some(win) = self.windows.get_mut(handle) {
            win.workspace_id = ws;
            if self.active_workspace != ws && self.focused_window == Some(handle) {
                self.focused_window = None;
            }
        }
    }

    /// Iterates over all visible windows on the active workspace.
    pub fn visible_windows(&self) -> impl Iterator<Item = (Handle<Window>, &Window)> {
        let active = self.active_workspace;
        self.windows
            .iter()
            .filter(move |(_, win)| win.workspace_id == active && !win.is_minimized)
    }
}
