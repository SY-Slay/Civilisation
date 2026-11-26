//! Vue des guerres actives, des fronts et des négociations de paix.

use super::theme::{colors, format_compact_number};
use super::widgets::{DrawCommand, Rect};
use super::UiModel;

/// État de la vue des conflits.
#[derive(Clone, Debug)]
pub struct WarView {
    pub bounds: Rect,
}

impl WarView {
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
            text: "Théâtre des Opérations Militaires".into(),
            color: colors::STATUS_WAR,
            scale: 1.2,
        });
        y += 28.0;

        if model.wars.is_empty() {
            out.push(DrawCommand::Text {
                x: self.bounds.x + pad,
                y,
                text: "Aucun conflit armé en cours. Le monde est en paix.".into(),
                color: colors::STATUS_PEACE,
                scale: 1.0,
            });
            return;
        }

        for war in &model.wars {
            out.push(DrawCommand::FillRect {
                rect: Rect::new(self.bounds.x + pad, y, self.bounds.width - pad * 2.0, 60.0),
                color: colors::HEADER_BG,
            });
            out.push(DrawCommand::StrokeRect {
                rect: Rect::new(self.bounds.x + pad, y, self.bounds.width - pad * 2.0, 60.0),
                color: colors::PANEL_BORDER,
                thickness: 1.0,
            });

            // Nom de la guerre
            out.push(DrawCommand::Text {
                x: self.bounds.x + pad + 12.0,
                y: y + 8.0,
                text: war.name.clone(),
                color: colors::TEXT_HIGHLIGHT,
                scale: 1.0,
            });

            // Belligérants
            let attackers_str = war.attackers.join(", ");
            let defenders_str = war.defenders.join(", ");
            let parties = format!("{} vs {}", attackers_str, defenders_str);
            out.push(DrawCommand::Text {
                x: self.bounds.x + pad + 12.0,
                y: y + 26.0,
                text: parties,
                color: colors::TEXT_PRIMARY,
                scale: 0.9,
            });

            // Statistiques
            let stats = format!(
                "Pertes: {} | Fronts: {} | Batailles: {}",
                format_compact_number(war.casualties),
                war.fronts,
                war.battles
            );
            out.push(DrawCommand::Text {
                x: self.bounds.x + pad + 12.0,
                y: y + 42.0,
                text: stats,
                color: colors::TEXT_MUTED,
                scale: 0.85,
            });

            y += 70.0;
            if y > self.bounds.y + self.bounds.height - 70.0 {
                break;
            }
        }

        if !model.assault_zones.is_empty() {
            y += 8.0;
            out.push(DrawCommand::Text {
                x: self.bounds.x + pad,
                y,
                text: "Zones d'assaut".into(),
                color: colors::TEXT_HIGHLIGHT,
                scale: 1.0,
            });
            y += 20.0;
            for zone in &model.assault_zones {
                let state = if zone.status == "controlled" {
                    format!(
                        "Contrôlée par {}",
                        zone.controller
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "personne".into())
                    )
                } else {
                    format!(
                        "Contestée (A {:.0} / D {:.0})",
                        zone.attacker_progress, zone.defender_progress
                    )
                };
                out.push(DrawCommand::Text {
                    x: self.bounds.x + pad,
                    y,
                    text: format!("{} — {} — score {:.0}", zone.province, state, zone.score),
                    color: colors::TEXT_PRIMARY,
                    scale: 0.85,
                });
                y += 18.0;
                if y > self.bounds.y + self.bounds.height - 20.0 {
                    break;
                }
            }
        }
    }
}
