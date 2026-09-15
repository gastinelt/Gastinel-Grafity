//! Gestionnaire de Fichiers (File Manager) pour Gastinel Desktop.
//!
//! Navigation arborescente, fil d'Ariane (breadcrumb), tri, recherche,
//! panneau de prévisualisation texte et raccourcis favoris.

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect};
use gastinel_core::render::{QuadInstance, RenderQueue};
use std::path::PathBuf;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FileSort {
    NomAZ,
    NomZA,
    Taille,
    Date,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FileItem {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub extension: String,
    pub is_selected: bool,
}

pub struct FilesApp {
    pub bounds: Rect,
    pub current_path: PathBuf,
    pub entries: Vec<FileItem>,
    pub selected_index: Option<usize>,
    pub search_query: String,
    pub sort: FileSort,
    pub show_preview: bool,
    pub preview_text: Option<String>,
    pub favorites: Vec<(&'static str, &'static str, &'static str)>, // (icone, nom, chemin)
}

impl FilesApp {
    pub fn new(bounds: Rect) -> Self {
        let initial_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let favorites = vec![
            ("🏠", "Accueil", "."),
            ("📄", "Documents", "./docs"),
            ("🦀", "Crates", "./crates"),
            ("⚙️", "Configuration", "./rootfs-overlay/etc"),
            ("💾", "SSD NVMe", "/"),
        ];

        let mut app = Self {
            bounds,
            current_path: initial_path,
            entries: Vec::with_capacity(64),
            selected_index: None,
            search_query: String::new(),
            sort: FileSort::NomAZ,
            show_preview: true,
            preview_text: None,
            favorites,
        };

        app.refresh_directory();
        app
    }

    pub fn navigate_to(&mut self, path: PathBuf) {
        if path.is_dir() {
            self.current_path = path;
            self.selected_index = None;
            self.preview_text = None;
            self.refresh_directory();
        }
    }

    pub fn navigate_up(&mut self) {
        if let Some(parent) = self.current_path.parent().map(|p| p.to_path_buf()) {
            self.navigate_to(parent);
        }
    }

    pub fn open_selected(&mut self) {
        if let Some(idx) = self.selected_index {
            if let Some(item) = self.entries.get(idx) {
                if item.is_dir {
                    let dir_path = item.path.clone();
                    self.navigate_to(dir_path);
                } else {
                    self.load_preview(idx);
                }
            }
        }
    }

    pub fn select_item(&mut self, idx: usize) {
        if idx < self.entries.len() {
            self.selected_index = Some(idx);
            self.load_preview(idx);
        }
    }

    fn load_preview(&mut self, idx: usize) {
        if let Some(item) = self.entries.get(idx) {
            if !item.is_dir {
                if let Ok(content) = std::fs::read_to_string(&item.path) {
                    let preview = content.lines().take(8).collect::<Vec<&str>>().join("\n");
                    self.preview_text = Some(preview);
                    return;
                }
            }
        }
        self.preview_text = None;
    }

    pub fn refresh_directory(&mut self) {
        self.entries.clear();

        if let Ok(read_dir) = std::fs::read_dir(&self.current_path) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();

                if file_name.starts_with('.') && file_name != ".git" {
                    continue;
                }

                let is_dir = path.is_dir();
                let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

                self.entries.push(FileItem {
                    name: file_name,
                    path,
                    is_dir,
                    size_bytes,
                    extension,
                    is_selected: false,
                });
            }
        }

        self.sort_entries();
    }

    pub fn sort_entries(&mut self) {
        match self.sort {
            FileSort::NomAZ => {
                self.entries.sort_by(|a, b| {
                    b.is_dir.cmp(&a.is_dir).then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                });
            }
            FileSort::NomZA => {
                self.entries.sort_by(|a, b| {
                    b.is_dir.cmp(&a.is_dir).then_with(|| b.name.to_lowercase().cmp(&a.name.to_lowercase()))
                });
            }
            FileSort::Taille => {
                self.entries.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
            }
            FileSort::Date => {}
        }
    }

    pub fn format_size(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} o", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.1} Ko", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} Mo", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} Go", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    fn file_icon(is_dir: bool, ext: &str) -> &'static str {
        if is_dir {
            return "📁";
        }
        match ext {
            "rs" => "🦀",
            "toml" => "⚙️",
            "md" | "txt" => "📝",
            "sh" | "bash" => "🐚",
            "png" | "jpg" | "jpeg" | "bmp" => "🖼️",
            "json" | "yaml" | "yml" => "📋",
            "lock" => "🔒",
            _ => "📄",
        }
    }

    pub fn render(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        // Fond de fenêtre principal
        queue.push(
            QuadInstance::rounded_box(
                self.bounds,
                14.0,
                [0.09, 0.10, 0.13, 0.98],
                [0.22, 0.25, 0.35, 0.7],
                1.5,
                0.2,
            )
            .with_shadow([0.0, 0.0, 0.0, 0.45], 24.0, [0.0, 6.0]),
        );

        // Barre d'outils supérieure & Breadcrumb
        let toolbar_rect = Rect::new(self.bounds.x + 12, self.bounds.y + 12, self.bounds.width - 24, 38);
        queue.push(QuadInstance::rounded_box(
            toolbar_rect,
            8.0,
            [0.13, 0.14, 0.19, 0.9],
            [0.20, 0.22, 0.30, 0.5],
            1.0,
            0.21,
        ));

        // Bouton retour parent
        let up_btn = Rect::new(toolbar_rect.x + 6, toolbar_rect.y + 4, 30, 30);
        queue.push(QuadInstance::rounded_box(up_btn, 6.0, [0.18, 0.20, 0.28, 0.8], [0.3, 0.35, 0.45, 0.5], 1.0, 0.22));
        atlas.layout_text("↑", Point::new(up_btn.x + 10, up_btn.y + 21), 14, FontWeight::Bold, [0.95, 0.96, 1.0, 1.0], glyphs);

        // Fil d'Ariane
        let breadcrumb_text = self.current_path.to_string_lossy().to_string();
        atlas.layout_text(
            &breadcrumb_text,
            Point::new(up_btn.right() + 12, toolbar_rect.y + 24),
            12,
            FontWeight::SemiBold,
            [0.85, 0.88, 0.98, 1.0],
            glyphs,
        );

        let content_y = toolbar_rect.bottom() + 10;
        let content_h = (self.bounds.bottom() - 14).saturating_sub(content_y) as u32;

        // Sidebar Favoris gauche (150px)
        let sidebar_w = 150u32;
        let sidebar_rect = Rect::new(self.bounds.x + 12, content_y, sidebar_w, content_h);
        queue.push(QuadInstance::rounded_box(
            sidebar_rect,
            8.0,
            [0.11, 0.12, 0.16, 0.85],
            [0.18, 0.20, 0.26, 0.4],
            1.0,
            0.21,
        ));

        atlas.layout_text("Accès Rapide", Point::new(sidebar_rect.x + 10, sidebar_rect.y + 22), 11, FontWeight::Bold, [0.65, 0.68, 0.78, 1.0], glyphs);

        let mut fy = sidebar_rect.y + 36;
        for (icon, name, _) in self.favorites.iter() {
            let fav_btn = Rect::new(sidebar_rect.x + 6, fy, sidebar_rect.width - 12, 28);
            let text = format!("{} {}", icon, name);
            atlas.layout_text(&text, Point::new(fav_btn.x + 6, fav_btn.y + 19), 11, FontWeight::Regular, [0.85, 0.88, 0.95, 1.0], glyphs);
            fy += 32;
        }

        // Zone principale liste des fichiers
        let list_x = sidebar_rect.right() + 10;
        let preview_w = if self.show_preview && self.preview_text.is_some() { 200u32 } else { 0u32 };
        let list_w = (self.bounds.right() - 12 - (preview_w as i32)).saturating_sub(list_x) as u32;
        let list_rect = Rect::new(list_x, content_y, list_w, content_h);

        queue.push(QuadInstance::rounded_box(
            list_rect,
            8.0,
            [0.11, 0.12, 0.16, 0.9],
            [0.18, 0.20, 0.26, 0.4],
            1.0,
            0.21,
        ));

        // Lignes de fichiers
        let mut row_y = list_rect.y + 8;
        for (idx, item) in self.entries.iter().take(12).enumerate() {
            let is_sel = self.selected_index == Some(idx);
            let row_rect = Rect::new(list_rect.x + 6, row_y, list_rect.width - 12, 28);

            if is_sel {
                queue.push(QuadInstance::rounded_box(
                    row_rect,
                    4.0,
                    [0.28, 0.35, 0.85, 0.9],
                    [0.45, 0.55, 1.0, 0.8],
                    1.0,
                    0.22,
                ));
            }

            let icon = Self::file_icon(item.is_dir, &item.extension);
            let row_text = format!("{}  {}", icon, item.name);
            let text_color = if is_sel { [1.0, 1.0, 1.0, 1.0] } else { [0.90, 0.92, 0.96, 1.0] };

            atlas.layout_text(&row_text, Point::new(row_rect.x + 8, row_rect.y + 19), 11, FontWeight::Regular, text_color, glyphs);

            let size_text = if item.is_dir { String::from("Dossier") } else { Self::format_size(item.size_bytes) };
            let sz_x = row_rect.right() - 80;
            atlas.layout_text(&size_text, Point::new(sz_x, row_rect.y + 19), 11, FontWeight::Regular, [0.55, 0.60, 0.70, 1.0], glyphs);

            row_y += 30;
        }

        // Panneau de prévisualisation texte à droite
        if self.show_preview {
            if let Some(ref preview) = self.preview_text {
                let prev_x = list_rect.right() + 8;
                let prev_rect = Rect::new(prev_x, content_y, preview_w, content_h);

                queue.push(QuadInstance::rounded_box(
                    prev_rect,
                    8.0,
                    [0.08, 0.09, 0.12, 0.95],
                    [0.24, 0.28, 0.40, 0.6],
                    1.0,
                    0.21,
                ));

                atlas.layout_text("Aperçu Rapide", Point::new(prev_rect.x + 10, prev_rect.y + 22), 11, FontWeight::Bold, [0.85, 0.75, 1.0, 1.0], glyphs);

                let mut py = prev_rect.y + 42;
                for line in preview.lines().take(10) {
                    let display_line: String = line.chars().take(22).collect();
                    atlas.layout_text(&display_line, Point::new(prev_rect.x + 8, py), 10, FontWeight::Regular, [0.70, 0.75, 0.85, 1.0], glyphs);
                    py += 18;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(FilesApp::format_size(500), "500 o");
        assert_eq!(FilesApp::format_size(2048), "2.0 Ko");
        assert_eq!(FilesApp::format_size(10 * 1024 * 1024), "10.0 Mo");
    }

    #[test]
    fn test_files_app_navigation() {
        let app = FilesApp::new(Rect::new(0, 0, 720, 480));
        assert!(!app.favorites.is_empty());
        assert!(!app.entries.is_empty());
    }
}
