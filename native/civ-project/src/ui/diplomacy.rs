//! Vue de la diplomatie, des traités et des alliances.

use super::theme::{colors, format_compact_number};
use super::widgets::{DrawCommand, Rect};
use super::UiModel;

/// État de la vue de diplomatie.
#[derive(Clone, Debug)]
pub struct DiplomacyView {
    pub bounds: Rect,
}

impl DiplomacyView {
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
            text: "Diplomatie internationale".into(),
            color: colors::TEXT_HIGHLIGHT,
            scale: 1.2,
        });
        y += 28.0;

        // Résumé
        let summary_text = format!(
            "Traités actifs : {} | Alliances/Blocs : {} | Sanctions : {} | Guerres : {}",
            model.diplomacy.active_treaties,
            model.diplomacy.organizations,
            model.diplomacy.sanctions,
            model.diplomacy.war_pairs
        );
        out.push(DrawCommand::Text {
            x: self.bounds.x + pad,
            y,
            text: summary_text,
            color: colors::TEXT_MUTED,
            scale: 1.0,
        });
        y += 24.0;

        // Ligne de séparation
        out.push(DrawCommand::Line {
            start: (self.bounds.x + pad, y),
            end: (self.bounds.x + self.bounds.width - pad, y),
            color: colors::PANEL_BORDER,
            thickness: 1.0,
        });
        y += 12.0;

        // Liste des nations et de leur statut
        for nation in &model.nations {
            if model.player_nation.as_ref() == Some(&nation.id) {
                continue;
            }

            let status_color = if nation.at_war {
                colors::STATUS_WAR
            } else {
                colors::STATUS_PEACE
            };

            out.push(DrawCommand::FillRect {
                rect: Rect::new(self.bounds.x + pad, y + 4.0, 8.0, 8.0),
                color: status_color,
            });

            out.push(DrawCommand::Text {
                x: self.bounds.x + pad + 16.0,
                y,
                text: nation.name.clone(),
                color: colors::TEXT_PRIMARY,
                scale: 1.0,
            });

            out.push(DrawCommand::Text {
                x: self.bounds.x + pad + 200.0,
                y,
                text: if nation.at_war {
                    "En guerre".into()
                } else {
                    "En paix".into()
                },
                color: colors::TEXT_PRIMARY,
                scale: 1.0,
            });

            y += 22.0;
            if y > self.bounds.y + self.bounds.height - 30.0 {
                break;
            }
        }

        y += 12.0;
        out.push(DrawCommand::Text {
            x: self.bounds.x + pad,
            y,
            text: "Demandes diplomatiques".into(),
            color: colors::TEXT_HIGHLIGHT,
            scale: 1.0,
        });
        y += 22.0;
        if model.diplomatic_requests.is_empty() {
            out.push(DrawCommand::Text {
                x: self.bounds.x + pad,
                y,
                text: "Aucune demande en attente.".into(),
                color: colors::TEXT_MUTED,
                scale: 0.9,
            });
        } else {
            for request in &model.diplomatic_requests {
                if y > self.bounds.y + self.bounds.height - 20.0 {
                    break;
                }
                out.push(DrawCommand::Text {
                    x: self.bounds.x + pad,
                    y,
                    text: format!(
                        "{} — {} -> {} [{}]",
                        request.title, request.from, request.to, request.status
                    ),
                    color: colors::TEXT_PRIMARY,
                    scale: 0.85,
                });
                y += 18.0;
            }
        }

        y += 12.0;
        out.push(DrawCommand::Text {
            x: self.bounds.x + pad,
            y,
            text: "Alliances".into(),
            color: colors::TEXT_HIGHLIGHT,
            scale: 1.0,
        });
        y += 22.0;
        if model.alliances.is_empty() {
            out.push(DrawCommand::Text {
                x: self.bounds.x + pad,
                y,
                text: "Aucune alliance enregistrée.".into(),
                color: colors::TEXT_MUTED,
                scale: 0.9,
            });
        } else {
            for alliance in &model.alliances {
                if y > self.bounds.y + self.bounds.height - 20.0 {
                    break;
                }
                out.push(DrawCommand::Text {
                    x: self.bounds.x + pad,
                    y,
                    text: format!(
                        "{} — {} membre(s), trésorerie {}, cohésion {:.0}",
                        alliance.name,
                        alliance.members.len(),
                        format_compact_number(alliance.treasury as u64),
                        alliance.cohesion
                    ),
                    color: colors::TEXT_PRIMARY,
                    scale: 0.85,
                });
                y += 18.0;
            }
        }
    }
}
