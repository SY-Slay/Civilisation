//! Vue des missiles balistiques, silos et défenses anti-aériennes.

use super::theme::colors;
use super::widgets::{DrawCommand, Rect};
use super::UiModel;

/// État de la vue de commandement des missiles.
#[derive(Clone, Debug)]
pub struct MissileView {
    pub bounds: Rect,
}

impl MissileView {
    pub fn new(bounds: Rect) -> Self {
        Self { bounds }
    }

    pub fn render(&self, model: &UiModel, out: &mut Vec<DrawCommand>) {
        out.push(DrawCommand::FillRect {
            rect: self.bounds,
            color: colors::PANEL_BG,
        });
        out.push(DrawCommand::StrokeRect {
            rect: self.bounds,
            color: colors::PANEL_BORDER,
            thickness: 1.0,
        });

        let pad = 16.0;
        let mut y = self.bounds.y + pad;

        out.push(DrawCommand::Text {
            x: self.bounds.x + pad,
            y,
            text: "Commandement des Forces Stratégiques (Missiles)".into(),
            color: colors::STATUS_TENSION,
            scale: 1.2,
        });
        y += 28.0;

        if model.missiles.is_empty() {
            out.push(DrawCommand::Text {
                x: self.bounds.x + pad,
                y,
                text: "Aucun missile actuellement détecté en vol.".into(),
                color: colors::TEXT_MUTED,
                scale: 1.0,
            });
            return;
        }

        for missile in &model.missiles {
            out.push(DrawCommand::FillRect {
                rect: Rect::new(self.bounds.x + pad, y, self.bounds.width - pad * 2.0, 48.0),
                color: colors::HEADER_BG,
            });
            out.push(DrawCommand::StrokeRect {
                rect: Rect::new(self.bounds.x + pad, y, self.bounds.width - pad * 2.0, 48.0),
                color: colors::STATUS_TENSION,
                thickness: 1.0,
            });

            out.push(DrawCommand::Text {
                x: self.bounds.x + pad + 12.0,
                y: y + 8.0,
                text: format!("ALERTE BALISTIQUE — ID {}", missile.id),
                color: colors::STATUS_WAR,
                scale: 1.0,
            });

            let details = format!(
                "Niveau de danger {}/5 | Origine: {} | Cible: {} | Statut: {} | Impact prévu au tick {}",
                missile.danger_tier,
                missile.owner,
                missile.target,
                missile.status,
                missile.arrival_tick.as_u64()
            );
            out.push(DrawCommand::Text {
                x: self.bounds.x + pad + 12.0,
                y: y + 26.0,
                text: details,
                color: colors::TEXT_PRIMARY,
                scale: 0.9,
            });

            y += 58.0;
            if y > self.bounds.y + self.bounds.height - 60.0 {
                break;
            }
        }
    }
}
