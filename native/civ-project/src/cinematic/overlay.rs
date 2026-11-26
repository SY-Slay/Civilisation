//! Habillage visuel cinématique (bandes noires letterbox et cartons textuels).

use crate::ui::theme::{colors, ColorRgba};
use crate::ui::widgets::{DrawCommand, Rect};

/// Calcule les bandes letterbox pour simuler un format cinéma (2.39:1 ou cinémascope).
#[derive(Clone, Debug, PartialEq)]
pub struct Letterbox {
    pub bar_height: f32,
    pub screen_width: f32,
    pub screen_height: f32,
}

impl Letterbox {
    pub fn for_screen(width: f32, height: f32) -> Self {
        let target_aspect = 2.39; // Scope standard
        let current_aspect = width / height.max(1.0);

        let bar_height = if current_aspect < target_aspect {
            // Écran plus haut que 2.39:1 -> bandes en haut et en bas
            let visible_h = width / target_aspect;
            ((height - visible_h) * 0.5).max(0.0)
        } else {
            0.0
        };

        Self {
            bar_height,
            screen_width: width,
            screen_height: height,
        }
    }

    /// Émet les rectangles noirs des barres supérieure et inférieure.
    pub fn render_bars(&self, out: &mut Vec<DrawCommand>) {
        if self.bar_height <= 0.0 {
            return;
        }

        let black = ColorRgba::new(0.0, 0.0, 0.0, 1.0);

        // Barre supérieure
        out.push(DrawCommand::FillRect {
            rect: Rect::new(0.0, 0.0, self.screen_width, self.bar_height),
            color: black,
        });

        // Barre inférieure
        out.push(DrawCommand::FillRect {
            rect: Rect::new(
                0.0,
                self.screen_height - self.bar_height,
                self.screen_width,
                self.bar_height,
            ),
            color: black,
        });
    }

    /// Affiche le titre et le sous-titre de la séquence dans la barre inférieure.
    pub fn render_captions(
        &self,
        title: Option<&str>,
        subtitle: Option<&str>,
        out: &mut Vec<DrawCommand>,
    ) {
        let bottom_y = self.screen_height - self.bar_height + 12.0;

        if let Some(t) = title {
            out.push(DrawCommand::Text {
                x: 40.0,
                y: bottom_y,
                text: t.to_uppercase(),
                color: colors::TEXT_HIGHLIGHT,
                scale: 1.2,
            });
        }

        if let Some(st) = subtitle {
            out.push(DrawCommand::Text {
                x: 40.0,
                y: bottom_y + 20.0,
                text: st.to_owned(),
                color: colors::TEXT_MUTED,
                scale: 0.9,
            });
        }
    }
}
