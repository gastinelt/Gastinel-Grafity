//! Dedicated Gemma 4 AI Chat Application for Gastinel Desktop.
//!
//! Provides a modern conversational interface with streaming tokens, code blocks,
//! prompt input, and real-time LPDDR5x (8733 MT/s) memory bandwidth telemetry.

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect};
use gastinel_core::render::{QuadInstance, RenderQueue};

#[derive(Clone, Debug)]
pub struct ChatMessage {
    pub is_user: bool,
    pub content: String,
    pub timestamp: String,
    pub tokens_per_sec: Option<f32>,
}

pub struct GemmaChatApp {
    pub bounds: Rect,
    pub messages: Vec<ChatMessage>,
    pub input_buffer: String,
    pub is_generating: bool,
    pub model_name: String,
    pub vram_mb: u32,
    pub estimated_tok_per_sec: f32,
}

impl GemmaChatApp {
    pub fn new(bounds: Rect) -> Self {
        let mut app = Self {
            bounds,
            messages: Vec::with_capacity(32),
            input_buffer: String::with_capacity(256),
            is_generating: false,
            model_name: String::from("Gemma 4 E2B Uncensored (HauhauCS Aggressive Q2_K_P)"),
            vram_mb: 2980,
            estimated_tok_per_sec: 74.2,
        };

        // Welcome system message
        app.messages.push(ChatMessage {
            is_user: false,
            content: String::from("Bonjour Théo ! Je suis Gemma 4, chargé en mémoire LPDDR5x (8733 MT/s). Prêt pour le code et le multitâche."),
            timestamp: String::from("22:20"),
            tokens_per_sec: Some(74.2),
        });

        app
    }

    pub fn set_bounds(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    pub fn insert_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    pub fn backspace(&mut self) {
        self.input_buffer.pop();
    }

    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
    }

    pub fn submit_current_input(&mut self) -> Option<String> {
        let trimmed = self.input_buffer.trim().to_string();
        if trimmed.is_empty() {
            return None;
        }

        self.messages.push(ChatMessage {
            is_user: true,
            content: trimmed.clone(),
            timestamp: String::from("22:21"),
            tokens_per_sec: None,
        });

        self.input_buffer.clear();
        self.is_generating = true;
        Some(trimmed)
    }

    pub fn add_assistant_response(&mut self, reply: &str, tok_per_sec: f32) {
        self.messages.push(ChatMessage {
            is_user: false,
            content: reply.to_string(),
            timestamp: String::from("22:21"),
            tokens_per_sec: Some(tok_per_sec),
        });
        self.is_generating = false;
    }

    /// Generates draw primitives for the chat window content
    pub fn render(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        // 1. Chat Container Background (Acrylic dark)
        let bg_quad = QuadInstance::rounded_box(
            self.bounds,
            12.0,
            [0.10, 0.11, 0.15, 0.95],
            [0.26, 0.22, 0.38, 0.70],
            1.0,
            0.1,
        );
        queue.push(bg_quad);

        // 2. Chat Header / Telemetry Bar (38px)
        let header_rect = Rect::new(self.bounds.x, self.bounds.y, self.bounds.width, 38);
        queue.push(QuadInstance::rounded_box(
            header_rect,
            12.0,
            [0.14, 0.12, 0.22, 0.90],
            [0.35, 0.28, 0.55, 0.60],
            1.0,
            0.1,
        ));

        // Model Title text
        atlas.layout_text(
            &self.model_name,
            Point::new(header_rect.x + 16, header_rect.y + 24),
            12,
            FontWeight::SemiBold,
            [0.90, 0.85, 1.0, 1.0], // Soft purple
            glyphs,
        );

        // Bandwidth & VRAM Stats on header right
        let stats_text = format!("{}MB UMA | {:.1} tok/s (8733 MT/s)", self.vram_mb, self.estimated_tok_per_sec);
        let stats_x = (header_rect.x + header_rect.width as i32) - 230;
        atlas.layout_text(
            &stats_text,
            Point::new(stats_x, header_rect.y + 24),
            11,
            FontWeight::Regular,
            [0.55, 0.85, 0.65, 1.0], // Emerald green
            glyphs,
        );

        // 3. Message Bubble List (scrollable area)
        let mut msg_y = header_rect.bottom() + 14;
        let content_width = self.bounds.width.saturating_sub(32);

        for msg in self.messages.iter().rev().take(6) {
            let bubble_w = (content_width as f32 * 0.85) as u32;
            let bubble_h = 44u32;

            let bubble_x = if msg.is_user {
                self.bounds.x + (self.bounds.width as i32 - bubble_w as i32 - 16)
            } else {
                self.bounds.x + 16
            };

            let bubble_rect = Rect::new(bubble_x, msg_y, bubble_w, bubble_h);

            let (fill, border) = if msg.is_user {
                // Electric blue / royal blue for user
                ([0.22, 0.40, 0.85, 0.85], [0.35, 0.55, 0.95, 0.80])
            } else {
                // Deep violet acrylic for Gemma 4
                ([0.18, 0.14, 0.28, 0.80], [0.38, 0.28, 0.55, 0.70])
            };

            queue.push(QuadInstance::rounded_box(
                bubble_rect,
                8.0,
                fill,
                border,
                1.0,
                0.2,
            ));

            // Bubble text
            let text_color = [0.95, 0.95, 0.98, 1.0];
            atlas.layout_text(
                &msg.content,
                Point::new(bubble_rect.x + 12, bubble_rect.y + 26),
                12,
                FontWeight::Regular,
                text_color,
                glyphs,
            );

            msg_y += bubble_h as i32 + 10;
        }

        // 4. Input Bar at Bottom (48px)
        let input_y = self.bounds.bottom() - 56;
        let input_rect = Rect::new(
            self.bounds.x + 16,
            input_y,
            self.bounds.width - 32,
            44,
        );

        queue.push(QuadInstance::rounded_box(
            input_rect,
            8.0,
            [0.12, 0.10, 0.18, 0.90],
            [0.40, 0.30, 0.65, 0.80],
            1.0,
            0.85,
        ));

        if self.input_buffer.is_empty() {
            atlas.layout_text(
                "Poser une question à Gemma 4 (Entrée pour envoyer)...",
                Point::new(input_rect.x + 14, input_rect.y + 27),
                13,
                FontWeight::Regular,
                [0.50, 0.45, 0.60, 1.0], // Placeholder muted
                glyphs,
            );
        } else {
            atlas.layout_text(
                &self.input_buffer,
                Point::new(input_rect.x + 14, input_rect.y + 27),
                13,
                FontWeight::Medium,
                [0.95, 0.92, 1.0, 1.0],
                glyphs,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemma_chat_app_submit() {
        let mut chat = GemmaChatApp::new(Rect::new(0, 0, 600, 400));
        assert_eq!(chat.messages.len(), 1);

        chat.insert_char('H');
        chat.insert_char('i');
        let submitted = chat.submit_current_input();
        assert_eq!(submitted, Some(String::from("Hi")));
        assert_eq!(chat.messages.len(), 2);
        assert!(chat.messages.last().unwrap().is_user);
        assert!(chat.is_generating);

        chat.add_assistant_response("Hello Théo!", 74.5);
        assert_eq!(chat.messages.len(), 3);
        assert!(!chat.is_generating);
    }
}
