//! Gastinel Write — Traitement de Texte et Prise de Notes Enrichie.
//!
//! Adapté de l'architecture native Gastinel Write :
//! - Modèle de document en liste plate de paragraphes
//! - Niveaux de titres (H1, H2, H3), listes à puces et numérotées
//! - Plages de style de caractères : Gras, Italique, Souligné
//! - Reconnaissance Markdown en direct au fil de la frappe (`markdown_live`)
//! - Barre d'outils de mise en forme riche et écran d'accueil avec notes récentes
//! - Moteur d'annulation / rétablissement (Undo / Redo)

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect};
use gastinel_core::render::{QuadInstance, RenderQueue};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HeadingLevel {
    None,
    H1,
    H2,
    H3,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ListKind {
    None,
    Bullet,
    Numbered,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StyleSpan {
    pub start: usize,
    pub length: usize,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

#[derive(Clone, Debug)]
pub struct Paragraph {
    pub text: String,
    pub heading: HeadingLevel,
    pub list: ListKind,
    pub align: Alignment,
    pub spans: Vec<StyleSpan>,
}

impl Paragraph {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            heading: HeadingLevel::None,
            list: ListKind::None,
            align: Alignment::Left,
            spans: Vec::new(),
        }
    }

    pub fn with_heading(mut self, heading: HeadingLevel) -> Self {
        self.heading = heading;
        self
    }

    pub fn with_list(mut self, list: ListKind) -> Self {
        self.list = list;
        self
    }
}

pub struct RecentNote {
    pub title: String,
    pub date: String,
    pub snippet: String,
}

#[derive(Clone, Debug)]
pub struct UndoSnapshot {
    pub paragraphs: Vec<Paragraph>,
    pub caret_p: usize,
    pub caret_c: usize,
}

pub struct GastinelWriteApp {
    pub bounds: Rect,
    pub document_name: String,
    pub paragraphs: Vec<Paragraph>,
    pub caret_p: usize,
    pub caret_c: usize,
    pub undo_stack: Vec<UndoSnapshot>,
    pub redo_stack: Vec<UndoSnapshot>,
    pub show_welcome: bool,
    pub recent_notes: Vec<RecentNote>,
    pub bold_active: bool,
    pub italic_active: bool,
}

impl GastinelWriteApp {
    pub fn new(bounds: Rect) -> Self {
        let mut recent_notes = Vec::with_capacity(4);
        recent_notes.push(RecentNote {
            title: String::from("Spécifications Gastinel OS"),
            date: String::from("15 sept. 22:10"),
            snippet: String::from("Architecture noyau Alpine Linux, Vulkan Direct, UMA 8733MT/s..."),
        });
        recent_notes.push(RecentNote {
            title: String::from("Notes Gemma 4"),
            date: String::from("14 sept. 18:30"),
            snippet: String::from("Inférence locale 74 tok/s avec modèle 3.0 Go en Q2_K_P."),
        });

        let mut doc = Vec::new();
        let mut p1 = Paragraph::new("Gastinel Write").with_heading(HeadingLevel::H1);
        p1.spans.push(StyleSpan { start: 0, length: 14, bold: true, italic: false, underline: false });
        doc.push(p1);

        doc.push(Paragraph::new("Application native de bureautique et prise de notes riche."));
        doc.push(Paragraph::new("Fonctionnalités prises en charge :").with_heading(HeadingLevel::H2));
        doc.push(Paragraph::new("Mise en forme Markdown en direct au clavier (**gras**, *italique*)").with_list(ListKind::Bullet));
        doc.push(Paragraph::new("Titres structurés H1, H2, H3 et listes à puces").with_list(ListKind::Bullet));
        doc.push(Paragraph::new("Zéro dépendance, rendu vectoriel Inter pur").with_list(ListKind::Bullet));

        Self {
            bounds,
            document_name: String::from("Document sans titre"),
            paragraphs: doc,
            caret_p: 1,
            caret_c: 0,
            undo_stack: Vec::with_capacity(32),
            redo_stack: Vec::with_capacity(32),
            show_welcome: false,
            recent_notes,
            bold_active: false,
            italic_active: false,
        }
    }

    fn push_undo(&mut self) {
        self.undo_stack.push(UndoSnapshot {
            paragraphs: self.paragraphs.clone(),
            caret_p: self.caret_p,
            caret_c: self.caret_c,
        });
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(snap) = self.undo_stack.pop() {
            self.redo_stack.push(UndoSnapshot {
                paragraphs: self.paragraphs.clone(),
                caret_p: self.caret_p,
                caret_c: self.caret_c,
            });
            self.paragraphs = snap.paragraphs;
            self.caret_p = snap.caret_p;
            self.caret_c = snap.caret_c;
        }
    }

    pub fn redo(&mut self) {
        if let Some(snap) = self.redo_stack.pop() {
            self.undo_stack.push(UndoSnapshot {
                paragraphs: self.paragraphs.clone(),
                caret_p: self.caret_p,
                caret_c: self.caret_c,
            });
            self.paragraphs = snap.paragraphs;
            self.caret_p = snap.caret_p;
            self.caret_c = snap.caret_c;
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.push_undo();

        if self.caret_p >= self.paragraphs.len() {
            self.paragraphs.push(Paragraph::new(""));
        }

        let p = &mut self.paragraphs[self.caret_p];
        let idx = self.caret_c.min(p.text.len());
        p.text.insert(idx, c);
        self.caret_c += 1;

        // Reconnaissance Markdown en direct à la frappe (comme Gastinel Write doc/markdown_live.cpp)
        self.check_markdown_live();
    }

    pub fn insert_newline(&mut self) {
        self.push_undo();

        if self.caret_p < self.paragraphs.len() {
            let cur_text = self.paragraphs[self.caret_p].text.clone();
            let split_idx = self.caret_c.min(cur_text.len());
            let left = cur_text[..split_idx].to_string();
            let right = cur_text[split_idx..].to_string();

            self.paragraphs[self.caret_p].text = left;
            let mut new_p = Paragraph::new(&right);
            // Conserver la liste si en cours
            if self.paragraphs[self.caret_p].list != ListKind::None {
                new_p.list = self.paragraphs[self.caret_p].list;
            }
            self.paragraphs.insert(self.caret_p + 1, new_p);
            self.caret_p += 1;
            self.caret_c = 0;
        }
    }

    pub fn backspace(&mut self) {
        if self.caret_p < self.paragraphs.len() {
            if self.caret_c > 0 {
                self.push_undo();
                let p = &mut self.paragraphs[self.caret_p];
                self.caret_c -= 1;
                p.text.remove(self.caret_c);
            } else if self.caret_p > 0 {
                self.push_undo();
                let cur = self.paragraphs.remove(self.caret_p);
                self.caret_p -= 1;
                let prev_len = self.paragraphs[self.caret_p].text.len();
                self.paragraphs[self.caret_p].text.push_str(&cur.text);
                self.caret_c = prev_len;
            }
        }
    }

    /// Reconnaissance Markdown en direct au fil de la frappe
    fn check_markdown_live(&mut self) {
        if self.caret_p >= self.paragraphs.len() {
            return;
        }
        let p = &mut self.paragraphs[self.caret_p];

        // 1. Détection des titres (# , ## , ### )
        if p.text.starts_with("### ") {
            p.text = p.text[4..].to_string();
            p.heading = HeadingLevel::H3;
            self.caret_c = self.caret_c.saturating_sub(4);
            return;
        } else if p.text.starts_with("## ") {
            p.text = p.text[3..].to_string();
            p.heading = HeadingLevel::H2;
            self.caret_c = self.caret_c.saturating_sub(3);
            return;
        } else if p.text.starts_with("# ") {
            p.text = p.text[2..].to_string();
            p.heading = HeadingLevel::H1;
            self.caret_c = self.caret_c.saturating_sub(2);
            return;
        }

        // 2. Détection des listes (- )
        if p.text.starts_with("- ") {
            p.text = p.text[2..].to_string();
            p.list = ListKind::Bullet;
            self.caret_c = self.caret_c.saturating_sub(2);
            return;
        }

        // 3. Détection de gras en direct (**mot**)
        if p.text.len() >= 4 && p.text.ends_with("**") {
            if let Some(first_idx) = p.text[..p.text.len() - 2].rfind("**") {
                let inner = p.text[first_idx + 2..p.text.len() - 2].to_string();
                if !inner.is_empty() && !inner.contains('*') {
                    p.text = format!("{}{}", &p.text[..first_idx], inner);
                    p.spans.push(StyleSpan {
                        start: first_idx,
                        length: inner.len(),
                        bold: true,
                        italic: false,
                        underline: false,
                    });
                    self.caret_c = p.text.len();
                }
            }
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
                12.0,
                [0.10, 0.11, 0.15, 0.98],
                [0.25, 0.28, 0.40, 0.7],
                1.5,
                0.2,
            )
            .with_shadow([0.0, 0.0, 0.0, 0.45], 24.0, [0.0, 6.0]),
        );

        // Barre d'outils supérieure de mise en forme (Toolbar 40px)
        let tb_rect = Rect::new(self.bounds.x + 12, self.bounds.y + 12, self.bounds.width - 24, 38);
        queue.push(QuadInstance::rounded_box(
            tb_rect,
            8.0,
            [0.14, 0.15, 0.20, 0.95],
            [0.22, 0.25, 0.35, 0.5],
            1.0,
            0.21,
        ));

        // Nom du document
        atlas.layout_text(
            &format!("📝 {}", self.document_name),
            Point::new(tb_rect.x + 10, tb_rect.y + 24),
            12,
            FontWeight::SemiBold,
            [0.95, 0.96, 1.0, 1.0],
            glyphs,
        );

        // Boutons de style dans la barre d'outils
        let tools = [
            ("B", self.bold_active),
            ("I", self.italic_active),
            ("H1", false),
            ("H2", false),
            ("• Liste", false),
            ("↩ Undo", false),
        ];

        let mut bx = tb_rect.right() - 250;
        for (label, active) in tools.iter() {
            let btn_w = 34u32;
            let b_rect = Rect::new(bx, tb_rect.y + 5, btn_w, 28);
            let (bg, border) = if *active {
                ([0.30, 0.45, 0.90, 0.9], [0.5, 0.65, 1.0, 0.8])
            } else {
                ([0.18, 0.20, 0.28, 0.8], [0.25, 0.28, 0.38, 0.4])
            };

            queue.push(QuadInstance::rounded_box(b_rect, 4.0, bg, border, 1.0, 0.22));
            atlas.layout_text(label, Point::new(bx + 8, b_rect.y + 19), 11, FontWeight::SemiBold, [0.9, 0.92, 0.98, 1.0], glyphs);
            bx += btn_w as i32 + 5;
        }

        // Zone de page de document (Page blanche/crème douce ou sombre)
        let page_margin = 18i32;
        let page_rect = Rect::new(
            self.bounds.x + page_margin,
            tb_rect.bottom() + 10,
            self.bounds.width - (page_margin as u32 * 2),
            (self.bounds.bottom() - 36).saturating_sub(tb_rect.bottom() + 10) as u32,
        );

        queue.push(QuadInstance::rounded_box(
            page_rect,
            8.0,
            [0.12, 0.13, 0.17, 0.95],
            [0.18, 0.20, 0.26, 0.4],
            1.0,
            0.21,
        ));

        // Rendu des paragraphes du document
        let mut py = page_rect.y + 24;
        for (p_idx, p) in self.paragraphs.iter().enumerate() {
            let (font_size, font_weight, text_color, line_h, prefix) = match p.heading {
                HeadingLevel::H1 => (20u16, FontWeight::Bold, [1.0, 1.0, 1.0, 1.0], 30i32, ""),
                HeadingLevel::H2 => (16u16, FontWeight::SemiBold, [0.90, 0.92, 1.0, 1.0], 26i32, ""),
                HeadingLevel::H3 => (14u16, FontWeight::SemiBold, [0.85, 0.88, 0.98, 1.0], 22i32, ""),
                HeadingLevel::None => match p.list {
                    ListKind::Bullet => (12u16, FontWeight::Regular, [0.85, 0.88, 0.95, 1.0], 20i32, "• "),
                    ListKind::Numbered => (12u16, FontWeight::Regular, [0.85, 0.88, 0.95, 1.0], 20i32, "1. "),
                    ListKind::None => (12u16, FontWeight::Regular, [0.85, 0.88, 0.95, 1.0], 20i32, ""),
                },
            };

            let px = if p.list != ListKind::None { page_rect.x + 36 } else { page_rect.x + 24 };
            let full_text = format!("{}{}", prefix, p.text);

            atlas.layout_text(&full_text, Point::new(px, py), font_size, font_weight, text_color, glyphs);

            // Curseur de saisie si paragraphe actif
            if p_idx == self.caret_p {
                let advance = (self.caret_c as i32 + prefix.len() as i32) * 7;
                let caret_rect = Rect::new(px + advance, py - (font_size as i32), 2, font_size as u32 + 2);
                queue.push(QuadInstance::solid(caret_rect, [0.45, 0.75, 1.0, 0.95], 0.23));
            }

            py += line_h;
            if py > page_rect.bottom() - 20 {
                break;
            }
        }

        // Barre d'état inférieure
        let status_y = self.bounds.bottom() - 26;
        let total_words: usize = self.paragraphs.iter().map(|p| p.text.split_whitespace().count()).sum();
        let status_str = format!("Paragraphe {}/{} • {} mots • Encodage UTF-8 • Mode Markdown Live",
            self.caret_p + 1, self.paragraphs.len(), total_words);

        atlas.layout_text(
            &status_str,
            Point::new(self.bounds.x + 20, status_y + 16),
            11,
            FontWeight::Regular,
            [0.50, 0.55, 0.65, 1.0],
            glyphs,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_markdown_live_headings() {
        let mut app = GastinelWriteApp::new(Rect::new(0, 0, 700, 500));
        app.paragraphs.clear();
        app.paragraphs.push(Paragraph::new(""));
        app.caret_p = 0;

        for c in "# Titre".chars() {
            app.insert_char(c);
        }
        assert_eq!(app.paragraphs[0].heading, HeadingLevel::H1);
        assert_eq!(app.paragraphs[0].text, "Titre");
    }

    #[test]
    fn test_write_markdown_live_bold() {
        let mut app = GastinelWriteApp::new(Rect::new(0, 0, 700, 500));
        app.paragraphs.clear();
        app.paragraphs.push(Paragraph::new(""));
        app.caret_p = 0;

        for c in "**important**".chars() {
            app.insert_char(c);
        }
        assert_eq!(app.paragraphs[0].text, "important");
        assert_eq!(app.paragraphs[0].spans.len(), 1);
        assert!(app.paragraphs[0].spans[0].bold);
    }

    #[test]
    fn test_write_undo_redo() {
        let mut app = GastinelWriteApp::new(Rect::new(0, 0, 700, 500));
        app.paragraphs.clear();
        app.paragraphs.push(Paragraph::new("Hello"));
        app.caret_p = 0;
        app.caret_c = 5;

        app.insert_char('!');
        assert_eq!(app.paragraphs[0].text, "Hello!");

        app.undo();
        assert_eq!(app.paragraphs[0].text, "Hello");

        app.redo();
        assert_eq!(app.paragraphs[0].text, "Hello!");
    }
}
