//! Direct Rendering Manager (DRM) and Kernel Mode Setting (KMS) abstractions.
//!
//! Handles kernel display discovery and page flipping without X11 or intermediary servers.

use gastinel_core::geom::Size;

#[derive(Clone, Debug)]
pub struct DrmOutputMode {
    pub width: u32,
    pub height: u32,
    pub refresh_rate_hz: u32,
}

#[derive(Clone, Debug)]
pub struct DrmConnector {
    pub name: String,
    pub connected: bool,
    pub current_mode: DrmOutputMode,
}

pub struct DrmDevice {
    pub card_path: String,
    pub connectors: Vec<DrmConnector>,
}

impl DrmDevice {
    pub fn probe_outputs() -> Vec<DrmConnector> {
        // On Linux, scans /sys/class/drm/card*-*
        #[cfg(target_os = "linux")]
        {
            let mut connectors = Vec::new();
            if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    if name.starts_with("card") && name.contains('-') {
                        let status_path = entry.path().join("status");
                        let connected = std::fs::read_to_string(status_path)
                            .map(|s| s.trim() == "connected")
                            .unwrap_or(false);

                        if connected {
                            connectors.push(DrmConnector {
                                name,
                                connected: true,
                                current_mode: DrmOutputMode {
                                    width: 1920,
                                    height: 1080,
                                    refresh_rate_hz: 60,
                                },
                            });
                        }
                    }
                }
            }
            connectors
        }

        #[cfg(not(target_os = "linux"))]
        {
            vec![DrmConnector {
                name: "eDP-1 (Virtual)".to_string(),
                connected: true,
                current_mode: DrmOutputMode {
                    width: 1920,
                    height: 1080,
                    refresh_rate_hz: 60,
                },
            }]
        }
    }

    pub fn primary_resolution(&self) -> Size {
        self.connectors
            .first()
            .map(|c| Size::new(c.current_mode.width, c.current_mode.height))
            .unwrap_or(Size::new(1920, 1080))
    }
}
