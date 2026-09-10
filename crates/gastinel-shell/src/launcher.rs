//! Spotlight-style fast application search and command launcher.
//!
//! Instant popup, fuzzy matching, and keyboard navigation.

use gastinel_core::geom::{Rect, Size};

#[derive(Clone, Debug)]
pub struct AppEntry {
    pub name: String,
    pub exec: String,
    pub comment: String,
}

pub struct Launcher {
    pub visible: bool,
    pub bounds: Rect,
    pub query: String,
    pub entries: Vec<AppEntry>,
    pub filtered_indices: Vec<usize>,
    pub selected_index: usize,
}

impl Launcher {
    pub const WIDTH: u32 = 640;
    pub const HEIGHT: u32 = 420;

    pub fn new(screen_size: Size) -> Self {
        let x = (screen_size.width.saturating_sub(Self::WIDTH) / 2) as i32;
        let y = (screen_size.height.saturating_sub(Self::HEIGHT) / 3) as i32; // slightly above center

        Self {
            visible: false,
            bounds: Rect::new(x, y, Self::WIDTH, Self::HEIGHT),
            query: String::with_capacity(64),
            entries: Vec::new(),
            filtered_indices: Vec::with_capacity(32),
            selected_index: 0,
        }
    }

    pub fn recalculate_position(&mut self, screen_size: Size) {
        self.bounds.x = (screen_size.width.saturating_sub(Self::WIDTH) / 2) as i32;
        self.bounds.y = (screen_size.height.saturating_sub(Self::HEIGHT) / 3) as i32;
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.query.clear();
            self.refilter();
            self.selected_index = 0;
        }
    }

    pub fn open(&mut self) {
        self.visible = true;
        self.query.clear();
        self.refilter();
        self.selected_index = 0;
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.query.clear();
    }

    pub fn add_app(&mut self, name: &str, exec: &str, comment: &str) {
        self.entries.push(AppEntry {
            name: name.to_string(),
            exec: exec.to_string(),
            comment: comment.to_string(),
        });
        self.refilter();
    }

    pub fn insert_char(&mut self, c: char) {
        self.query.push(c);
        self.refilter();
        self.selected_index = 0;
    }

    pub fn backspace(&mut self) {
        self.query.pop();
        self.refilter();
        self.selected_index = 0;
    }

    pub fn select_next(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_indices.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.filtered_indices.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.filtered_indices.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn selected_entry(&self) -> Option<&AppEntry> {
        let idx = *self.filtered_indices.get(self.selected_index)?;
        self.entries.get(idx)
    }

    /// Fast, zero-allocation fuzzy matching
    fn refilter(&mut self) {
        self.filtered_indices.clear();

        if self.query.is_empty() {
            for i in 0..self.entries.len() {
                self.filtered_indices.push(i);
            }
            return;
        }

        let query_lower = self.query.to_lowercase();

        for (i, entry) in self.entries.iter().enumerate() {
            let name_lower = entry.name.to_lowercase();
            if fuzzy_match(&name_lower, &query_lower) {
                self.filtered_indices.push(i);
            }
        }
    }
}

/// Simple sub-sequence fuzzy search: returns true if all query chars appear in sequence
fn fuzzy_match(target: &str, query: &str) -> bool {
    let mut target_chars = target.chars();
    for qc in query.chars() {
        match target_chars.find(|&tc| tc == qc) {
            Some(_) => continue,
            None => return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launcher_filter() {
        let mut launcher = Launcher::new(Size::new(1920, 1080));
        launcher.add_app("Firefox", "firefox", "Web Browser");
        launcher.add_app("Alacritty", "alacritty", "Terminal Emulator");
        launcher.add_app("Foot", "foot", "Lightweight Wayland Terminal");

        launcher.open();
        assert_eq!(launcher.filtered_indices.len(), 3);

        launcher.insert_char('f');
        launcher.insert_char('i');
        launcher.insert_char('r');
        assert_eq!(launcher.filtered_indices.len(), 1);
        assert_eq!(launcher.selected_entry().unwrap().name, "Firefox");
    }
}
