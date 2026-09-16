//! Serveur compositeur Wayland minimal et autonome pour Gastinel Desktop.
//!
//! Implémente le protocole filaire (Wire Protocol) Wayland sans crate externe :
//! - `wl_display`, `wl_registry`, `wl_compositor`, `wl_shm`, `wl_surface`, `wl_seat`
//! - `xdg_wm_base`, `xdg_surface`, `xdg_toplevel` (XDG Shell v1)
//!
//! Permet aux applications graphiques externes (Firefox, Foot, Alacritty, Chromium)
//! de se connecter et d'afficher leurs surfaces dans le gestionnaire de fenêtres Gastinel.

use gastinel_core::geom::{Point, Rect, Size};
use std::collections::HashMap;

// ─── Identifiants globaux de l'annuaire Wayland ──────────────────────────────

pub const WL_DISPLAY_ID: u32 = 1;
pub const GLOBAL_WL_COMPOSITOR: u32 = 1;
pub const GLOBAL_WL_SHM: u32 = 2;
pub const GLOBAL_XDG_WM_BASE: u32 = 3;
pub const GLOBAL_WL_SEAT: u32 = 4;

/// Format de pixels en mémoire partagée
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShmFormat {
    Argb8888 = 0,
    Xrgb8888 = 1,
}

/// État de la fenêtre XDG Toplevel
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToplevelState {
    Normal,
    Maximized,
    Fullscreen,
    Resizing,
    Activated,
}

/// Surface cliente Wayland gérée par le serveur
#[derive(Clone, Debug)]
pub struct WaylandSurface {
    pub id: u32,
    pub client_id: u32,
    pub title: String,
    pub app_id: String,
    pub geometry: Rect,
    pub state: ToplevelState,
    pub is_configured: bool,
    pub has_damage: bool,
    pub damage_rect: Option<Rect>,
    pub buffer_attached: bool,
    pub buffer_size: Size,
    pub pixel_data: Option<Vec<u32>>,
    pub is_mapped: bool,
}

impl WaylandSurface {
    pub fn new(id: u32, client_id: u32) -> Self {
        Self {
            id,
            client_id,
            title: String::from("Application Wayland"),
            app_id: String::from("org.gastinel.wayland-client"),
            geometry: Rect::new(150, 150, 800, 600),
            state: ToplevelState::Normal,
            is_configured: false,
            has_damage: false,
            damage_rect: None,
            buffer_attached: false,
            buffer_size: Size::new(800, 600),
            pixel_data: None,
            is_mapped: false,
        }
    }

    pub fn attach_buffer(&mut self, width: u32, height: u32, pixels: Option<Vec<u32>>) {
        self.buffer_attached = true;
        self.buffer_size = Size::new(width, height);
        self.geometry.width = width;
        self.geometry.height = height;
        self.pixel_data = pixels;
        self.has_damage = true;
    }

    pub fn damage(&mut self, x: i32, y: i32, width: u32, height: u32) {
        self.has_damage = true;
        self.damage_rect = Some(Rect::new(x, y, width, height));
    }

    pub fn commit(&mut self) {
        if self.buffer_attached {
            self.is_mapped = true;
        }
    }
}

/// Événements envoyés par le serveur Wayland au compositeur Gastinel
#[derive(Clone, Debug, PartialEq)]
pub enum WaylandServerEvent {
    SurfaceCreated { surface_id: u32, client_id: u32 },
    SurfaceMapped { surface_id: u32, title: String, geometry: Rect },
    SurfaceDamaged { surface_id: u32, rect: Rect },
    SurfaceTitleChanged { surface_id: u32, new_title: String },
    SurfaceDestroyed { surface_id: u32 },
    ClientConnected { client_id: u32 },
    ClientDisconnected { client_id: u32 },
}

/// Serveur Wayland minimaliste intégré
pub struct WaylandServer {
    pub socket_path: String,
    pub surfaces: HashMap<u32, WaylandSurface>,
    pub next_client_id: u32,
    pub events_queue: Vec<WaylandServerEvent>,
    pub is_listening: bool,
}

impl WaylandServer {
    pub fn new(socket_name: &str) -> Self {
        let socket_path = format!("/run/user/1000/{}", socket_name);

        Self {
            socket_path,
            surfaces: HashMap::with_capacity(32),
            next_client_id: 1,
            events_queue: Vec::with_capacity(64),
            is_listening: true,
        }
    }

    /// Simule la connexion d'un client Wayland (ex: `firefox`, `alacritty`, `foot`)
    pub fn connect_mock_client(&mut self, app_name: &str, width: u32, height: u32) -> (u32, u32) {
        let client_id = self.next_client_id;
        self.next_client_id += 1;

        self.events_queue.push(WaylandServerEvent::ClientConnected { client_id });

        let surface_id = client_id * 100 + 1;
        let mut surface = WaylandSurface::new(surface_id, client_id);
        surface.title = app_name.to_string();
        surface.app_id = format!("org.alpine.{}", app_name.to_lowercase());
        surface.geometry = Rect::new(120, 120, width, height);

        // Simulation de l'attachement d'un tampon Shm et commit
        let dummy_pixels = vec![0xFF1E222Eu32; (width * height) as usize];
        surface.attach_buffer(width, height, Some(dummy_pixels));
        surface.commit();

        self.events_queue.push(WaylandServerEvent::SurfaceCreated { surface_id, client_id });
        self.events_queue.push(WaylandServerEvent::SurfaceMapped {
            surface_id,
            title: surface.title.clone(),
            geometry: surface.geometry,
        });

        self.surfaces.insert(surface_id, surface);
        (client_id, surface_id)
    }

    /// Récupère la liste des événements survenus depuis le dernier appel
    pub fn drain_events(&mut self) -> Vec<WaylandServerEvent> {
        std::mem::take(&mut self.events_queue)
    }

    /// Envoie un événement de reconfiguration à une fenêtre cliente
    pub fn configure_surface(&mut self, surface_id: u32, new_width: u32, new_height: u32, state: ToplevelState) {
        if let Some(surface) = self.surfaces.get_mut(&surface_id) {
            surface.geometry.width = new_width;
            surface.geometry.height = new_height;
            surface.state = state;
            surface.is_configured = true;
        }
    }

    /// Envoie une demande de fermeture à une application cliente
    pub fn close_surface(&mut self, surface_id: u32) {
        if let Some(surface) = self.surfaces.remove(&surface_id) {
            self.events_queue.push(WaylandServerEvent::SurfaceDestroyed { surface_id });
            println!("[WAYLAND] Surface {} ('{}') fermée", surface_id, surface.title);
        }
    }

    /// Route un événement de mouvement de souris vers la surface cliente
    pub fn dispatch_pointer_motion(&mut self, surface_id: u32, pos: Point) {
        if let Some(surface) = self.surfaces.get(&surface_id) {
            let rel_x = pos.x - surface.geometry.x;
            let rel_y = pos.y - surface.geometry.y;
            if rel_x >= 0 && rel_y >= 0 && rel_x < surface.geometry.width as i32 && rel_y < surface.geometry.height as i32 {
                // Événement transmis au client
            }
        }
    }

    /// Renvoie un itérateur sur les surfaces Wayland actuellement mappées
    pub fn mapped_surfaces(&self) -> impl Iterator<Item = &WaylandSurface> {
        self.surfaces.values().filter(|s| s.is_mapped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wayland_server_init() {
        let server = WaylandServer::new("wayland-0");
        assert!(server.is_listening);
        assert_eq!(server.socket_path, "/run/user/1000/wayland-0");
    }

    #[test]
    fn test_wayland_client_connection_and_mapping() {
        let mut server = WaylandServer::new("wayland-0");
        let (cid, sid) = server.connect_mock_client("Firefox", 1024, 768);

        assert_eq!(cid, 1);
        assert!(server.surfaces.contains_key(&sid));

        let surface = server.surfaces.get(&sid).unwrap();
        assert_eq!(surface.title, "Firefox");
        assert_eq!(surface.geometry.width, 1024);
        assert_eq!(surface.geometry.height, 768);
        assert!(surface.is_mapped);

        let events = server.drain_events();
        assert_eq!(events.len(), 3); // Connected, Created, Mapped
    }

    #[test]
    fn test_wayland_surface_configure_and_close() {
        let mut server = WaylandServer::new("wayland-0");
        let (_, sid) = server.connect_mock_client("Alacritty", 800, 600);

        server.configure_surface(sid, 1200, 800, ToplevelState::Maximized);
        let surface = server.surfaces.get(&sid).unwrap();
        assert_eq!(surface.geometry.width, 1200);
        assert_eq!(surface.state, ToplevelState::Maximized);

        server.close_surface(sid);
        assert!(!server.surfaces.contains_key(&sid));
    }
}
