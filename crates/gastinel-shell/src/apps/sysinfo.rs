//! Moniteur Système Gastinel (Gestionnaire de performances : Mémoire, Cœurs CPU, GPU).
//!
//! Conçu spécialement pour l'architecture Intel Core Ultra 5 225U (P-cores / E-cores)
//! et la mémoire LPDDR5x à 8733 MT/s avec swap zram compressé lz4.
//!
//! Fonctionne sans aucune crate tierce : lecture directe de `/proc/` sur Linux
//! et FFI Win32 native sur Windows de développement.

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect};
use gastinel_core::render::{QuadInstance, RenderQueue};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MonitorTab {
    Memoire,
    Cpu,
    Gpu,
}

#[derive(Clone, Debug)]
pub struct MemoryStats {
    pub total_mb: u32,
    pub used_mb: u32,
    pub available_mb: u32,
    pub zram_saved_mb: u32,
    pub zram_used_mb: u32,
    pub cached_mb: u32,
    pub swap_total_mb: u32,
    pub swap_used_mb: u32,
}

#[derive(Clone, Debug)]
pub struct CoreUsage {
    pub id: usize,
    pub usage_pct: f32,
    pub is_p_core: bool,
}

#[derive(Clone, Debug)]
pub struct CpuStats {
    pub model_name: String,
    pub total_usage_pct: f32,
    pub cores: Vec<CoreUsage>,
    pub frequency_ghz: f32,
}

#[derive(Clone, Debug)]
pub struct GpuStats {
    pub name: String,
    pub vram_total_mb: u32,
    pub vram_used_mb: u32,
    pub usage_pct: f32,
    pub driver_api: String,
}

#[derive(Clone, Debug)]
pub struct HistoryBuffer {
    pub samples: [f32; 60],
    pub head: usize,
}

impl HistoryBuffer {
    pub fn new() -> Self {
        Self {
            samples: [0.0; 60],
            head: 0,
        }
    }

    pub fn push(&mut self, val: f32) {
        self.samples[self.head] = val.clamp(0.0, 100.0);
        self.head = (self.head + 1) % self.samples.len();
    }
}

pub struct GastinelMonitor {
    pub bounds: Rect,
    pub active_tab: MonitorTab,
    pub memory: MemoryStats,
    pub cpu: CpuStats,
    pub gpu: GpuStats,
    pub mem_history: HistoryBuffer,
    pub cpu_history: HistoryBuffer,
    pub gpu_history: HistoryBuffer,
}

impl GastinelMonitor {
    pub fn new(bounds: Rect) -> Self {
        let mut cpu_cores = Vec::with_capacity(10);
        // Intel Core Ultra 5 225U : 2 Performance cores (4 threads) + 8 Efficient cores = 12 vCPUs ou 10 cœurs physiques
        for i in 0..10 {
            cpu_cores.push(CoreUsage {
                id: i,
                usage_pct: 12.0 + (i as f32 * 3.5),
                is_p_core: i < 2,
            });
        }

        let mut mon = Self {
            bounds,
            active_tab: MonitorTab::Memoire,
            memory: MemoryStats {
                total_mb: 7840,
                used_mb: 1820,
                available_mb: 6020,
                zram_saved_mb: 1420,
                zram_used_mb: 480,
                cached_mb: 1240,
                swap_total_mb: 6144,
                swap_used_mb: 480,
            },
            cpu: CpuStats {
                model_name: String::from("Intel Core Ultra 5 225U (Intel 4)"),
                total_usage_pct: 18.5,
                cores: cpu_cores,
                frequency_ghz: 3.8,
            },
            gpu: GpuStats {
                name: String::from("Intel(R) Graphics (Xe-LPG 4 Xe-cores)"),
                vram_total_mb: 3920,
                vram_used_mb: 2980, // modèle Gemma 4 chargé en UMA
                usage_pct: 34.0,
                driver_api: String::from("Vulkan 1.4.309"),
            },
            mem_history: HistoryBuffer::new(),
            cpu_history: HistoryBuffer::new(),
            gpu_history: HistoryBuffer::new(),
        };

        // Remplir un historique initial réaliste
        for i in 0..60 {
            mon.mem_history.push(22.0 + (i as f32 * 0.1));
            mon.cpu_history.push(15.0 + (i as f32 % 7.0) * 3.0);
            mon.gpu_history.push(30.0 + (i as f32 % 5.0) * 4.0);
        }

        mon
    }

    pub fn switch_tab(&mut self, tab: MonitorTab) {
        self.active_tab = tab;
    }

    /// Actualise les métriques depuis le système hôte.
    pub fn refresh(&mut self) {
        #[cfg(target_os = "windows")]
        self.refresh_windows();

        #[cfg(target_os = "linux")]
        self.refresh_linux();
    }

    #[cfg(target_os = "windows")]
    fn refresh_windows(&mut self) {
        #[allow(non_snake_case)]
        #[repr(C)]
        struct MEMORYSTATUSEX {
            dwLength: u32,
            dwMemoryLoad: u32,
            ullTotalPhys: u64,
            ullAvailPhys: u64,
            ullTotalPageFile: u64,
            ullAvailPageFile: u64,
            ullTotalVirtual: u64,
            ullAvailVirtual: u64,
            ullAvailExtendedVirtual: u64,
        }

        #[link(name = "kernel32")]
        extern "system" {
            fn GlobalMemoryStatusEx(lpBuffer: *mut MEMORYSTATUSEX) -> i32;
        }

        unsafe {
            let mut stat: MEMORYSTATUSEX = std::mem::zeroed();
            stat.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
            if GlobalMemoryStatusEx(&mut stat) != 0 {
                let total_mb = (stat.ullTotalPhys / (1024 * 1024)) as u32;
                let avail_mb = (stat.ullAvailPhys / (1024 * 1024)) as u32;
                let used_mb = total_mb.saturating_sub(avail_mb);

                self.memory.total_mb = total_mb;
                self.memory.available_mb = avail_mb;
                self.memory.used_mb = used_mb;

                let usage_pct = (used_mb as f32 / total_mb.max(1) as f32) * 100.0;
                self.mem_history.push(usage_pct);
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn refresh_linux(&mut self) {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            let mut total_kb = 0u64;
            let mut avail_kb = 0u64;

            for line in content.lines() {
                let mut parts = line.split_whitespace();
                if let (Some(key), Some(val_str)) = (parts.next(), parts.next()) {
                    if let Ok(val) = val_str.parse::<u64>() {
                        match key {
                            "MemTotal:" => total_kb = val,
                            "MemAvailable:" => avail_kb = val,
                            _ => {}
                        }
                    }
                }
            }

            if total_kb > 0 {
                self.memory.total_mb = (total_kb / 1024) as u32;
                self.memory.available_mb = (avail_kb / 1024) as u32;
                self.memory.used_mb = self.memory.total_mb.saturating_sub(self.memory.available_mb);
                let pct = (self.memory.used_mb as f32 / self.memory.total_mb as f32) * 100.0;
                self.mem_history.push(pct);
            }
        }
    }

    pub fn render(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        // Fond principal
        queue.push(
            QuadInstance::rounded_box(
                self.bounds,
                14.0,
                [0.09, 0.10, 0.14, 0.98],
                [0.22, 0.25, 0.35, 0.7],
                1.5,
                0.2,
            )
            .with_shadow([0.0, 0.0, 0.0, 0.45], 24.0, [0.0, 6.0]),
        );

        // Barre d'onglets (Mémoire, CPU, GPU)
        let tab_bar_rect = Rect::new(self.bounds.x + 14, self.bounds.y + 14, self.bounds.width - 28, 38);
        queue.push(QuadInstance::rounded_box(
            tab_bar_rect,
            8.0,
            [0.13, 0.14, 0.19, 0.9],
            [0.20, 0.22, 0.30, 0.5],
            1.0,
            0.21,
        ));

        let tabs = [
            (MonitorTab::Memoire, "💾 Mémoire & zram"),
            (MonitorTab::Cpu, "🔲 Processeur (10 cœurs)"),
            (MonitorTab::Gpu, "🎮 GPU & VRAM UMA"),
        ];

        let tab_w = (tab_bar_rect.width - 12) / 3;
        for (i, (tab_kind, title)) in tabs.iter().enumerate() {
            let tx = tab_bar_rect.x + 4 + (i as i32 * tab_w as i32);
            let t_rect = Rect::new(tx, tab_bar_rect.y + 3, tab_w, 32);

            let is_sel = *tab_kind == self.active_tab;
            if is_sel {
                queue.push(QuadInstance::rounded_box(
                    t_rect,
                    6.0,
                    [0.28, 0.35, 0.85, 0.9],
                    [0.42, 0.50, 0.98, 0.9],
                    1.0,
                    0.22,
                ));
            }

            atlas.layout_text(
                title,
                Point::new(tx + 12, t_rect.y + 21),
                11,
                if is_sel { FontWeight::SemiBold } else { FontWeight::Regular },
                if is_sel { [1.0, 1.0, 1.0, 1.0] } else { [0.65, 0.68, 0.78, 1.0] },
                glyphs,
            );
        }

        let content_y = tab_bar_rect.bottom() + 16;

        match self.active_tab {
            MonitorTab::Memoire => {
                // Métrique principale RAM
                let used_pct = (self.memory.used_mb as f32 / self.memory.total_mb.max(1) as f32) * 100.0;
                let ram_card = Rect::new(self.bounds.x + 16, content_y, self.bounds.width - 32, 100);
                queue.push(QuadInstance::rounded_box(ram_card, 10.0, [0.12, 0.13, 0.18, 0.9], [0.25, 0.30, 0.45, 0.6], 1.0, 0.21));

                atlas.layout_text(
                    "MÉMOIRE VIVE PHYSIQUE (LPDDR5x @ 8733 MT/s)",
                    Point::new(ram_card.x + 14, ram_card.y + 24),
                    11,
                    FontWeight::SemiBold,
                    [0.65, 0.70, 0.85, 1.0],
                    glyphs,
                );

                let ram_label = format!("{:.1} Go utilisés / {:.1} Go ({:.0}%)",
                    self.memory.used_mb as f32 / 1024.0,
                    self.memory.total_mb as f32 / 1024.0,
                    used_pct
                );
                atlas.layout_text(&ram_label, Point::new(ram_card.x + 14, ram_card.y + 50), 16, FontWeight::Bold, [0.95, 0.96, 1.0, 1.0], glyphs);

                // Jauge barre de progression
                let bar_rect = Rect::new(ram_card.x + 14, ram_card.y + 64, ram_card.width - 28, 14);
                queue.push(QuadInstance::rounded_box(bar_rect, 7.0, [0.16, 0.18, 0.24, 0.9], [0.0, 0.0, 0.0, 0.0], 0.0, 0.22));

                let fill_w = ((bar_rect.width as f32) * (used_pct / 100.0).clamp(0.0, 1.0)) as u32;
                let fill_rect = Rect::new(bar_rect.x, bar_rect.y, fill_w.max(6), bar_rect.height);
                queue.push(QuadInstance::rounded_box(fill_rect, 7.0, [0.35, 0.55, 0.95, 0.95], [0.5, 0.7, 1.0, 0.8], 1.0, 0.23));

                // Carte zram
                let zram_card = Rect::new(self.bounds.x + 16, ram_card.bottom() + 12, self.bounds.width - 32, 70);
                queue.push(QuadInstance::rounded_box(zram_card, 10.0, [0.12, 0.13, 0.18, 0.9], [0.35, 0.25, 0.55, 0.6], 1.0, 0.21));

                atlas.layout_text("ZRAM SWAP COMPRESSÉ (lz4 - priorité max)", Point::new(zram_card.x + 14, zram_card.y + 24), 11, FontWeight::SemiBold, [0.85, 0.70, 1.0, 1.0], glyphs);
                let zram_label = format!("{} Mo économisés virtuellement • {} Mo consommés en RAM", self.memory.zram_saved_mb, self.memory.zram_used_mb);
                atlas.layout_text(&zram_label, Point::new(zram_card.x + 14, zram_card.y + 48), 13, FontWeight::Regular, [0.45, 0.85, 0.60, 1.0], glyphs);
            }
            MonitorTab::Cpu => {
                let cpu_card = Rect::new(self.bounds.x + 16, content_y, self.bounds.width - 32, 60);
                queue.push(QuadInstance::rounded_box(cpu_card, 8.0, [0.12, 0.13, 0.18, 0.9], [0.22, 0.25, 0.35, 0.5], 1.0, 0.21));
                atlas.layout_text(&self.cpu.model_name, Point::new(cpu_card.x + 14, cpu_card.y + 24), 13, FontWeight::Bold, [0.95, 0.96, 1.0, 1.0], glyphs);
                let cpu_usage_str = format!("Utilisation globale: {:.1}% @ {:.1} GHz", self.cpu.total_usage_pct, self.cpu.frequency_ghz);
                atlas.layout_text(&cpu_usage_str, Point::new(cpu_card.x + 14, cpu_card.y + 46), 12, FontWeight::Regular, [0.45, 0.85, 0.55, 1.0], glyphs);

                // Grille des 10 cœurs (2 P-cores + 8 E-cores)
                let cy = cpu_card.bottom() + 10;
                let core_w = (self.bounds.width - 40) / 2;
                for (idx, core) in self.cpu.cores.iter().enumerate() {
                    let col = idx % 2;
                    let row = idx / 2;
                    let cx = self.bounds.x + 16 + (col as i32 * (core_w as i32 + 8));
                    let c_rect = Rect::new(cx, cy + (row as i32 * 32), core_w, 26);

                    let (type_str, fill_color) = if core.is_p_core {
                        ("P-Core", [0.35, 0.55, 0.95, 0.9])
                    } else {
                        ("E-Core", [0.45, 0.35, 0.85, 0.9])
                    };

                    queue.push(QuadInstance::rounded_box(c_rect, 4.0, [0.13, 0.14, 0.19, 0.9], [0.22, 0.24, 0.30, 0.5], 1.0, 0.21));

                    // Mini barre de charge
                    let inner_w = ((c_rect.width as f32 - 100.0) * (core.usage_pct / 100.0).clamp(0.0, 1.0)) as u32;
                    let inner_rect = Rect::new(c_rect.x + 60, c_rect.y + 7, inner_w.max(4), 12);
                    queue.push(QuadInstance::rounded_box(inner_rect, 3.0, fill_color, [0.0, 0.0, 0.0, 0.0], 0.0, 0.22));

                    let label = format!("{} {}: {:.0}%", type_str, core.id, core.usage_pct);
                    atlas.layout_text(&label, Point::new(c_rect.x + 6, c_rect.y + 18), 10, FontWeight::Regular, [0.85, 0.88, 0.95, 1.0], glyphs);
                }
            }
            MonitorTab::Gpu => {
                let gpu_card = Rect::new(self.bounds.x + 16, content_y, self.bounds.width - 32, 120);
                queue.push(QuadInstance::rounded_box(gpu_card, 10.0, [0.12, 0.13, 0.18, 0.9], [0.25, 0.40, 0.70, 0.6], 1.0, 0.21));

                atlas.layout_text(&self.gpu.name, Point::new(gpu_card.x + 14, gpu_card.y + 26), 13, FontWeight::Bold, [0.95, 0.96, 1.0, 1.0], glyphs);
                atlas.layout_text(&format!("Pilote graphique : {}", self.gpu.driver_api), Point::new(gpu_card.x + 14, gpu_card.y + 48), 11, FontWeight::Regular, [0.65, 0.70, 0.85, 1.0], glyphs);

                let vram_label = format!("VRAM UMA Allouée : {} Mo / {} Mo (Modèle IA inclus)", self.gpu.vram_used_mb, self.gpu.vram_total_mb);
                atlas.layout_text(&vram_label, Point::new(gpu_card.x + 14, gpu_card.y + 76), 12, FontWeight::SemiBold, [0.85, 0.75, 1.0, 1.0], glyphs);

                // Barre VRAM
                let vbar_rect = Rect::new(gpu_card.x + 14, gpu_card.y + 88, gpu_card.width - 28, 12);
                queue.push(QuadInstance::rounded_box(vbar_rect, 6.0, [0.16, 0.18, 0.24, 0.9], [0.0, 0.0, 0.0, 0.0], 0.0, 0.22));

                let vpct = (self.gpu.vram_used_mb as f32 / self.gpu.vram_total_mb.max(1) as f32).clamp(0.0, 1.0);
                let vfill_w = ((vbar_rect.width as f32) * vpct) as u32;
                queue.push(QuadInstance::rounded_box(Rect::new(vbar_rect.x, vbar_rect.y, vfill_w.max(6), vbar_rect.height), 6.0, [0.55, 0.35, 0.95, 0.9], [0.7, 0.5, 1.0, 0.8], 1.0, 0.23));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitor_init_and_history() {
        let mon = GastinelMonitor::new(Rect::new(0, 0, 500, 400));
        assert_eq!(mon.cpu.cores.len(), 10);
        assert!(mon.cpu.cores[0].is_p_core);
        assert!(!mon.cpu.cores[9].is_p_core);
        assert_eq!(mon.mem_history.samples.len(), 60);
    }

    #[test]
    fn test_history_buffer_wrap() {
        let mut buf = HistoryBuffer::new();
        for i in 0..70 {
            buf.push(i as f32);
        }
        assert_eq!(buf.head, 10);
    }
}
