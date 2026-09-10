//! Workspaces and Window Management for Gastinel Desktop with Inter DA.

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect};
use gastinel_core::mem::{Handle, Pool};
use gastinel_core::render::{QuadInstance, RenderQueue};

#[derive(Clone, Debug, PartialEq)]
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
    pub const TITLEBAR_HEIGHT: u32 = 34;

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

    pub fn visible_windows(&self) -> impl Iterator<Item = (Handle<Window>, &Window)> {
        let active = self.active_workspace;
        self.windows
            .iter()
            .filter(move |(_, win)| win.workspace_id == active && !win.is_minimized)
    }

    /// Renders visible window frames, titlebars, and Inter titles into the RenderQueue.
    pub fn render(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        for (_, win) in self.visible_windows() {
            let z = if win.is_focused { 0.5 } else { 0.3 };

            // 1. Window Frame & Shadow
            let (border_color, border_w) = if win.is_focused {
                ([0.30, 0.55, 0.95, 0.85], 1.5) // Glowing focus border
            } else {
                ([0.20, 0.22, 0.26, 0.50], 1.0)
            };

            let frame = QuadInstance::rounded_box(
                win.geometry,
                10.0,
                [0.10, 0.11, 0.13, 0.98], // Opaque window background
                border_color,
                border_w,
                z,
            )
            .with_shadow(
                [0.0, 0.0, 0.0, 0.40],
                20.0,
                [0.0, 6.0],
            );
            queue.push(frame);

            // 2. Title bar
            let title_rect = Rect::new(
                win.geometry.x,
                win.geometry.y,
                win.geometry.width,
                Self::TITLEBAR_HEIGHT,
            );
            let titlebar = QuadInstance::rounded_box(
                title_rect,
                10.0,
                [0.14, 0.15, 0.18, 0.95],
                [0.22, 0.24, 0.28, 0.40],
                1.0,
                z + 0.01,
            );
            queue.push(titlebar);

            // 3. Window Title (Inter SemiBold 13px)
            atlas.layout_text(
                &win.title,
                Point::new(win.geometry.x + 14, win.geometry.y + 22),
                13,
                FontWeight::SemiBold,
                if win.is_focused {
                    [0.95, 0.96, 0.98, 1.0]
                } else {
                    [0.60, 0.63, 0.70, 1.0]
                },
                glyphs,
            );

            // 4. Window Controls: close button pill
            let close_rect = Rect::new(
                win.geometry.right() - 28,
                win.geometry.y + 11,
                12,
                12,
            );
            queue.push(QuadInstance::rounded_box(
                close_rect,
                6.0,
                [0.85, 0.25, 0.30, 0.80], // Red dot
                [0.95, 0.35, 0.40, 0.90],
                1.0,
                z + 0.02,
            ));
        }
    }
}
