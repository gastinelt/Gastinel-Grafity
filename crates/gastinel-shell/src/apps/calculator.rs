//! Calculatrice système pour Gastinel Desktop.
//!
//! Exécute tous les calculs arithmétiques et scientifiques directement
//! en assembleur x86_64 inline via `gastinel_simd::calc_asm`.

use gastinel_core::font::{FontWeight, GlyphInstance, InterAtlas};
use gastinel_core::geom::{Point, Rect};
use gastinel_core::render::{QuadInstance, RenderQueue};
use gastinel_simd::calc_asm::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CalcOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
}

pub struct Calculator {
    pub bounds: Rect,
    pub display: String,
    pub accumulator: f64,
    pub pending_op: Option<CalcOp>,
    pub clear_on_next_digit: bool,
    pub history: Vec<String>,
    pub error: Option<String>,
}

impl Calculator {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            display: String::from("0"),
            accumulator: 0.0,
            pending_op: None,
            clear_on_next_digit: true,
            history: Vec::with_capacity(8),
            error: None,
        }
    }

    pub fn press_digit(&mut self, digit: char) {
        if self.error.is_some() {
            self.press_clear();
        }

        if self.clear_on_next_digit {
            self.display.clear();
            self.clear_on_next_digit = false;
        }

        if self.display == "0" && digit != '.' {
            self.display.clear();
        }

        if self.display.len() < 16 {
            self.display.push(digit);
        }
    }

    pub fn press_dot(&mut self) {
        if self.clear_on_next_digit {
            self.display = String::from("0.");
            self.clear_on_next_digit = false;
            return;
        }

        if !self.display.contains('.') {
            self.display.push('.');
        }
    }

    pub fn press_backspace(&mut self) {
        if self.clear_on_next_digit || self.error.is_some() {
            return;
        }
        self.display.pop();
        if self.display.is_empty() || self.display == "-" {
            self.display = String::from("0");
            self.clear_on_next_digit = true;
        }
    }

    pub fn press_clear(&mut self) {
        self.display = String::from("0");
        self.accumulator = 0.0;
        self.pending_op = None;
        self.clear_on_next_digit = true;
        self.error = None;
    }

    pub fn parse_display(&self) -> f64 {
        self.display.parse::<f64>().unwrap_or(0.0)
    }

    pub fn press_op(&mut self, op: CalcOp) {
        if self.error.is_some() {
            return;
        }

        let current_val = self.parse_display();

        if let Some(prev_op) = self.pending_op {
            if !self.clear_on_next_digit {
                self.execute_op(prev_op, current_val);
            }
        } else {
            self.accumulator = current_val;
        }

        self.pending_op = Some(op);
        self.clear_on_next_digit = true;
    }

    pub fn press_equals(&mut self) {
        if self.error.is_some() {
            return;
        }

        if let Some(op) = self.pending_op.take() {
            let current_val = self.parse_display();
            self.execute_op(op, current_val);
            self.clear_on_next_digit = true;
        }
    }

    fn execute_op(&mut self, op: CalcOp, rhs: f64) {
        let lhs = self.accumulator;
        let (res, op_sym) = match op {
            CalcOp::Add => (asm_add(lhs, rhs), "+"),
            CalcOp::Sub => (asm_sub(lhs, rhs), "−"),
            CalcOp::Mul => (asm_mul(lhs, rhs), "×"),
            CalcOp::Div => {
                if rhs == 0.0 {
                    self.error = Some(String::from("Div par 0"));
                    self.display = String::from("Erreur");
                    return;
                }
                (asm_div(lhs, rhs), "÷")
            }
            CalcOp::Mod => (asm_mod(lhs, rhs), "%"),
            CalcOp::Pow => (asm_pow_int(lhs, rhs as i32), "^"),
        };

        let hist = format!("{:.4} {} {:.4} = {:.4}", lhs, op_sym, rhs, res);
        self.history.push(hist);
        if self.history.len() > 6 {
            self.history.remove(0);
        }

        self.accumulator = res;
        self.display = format_float(res);
    }

    pub fn press_sqrt(&mut self) {
        let val = self.parse_display();
        if val < 0.0 {
            self.error = Some(String::from("Racine négative"));
            self.display = String::from("Erreur");
            return;
        }
        let res = asm_sqrt(val);
        self.accumulator = res;
        self.history.push(format!("√({:.4}) = {:.4}", val, res));
        self.display = format_float(res);
        self.clear_on_next_digit = true;
    }

    pub fn press_negate(&mut self) {
        let val = self.parse_display();
        let res = asm_neg(val);
        self.accumulator = res;
        self.display = format_float(res);
    }

    pub fn press_sin(&mut self) {
        let val = self.parse_display();
        let res = asm_sin(val);
        self.accumulator = res;
        self.history.push(format!("sin({:.4}) = {:.4}", val, res));
        self.display = format_float(res);
        self.clear_on_next_digit = true;
    }

    pub fn press_cos(&mut self) {
        let val = self.parse_display();
        let res = asm_cos(val);
        self.accumulator = res;
        self.history.push(format!("cos({:.4}) = {:.4}", val, res));
        self.display = format_float(res);
        self.clear_on_next_digit = true;
    }

    /// Rendu UI complet de la calculatrice dans la RenderQueue.
    pub fn render(
        &self,
        atlas: &InterAtlas,
        queue: &mut RenderQueue,
        glyphs: &mut Vec<GlyphInstance>,
    ) {
        // 1. Fond de fenêtre sombre acrylique
        queue.push(
            QuadInstance::rounded_box(
                self.bounds,
                14.0,
                [0.11, 0.12, 0.16, 0.98],
                [0.24, 0.26, 0.35, 0.8],
                1.5,
                0.2,
            )
            .with_shadow([0.0, 0.0, 0.0, 0.45], 24.0, [0.0, 8.0]),
        );

        // 2. Écran d'affichage (Display Screen)
        let display_rect = Rect::new(
            self.bounds.x + 14,
            self.bounds.y + 16,
            self.bounds.width - 28,
            86,
        );
        queue.push(QuadInstance::rounded_box(
            display_rect,
            10.0,
            [0.07, 0.08, 0.11, 0.95],
            [0.18, 0.20, 0.28, 0.6],
            1.0,
            0.21,
        ));

        // Historique récent
        if let Some(last_hist) = self.history.last() {
            atlas.layout_text(
                last_hist,
                Point::new(display_rect.x + 12, display_rect.y + 24),
                11,
                FontWeight::Regular,
                [0.55, 0.58, 0.68, 1.0],
                glyphs,
            );
        }

        // Chiffres affichés (grand texte)
        let display_color = if self.error.is_some() {
            [0.95, 0.35, 0.35, 1.0]
        } else {
            [0.96, 0.97, 1.0, 1.0]
        };
        atlas.layout_text(
            &self.display,
            Point::new(display_rect.x + 12, display_rect.y + 64),
            26,
            FontWeight::Bold,
            display_color,
            glyphs,
        );

        // 3. Grille des boutons 5 lignes x 4 colonnes
        let pad_x = self.bounds.x + 14;
        let pad_y = display_rect.bottom() + 14;
        let btn_w = (self.bounds.width - 28 - (3 * 8)) / 4;
        let btn_h = 42u32;

        let buttons: [[(&str, [f32; 4]); 4]; 5] = [
            [("C", [0.75, 0.22, 0.26, 0.9]), ("√", [0.22, 0.25, 0.35, 0.9]), ("%", [0.22, 0.25, 0.35, 0.9]), ("÷", [0.35, 0.40, 0.85, 0.95])],
            [("7", [0.16, 0.18, 0.24, 0.9]), ("8", [0.16, 0.18, 0.24, 0.9]), ("9", [0.16, 0.18, 0.24, 0.9]), ("×", [0.35, 0.40, 0.85, 0.95])],
            [("4", [0.16, 0.18, 0.24, 0.9]), ("5", [0.16, 0.18, 0.24, 0.9]), ("6", [0.16, 0.18, 0.24, 0.9]), ("−", [0.35, 0.40, 0.85, 0.95])],
            [("1", [0.16, 0.18, 0.24, 0.9]), ("2", [0.16, 0.18, 0.24, 0.9]), ("3", [0.16, 0.18, 0.24, 0.9]), ("+", [0.35, 0.40, 0.85, 0.95])],
            [("±", [0.16, 0.18, 0.24, 0.9]), ("0", [0.16, 0.18, 0.24, 0.9]), (".", [0.16, 0.18, 0.24, 0.9]), ("=", [0.45, 0.30, 0.85, 0.95])],
        ];

        for (row_idx, row) in buttons.iter().enumerate() {
            for (col_idx, (label, color)) in row.iter().enumerate() {
                let bx = pad_x + (col_idx as i32) * (btn_w as i32 + 8);
                let by = pad_y + (row_idx as i32) * (btn_h as i32 + 8);
                let btn_rect = Rect::new(bx, by, btn_w, btn_h);

                queue.push(QuadInstance::rounded_box(
                    btn_rect,
                    8.0,
                    *color,
                    [color[0] * 1.2, color[1] * 1.2, color[2] * 1.2, 0.6],
                    1.0,
                    0.22,
                ));

                atlas.layout_text(
                    label,
                    Point::new(bx + (btn_w as i32 / 2) - 6, by + 27),
                    15,
                    FontWeight::SemiBold,
                    [0.98, 0.98, 1.0, 1.0],
                    glyphs,
                );
            }
        }
    }
}

fn format_float(val: f64) -> String {
    if val.fract() == 0.0 && val.abs() < 1e12 {
        format!("{:.0}", val)
    } else {
        let s = format!("{:.6}", val);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculator_basic() {
        let mut calc = Calculator::new(Rect::new(0, 0, 320, 440));
        calc.press_digit('5');
        calc.press_op(CalcOp::Add);
        calc.press_digit('7');
        calc.press_equals();
        assert_eq!(calc.display, "12");

        calc.press_op(CalcOp::Mul);
        calc.press_digit('3');
        calc.press_equals();
        assert_eq!(calc.display, "36");
    }

    #[test]
    fn test_calculator_div_zero() {
        let mut calc = Calculator::new(Rect::new(0, 0, 320, 440));
        calc.press_digit('8');
        calc.press_op(CalcOp::Div);
        calc.press_digit('0');
        calc.press_equals();
        assert_eq!(calc.display, "Erreur");
        assert!(calc.error.is_some());
    }

    #[test]
    fn test_calculator_sqrt_and_neg() {
        let mut calc = Calculator::new(Rect::new(0, 0, 320, 440));
        calc.press_digit('9');
        calc.press_sqrt();
        assert_eq!(calc.display, "3");
        calc.press_negate();
        assert_eq!(calc.display, "-3");
    }
}
