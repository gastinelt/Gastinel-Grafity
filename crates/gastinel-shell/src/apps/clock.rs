//! Application Horloge, Alarmes, Chronomètre et Minuteur pour Gastinel Desktop.
//!
//! Affiche l'heure avec une précision à la milliseconde (`.mmm`) et offre
//! un chronomètre précis avec enregistrement des tours, alarmes et minuteurs.

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect};
use gastinel_core::render::{QuadInstance, RenderQueue};
use std::time::Instant;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ClockTab {
    Horloge,
    Alarmes,
    Chronometre,
    Minuteur,
}

#[derive(Clone, Debug)]
pub struct Alarm {
    pub hour: u8,
    pub minute: u8,
    pub label: String,
    pub enabled: bool,
    pub repeat_weekdays: bool,
}

#[derive(Clone, Debug)]
pub struct Stopwatch {
    pub is_running: bool,
    pub start_instant: Option<Instant>,
    pub accumulated_ms: u64,
    pub laps: Vec<u64>,
}

impl Stopwatch {
    pub fn new() -> Self {
        Self {
            is_running: false,
            start_instant: None,
            accumulated_ms: 0,
            laps: Vec::with_capacity(16),
        }
    }

    pub fn start(&mut self) {
        if !self.is_running {
            self.start_instant = Some(Instant::now());
            self.is_running = true;
        }
    }

    pub fn stop(&mut self) {
        if self.is_running {
            if let Some(st) = self.start_instant.take() {
                self.accumulated_ms += st.elapsed().as_millis() as u64;
            }
            self.is_running = false;
        }
    }

    pub fn lap(&mut self) {
        let current = self.elapsed_ms();
        self.laps.push(current);
    }

    pub fn reset(&mut self) {
        self.is_running = false;
        self.start_instant = None;
        self.accumulated_ms = 0;
        self.laps.clear();
    }

    pub fn elapsed_ms(&self) -> u64 {
        let mut total = self.accumulated_ms;
        if self.is_running {
            if let Some(st) = self.start_instant {
                total += st.elapsed().as_millis() as u64;
            }
        }
        total
    }
}

pub struct CountdownTimer {
    pub total_duration_ms: u64,
    pub remaining_ms: u64,
    pub is_running: bool,
    pub label: String,
}

pub struct ClockApp {
    pub bounds: Rect,
    pub active_tab: ClockTab,
    pub current_time_str: String,
    pub current_ms: u16,
    pub stopwatch: Stopwatch,
    pub alarms: Vec<Alarm>,
    pub timer: CountdownTimer,
}

impl ClockApp {
    pub fn new(bounds: Rect) -> Self {
        let mut alarms = Vec::with_capacity(4);
        alarms.push(Alarm {
            hour: 7,
            minute: 0,
            label: String::from("Réveil"),
            enabled: true,
            repeat_weekdays: true,
        });
        alarms.push(Alarm {
            hour: 22,
            minute: 30,
            label: String::from("Session IA Gemma"),
            enabled: false,
            repeat_weekdays: false,
        });

        Self {
            bounds,
            active_tab: ClockTab::Horloge,
            current_time_str: String::from("22:45:00"),
            current_ms: 248,
            stopwatch: Stopwatch::new(),
            alarms,
            timer: CountdownTimer {
                total_duration_ms: 5 * 60 * 1000,
                remaining_ms: 5 * 60 * 1000,
                is_running: false,
                label: String::from("Pause café"),
            },
        }
    }

    pub fn switch_tab(&mut self, tab: ClockTab) {
        self.active_tab = tab;
    }

    pub fn update_time(&mut self, time_str: &str, ms: u16) {
        self.current_time_str = time_str.to_string();
        self.current_ms = ms;
    }

    pub fn toggle_alarm(&mut self, index: usize) {
        if let Some(al) = self.alarms.get_mut(index) {
            al.enabled = !al.enabled;
        }
    }

    pub fn add_alarm(&mut self, hour: u8, minute: u8, label: &str) {
        self.alarms.push(Alarm {
            hour,
            minute,
            label: label.to_string(),
            enabled: true,
            repeat_weekdays: false,
        });
    }

    pub fn format_stopwatch(&self) -> String {
        let total_ms = self.stopwatch.elapsed_ms();
        let ms = total_ms % 1000;
        let total_secs = total_ms / 1000;
        let secs = total_secs % 60;
        let mins = (total_secs / 60) % 60;
        let hours = total_secs / 3600;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}.{:03}", hours, mins, secs, ms)
        } else {
            format!("{:02}:{:02}.{:03}", mins, secs, ms)
        }
    }

    pub fn render(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        // Fond de fenêtre
        queue.push(
            QuadInstance::rounded_box(
                self.bounds,
                14.0,
                [0.10, 0.11, 0.15, 0.98],
                [0.22, 0.25, 0.38, 0.7],
                1.5,
                0.2,
            )
            .with_shadow([0.0, 0.0, 0.0, 0.40], 24.0, [0.0, 6.0]),
        );

        // Barre d'onglets (4 onglets)
        let tab_bar_rect = Rect::new(self.bounds.x + 12, self.bounds.y + 12, self.bounds.width - 24, 38);
        queue.push(QuadInstance::rounded_box(
            tab_bar_rect,
            8.0,
            [0.14, 0.15, 0.20, 0.9],
            [0.22, 0.24, 0.32, 0.5],
            1.0,
            0.21,
        ));

        let tabs = [
            (ClockTab::Horloge, "🕐 Horloge"),
            (ClockTab::Alarmes, "⏰ Alarmes"),
            (ClockTab::Chronometre, "⏱ Chrono"),
            (ClockTab::Minuteur, "⏳ Minuteur"),
        ];

        let tab_w = (tab_bar_rect.width - 12) / 4;
        for (i, (tab_kind, title)) in tabs.iter().enumerate() {
            let tx = tab_bar_rect.x + 4 + (i as i32 * tab_w as i32);
            let t_rect = Rect::new(tx, tab_bar_rect.y + 3, tab_w, 32);

            let is_sel = *tab_kind == self.active_tab;
            if is_sel {
                queue.push(QuadInstance::rounded_box(
                    t_rect,
                    6.0,
                    [0.32, 0.38, 0.85, 0.9],
                    [0.45, 0.52, 0.98, 0.9],
                    1.0,
                    0.22,
                ));
            }

            atlas.layout_text(
                title,
                Point::new(tx + 8, t_rect.y + 21),
                11,
                if is_sel { FontWeight::SemiBold } else { FontWeight::Regular },
                if is_sel { [1.0, 1.0, 1.0, 1.0] } else { [0.65, 0.68, 0.78, 1.0] },
                glyphs,
            );
        }

        let content_y = tab_bar_rect.bottom() + 14;

        match self.active_tab {
            ClockTab::Horloge => {
                // Cadre grand affichage heure
                let clock_box = Rect::new(self.bounds.x + 16, content_y, self.bounds.width - 32, 130);
                queue.push(QuadInstance::rounded_box(
                    clock_box,
                    10.0,
                    [0.08, 0.09, 0.12, 0.95],
                    [0.25, 0.28, 0.40, 0.6],
                    1.0,
                    0.21,
                ));

                let time_full = format!("{}.{:03}", self.current_time_str, self.current_ms);
                atlas.layout_text(
                    &time_full,
                    Point::new(clock_box.x + 24, clock_box.y + 70),
                    36,
                    FontWeight::Bold,
                    [0.92, 0.95, 1.0, 1.0],
                    glyphs,
                );

                atlas.layout_text(
                    "Précision: ±1 milliseconde • Fuseau: Europe/Paris (CET)",
                    Point::new(clock_box.x + 24, clock_box.y + 104),
                    12,
                    FontWeight::Regular,
                    [0.45, 0.78, 0.55, 1.0], // vert émeraude
                    glyphs,
                );
            }
            ClockTab::Chronometre => {
                let chrono_box = Rect::new(self.bounds.x + 16, content_y, self.bounds.width - 32, 110);
                queue.push(QuadInstance::rounded_box(
                    chrono_box,
                    10.0,
                    [0.08, 0.09, 0.12, 0.95],
                    [0.35, 0.28, 0.55, 0.6],
                    1.0,
                    0.21,
                ));

                let chrono_text = self.format_stopwatch();
                atlas.layout_text(
                    &chrono_text,
                    Point::new(chrono_box.x + 24, chrono_box.y + 65),
                    32,
                    FontWeight::Bold,
                    [0.85, 0.82, 1.0, 1.0],
                    glyphs,
                );

                // Boutons d'action sous le chrono
                let btn_y = chrono_box.bottom() + 14;
                let (btn_lbl, btn_col) = if self.stopwatch.is_running {
                    ("Pause", [0.85, 0.30, 0.35, 0.9])
                } else {
                    ("Démarrer", [0.25, 0.65, 0.45, 0.9])
                };

                let start_btn = Rect::new(self.bounds.x + 16, btn_y, 110, 36);
                queue.push(QuadInstance::rounded_box(start_btn, 6.0, btn_col, [1.0, 1.0, 1.0, 0.3], 1.0, 0.22));
                atlas.layout_text(btn_lbl, Point::new(start_btn.x + 20, start_btn.y + 24), 13, FontWeight::SemiBold, [1.0, 1.0, 1.0, 1.0], glyphs);

                let lap_btn = Rect::new(start_btn.right() + 10, btn_y, 100, 36);
                queue.push(QuadInstance::rounded_box(lap_btn, 6.0, [0.20, 0.24, 0.35, 0.9], [0.35, 0.42, 0.60, 0.5], 1.0, 0.22));
                atlas.layout_text("Tour", Point::new(lap_btn.x + 30, lap_btn.y + 24), 13, FontWeight::Regular, [0.9, 0.9, 0.95, 1.0], glyphs);
            }
            ClockTab::Alarmes => {
                let mut ay = content_y;
                for (idx, al) in self.alarms.iter().enumerate() {
                    let card = Rect::new(self.bounds.x + 16, ay, self.bounds.width - 32, 52);
                    let (border_col, text_col) = if al.enabled {
                        ([0.35, 0.55, 0.95, 0.8], [0.95, 0.96, 1.0, 1.0])
                    } else {
                        ([0.20, 0.22, 0.28, 0.4], [0.55, 0.58, 0.65, 1.0])
                    };

                    queue.push(QuadInstance::rounded_box(card, 8.0, [0.12, 0.13, 0.18, 0.9], border_col, 1.0, 0.21));

                    let time_str = format!("{:02}:{:02}", al.hour, al.minute);
                    atlas.layout_text(&time_str, Point::new(card.x + 14, card.y + 34), 20, FontWeight::Bold, text_col, glyphs);
                    atlas.layout_text(&al.label, Point::new(card.x + 100, card.y + 32), 13, FontWeight::Regular, text_col, glyphs);

                    // Toggle pill
                    let switch_rect = Rect::new(card.right() - 54, card.y + 14, 40, 22);
                    let switch_col = if al.enabled { [0.25, 0.70, 0.45, 0.9] } else { [0.30, 0.32, 0.38, 0.7] };
                    queue.push(QuadInstance::rounded_box(switch_rect, 11.0, switch_col, [1.0, 1.0, 1.0, 0.2], 1.0, 0.22));

                    ay += 60;
                    if idx >= 3 { break; }
                }
            }
            ClockTab::Minuteur => {
                let timer_box = Rect::new(self.bounds.x + 16, content_y, self.bounds.width - 32, 120);
                queue.push(QuadInstance::rounded_box(
                    timer_box,
                    10.0,
                    [0.08, 0.09, 0.12, 0.95],
                    [0.45, 0.35, 0.75, 0.6],
                    1.0,
                    0.21,
                ));

                let mins = self.timer.remaining_ms / 60000;
                let secs = (self.timer.remaining_ms % 60000) / 1000;
                let t_text = format!("{:02}:{:02}", mins, secs);

                atlas.layout_text(&t_text, Point::new(timer_box.x + 24, timer_box.y + 68), 34, FontWeight::Bold, [0.95, 0.92, 1.0, 1.0], glyphs);
                atlas.layout_text(&self.timer.label, Point::new(timer_box.x + 24, timer_box.y + 98), 13, FontWeight::Regular, [0.65, 0.68, 0.80, 1.0], glyphs);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stopwatch_flow() {
        let mut sw = Stopwatch::new();
        assert_eq!(sw.elapsed_ms(), 0);
        assert!(!sw.is_running);

        sw.start();
        assert!(sw.is_running);
        sw.lap();
        assert_eq!(sw.laps.len(), 1);

        sw.stop();
        assert!(!sw.is_running);

        sw.reset();
        assert_eq!(sw.elapsed_ms(), 0);
        assert_eq!(sw.laps.len(), 0);
    }

    #[test]
    fn test_clock_app_alarms() {
        let mut app = ClockApp::new(Rect::new(0, 0, 420, 360));
        assert_eq!(app.alarms.len(), 2);
        assert!(app.alarms[0].enabled);

        app.toggle_alarm(0);
        assert!(!app.alarms[0].enabled);

        app.add_alarm(14, 15, "Réunion");
        assert_eq!(app.alarms.len(), 3);
        assert_eq!(app.alarms[2].hour, 14);
    }
}
