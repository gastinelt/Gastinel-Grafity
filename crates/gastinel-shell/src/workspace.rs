//! Workspaces and Window Management for Gastinel Desktop with Inter DA.

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect};
use gastinel_core::mem::{Handle, Pool};
use gastinel_core::render::{QuadInstance, RenderQueue};

/// État de l'interaction souris en cours sur une fenêtre.
#[derive(Clone, Debug, PartialEq)]
pub enum DragState {
    /// Aucun glissement en cours.
    None,
    /// Déplacement de la fenêtre (offset initial du clic dans la barre de titre).
    Moving {
        handle: Handle<Window>,
        offset_x: i32,
        offset_y: i32,
    },
    /// Redimensionnement depuis un bord ou coin.
    Resizing {
        handle: Handle<Window>,
        edge: ResizeEdge,
        initial_rect: Rect,
        start_pos: Point,
    },
}

/// Bord ou coin utilisé pour le redimensionnement.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ResizeEdge {
    Right,
    Bottom,
    BottomRight,
}

/// Zone de snap (accrochage) détectée quand la souris touche un bord de l'écran.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SnapZone {
    /// Moitié gauche de l'écran.
    Left,
    /// Moitié droite de l'écran.
    Right,
    /// Plein écran (bord supérieur).
    Maximize,
}

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
    pub drag_state: DragState,
    pub screen_width: u32,
    pub screen_height: u32,
}

impl WorkspaceManager {
    pub const TITLEBAR_HEIGHT: u32 = 34;

    pub fn new(workspace_count: u32) -> Self {
        Self {
            active_workspace: 1,
            workspace_count,
            windows: Pool::with_capacity(32),
            focused_window: None,
            drag_state: DragState::None,
            screen_width: 1920,
            screen_height: 1080,
        }
    }

    pub fn with_screen_size(mut self, width: u32, height: u32) -> Self {
        self.screen_width = width;
        self.screen_height = height;
        self
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

    /// Taille de la zone de grip pour le redimensionnement (pixels).
    const RESIZE_GRIP: i32 = 8;

    /// Seuil en pixels pour déclencher le snap aux bords de l'écran.
    const SNAP_THRESHOLD: i32 = 12;

    /// Taille minimum d'une fenêtre (largeur et hauteur).
    const MIN_WINDOW_SIZE: u32 = 200;

    /// Détecte la zone de snap en fonction de la position de la souris.
    pub fn detect_snap_zone(&self, pos: Point) -> Option<SnapZone> {
        if pos.y <= Self::SNAP_THRESHOLD {
            Some(SnapZone::Maximize)
        } else if pos.x <= Self::SNAP_THRESHOLD {
            Some(SnapZone::Left)
        } else if pos.x >= self.screen_width as i32 - Self::SNAP_THRESHOLD {
            Some(SnapZone::Right)
        } else {
            None
        }
    }

    /// Applique le snap à une fenêtre selon la zone détectée.
    pub fn apply_snap(&mut self, handle: Handle<Window>, zone: SnapZone) {
        if let Some(win) = self.windows.get_mut(handle) {
            let topbar_h = 34i32; // Hauteur de la TopBar
            match zone {
                SnapZone::Left => {
                    win.geometry = Rect::new(
                        0,
                        topbar_h,
                        self.screen_width / 2,
                        self.screen_height - topbar_h as u32,
                    );
                }
                SnapZone::Right => {
                    let half_w = self.screen_width / 2;
                    win.geometry = Rect::new(
                        half_w as i32,
                        topbar_h,
                        self.screen_width - half_w,
                        self.screen_height - topbar_h as u32,
                    );
                }
                SnapZone::Maximize => {
                    win.geometry = Rect::new(
                        0,
                        topbar_h,
                        self.screen_width,
                        self.screen_height - topbar_h as u32,
                    );
                    win.is_fullscreen = true;
                }
            }
        }
    }

    /// Détecte le bord de redimensionnement sous le curseur pour une fenêtre donnée.
    fn detect_resize_edge(win: &Window, pos: Point) -> Option<ResizeEdge> {
        let grip = Self::RESIZE_GRIP;
        let near_right = pos.x >= win.geometry.right() - grip;
        let near_bottom = pos.y >= win.geometry.bottom() - grip;

        match (near_right, near_bottom) {
            (true, true) => Some(ResizeEdge::BottomRight),
            (true, false) => Some(ResizeEdge::Right),
            (false, true) => Some(ResizeEdge::Bottom),
            _ => None,
        }
    }

    /// Gère l'appui du bouton gauche de la souris.
    /// Retourne true si un drag/resize a été initié.
    pub fn handle_mouse_down(&mut self, pos: Point) -> bool {
        // Cherche la fenêtre sous le curseur (du dessus vers le dessous)
        let active = self.active_workspace;
        let mut target = None;

        for (handle, win) in self.windows.iter() {
            if win.workspace_id == active && !win.is_minimized && win.geometry.contains(pos) {
                target = Some((handle, win.geometry, win.title.clone()));
            }
        }

        let (handle, geom, _title) = match target {
            Some(t) => t,
            None => return false,
        };

        // Focus la fenêtre cliquée
        self.set_focus(Some(handle));

        // Vérifie si on est sur une zone de redimensionnement
        if let Some(win) = self.windows.get(handle) {
            if let Some(edge) = Self::detect_resize_edge(win, pos) {
                self.drag_state = DragState::Resizing {
                    handle,
                    edge,
                    initial_rect: geom,
                    start_pos: pos,
                };
                return true;
            }
        }

        // Vérifie si on est dans la barre de titre => déplacement
        let titlebar = Rect::new(geom.x, geom.y, geom.width, Self::TITLEBAR_HEIGHT);
        if titlebar.contains(pos) {
            self.drag_state = DragState::Moving {
                handle,
                offset_x: pos.x - geom.x,
                offset_y: pos.y - geom.y,
            };
            return true;
        }

        false
    }

    /// Gère le mouvement de la souris pendant un drag en cours.
    pub fn handle_mouse_move(&mut self, pos: Point) {
        match self.drag_state.clone() {
            DragState::Moving { handle, offset_x, offset_y } => {
                if let Some(win) = self.windows.get_mut(handle) {
                    win.geometry.x = pos.x - offset_x;
                    win.geometry.y = pos.y - offset_y;
                    win.is_fullscreen = false;
                }
            }
            DragState::Resizing { handle, edge, initial_rect, start_pos } => {
                let dx = pos.x - start_pos.x;
                let dy = pos.y - start_pos.y;

                if let Some(win) = self.windows.get_mut(handle) {
                    match edge {
                        ResizeEdge::Right => {
                            let new_w = (initial_rect.width as i32 + dx).max(Self::MIN_WINDOW_SIZE as i32) as u32;
                            win.geometry.width = new_w;
                        }
                        ResizeEdge::Bottom => {
                            let new_h = (initial_rect.height as i32 + dy).max(Self::MIN_WINDOW_SIZE as i32) as u32;
                            win.geometry.height = new_h;
                        }
                        ResizeEdge::BottomRight => {
                            let new_w = (initial_rect.width as i32 + dx).max(Self::MIN_WINDOW_SIZE as i32) as u32;
                            let new_h = (initial_rect.height as i32 + dy).max(Self::MIN_WINDOW_SIZE as i32) as u32;
                            win.geometry.width = new_w;
                            win.geometry.height = new_h;
                        }
                    }
                }
            }
            DragState::None => {}
        }
    }

    /// Gère le relâchement du bouton de la souris.
    /// Applique le snap si la souris est dans une zone d'accrochage.
    pub fn handle_mouse_up(&mut self, pos: Point) {
        if let DragState::Moving { handle, .. } = self.drag_state {
            if let Some(zone) = self.detect_snap_zone(pos) {
                self.apply_snap(handle, zone);
            }
        }
        self.drag_state = DragState::None;
    }

    /// Vérifie si une interaction de drag est en cours.
    pub fn is_dragging(&self) -> bool {
        self.drag_state != DragState::None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drag_window_move() {
        let mut wm = WorkspaceManager::new(3);
        let handle = wm.create_window("Test", Rect::new(100, 100, 400, 300));

        // Clic dans la barre de titre
        assert!(wm.handle_mouse_down(Point::new(200, 110)));
        assert!(wm.is_dragging());

        // Déplacement
        wm.handle_mouse_move(Point::new(300, 200));
        let win = wm.windows.get(handle).unwrap();
        assert_eq!(win.geometry.x, 200); // 300 - (200-100) = 200
        assert_eq!(win.geometry.y, 190); // 200 - (110-100) = 190

        // Relâchement
        wm.handle_mouse_up(Point::new(300, 200));
        assert!(!wm.is_dragging());
    }

    #[test]
    fn test_snap_left() {
        let mut wm = WorkspaceManager::new(3);
        let handle = wm.create_window("Snap Test", Rect::new(100, 100, 400, 300));

        // Clic dans la barre de titre
        wm.handle_mouse_down(Point::new(200, 110));

        // Relâchement sur le bord gauche
        wm.handle_mouse_up(Point::new(5, 300));

        let win = wm.windows.get(handle).unwrap();
        assert_eq!(win.geometry.x, 0);
        assert_eq!(win.geometry.width, 960); // 1920 / 2
    }

    #[test]
    fn test_snap_maximize() {
        let mut wm = WorkspaceManager::new(3);
        let handle = wm.create_window("Max Test", Rect::new(100, 100, 400, 300));

        wm.handle_mouse_down(Point::new(200, 110));
        wm.handle_mouse_up(Point::new(500, 5));

        let win = wm.windows.get(handle).unwrap();
        assert!(win.is_fullscreen);
        assert_eq!(win.geometry.width, 1920);
    }

    #[test]
    fn test_resize_bottom_right() {
        let mut wm = WorkspaceManager::new(3);
        let handle = wm.create_window("Resize", Rect::new(100, 100, 400, 300));

        // Clic dans le coin bas-droit (499, 399 = right()-1, bottom()-1)
        assert!(wm.handle_mouse_down(Point::new(498, 398)));
        match &wm.drag_state {
            DragState::Resizing { edge, .. } => assert_eq!(*edge, ResizeEdge::BottomRight),
            _ => panic!("Expected Resizing state"),
        }

        // Agrandir de 50px dans chaque direction
        wm.handle_mouse_move(Point::new(548, 448));
        let win = wm.windows.get(handle).unwrap();
        assert_eq!(win.geometry.width, 450);
        assert_eq!(win.geometry.height, 350);

        wm.handle_mouse_up(Point::new(548, 448));
        assert!(!wm.is_dragging());
    }

    #[test]
    fn test_detect_snap_zones() {
        let wm = WorkspaceManager::new(3);
        assert_eq!(wm.detect_snap_zone(Point::new(5, 500)), Some(SnapZone::Left));
        assert_eq!(wm.detect_snap_zone(Point::new(1915, 500)), Some(SnapZone::Right));
        assert_eq!(wm.detect_snap_zone(Point::new(500, 5)), Some(SnapZone::Maximize));
        assert_eq!(wm.detect_snap_zone(Point::new(500, 500)), None);
    }
}
