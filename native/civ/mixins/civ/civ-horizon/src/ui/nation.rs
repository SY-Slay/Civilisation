//! Vue détaillée d'une nation (gouvernement, économie, population).

use super::theme::{colors, format_compact_number, format_percent, ColorRgba};
use super::widgets::{DrawCommand, Rect};
use super::{NationSummary, UiModel};
use crate::civ_core::ids::NationId;

/// État de la vue de nation.
#[derive(Clone, Debug)]
pub struct NationView {
    pub bounds: Rect,
    pub selected_nation: Option<NationId>,
}

impl NationView {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            selected_nation: None,
        }
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

        let target_id = self
            .selected_nation
            .as_ref()
            .or(model.player_nation.as_ref())
            .or_else(|| model.nations.first().map(|n| &n.id));

        let nation = match target_id.and_then(|id| model.nations.iter().find(|n| &n.id == id)) {
            Some(n) => n,
            None => {
                out.push(DrawCommand::Text {
                    x: self.bounds.x + 20.0,
                    y: self.bounds.y + 20.0,
                    text: "Aucune nation sélectionnée.".into(),
                    color: colors::TEXT_MUTED,
                    scale: 1.0,
                });
                return;
            }
        };

        render_nation_details(nation, self.bounds, out);
    }
}

fn render_nation_details(nation: &NationSummary, bounds: Rect, out: &mut Vec<DrawCommand>) {
    let pad = 16.0;
    let mut y = bounds.y + pad;

    // Titre et couleur
    let col = ColorRgba::from_hex(&nation.color).unwrap_or(colors::ACCENT);
    out.push(DrawCommand::FillRect {
        rect: Rect::new(bounds.x + pad, y, 16.0, 16.0),
        color: col,
    });
    out.push(DrawCommand::Text {
        x: bounds.x + pad + 24.0,
        y: y - 2.0,
        text: format!("{} ({})", nation.name, nation.short_name),
        color: colors::TEXT_HIGHLIGHT,
        scale: 1.2,
    });
    y += 32.0;

    // Ligne de séparation
    out.push(DrawCommand::Line {
        start: (bounds.x + pad, y),
        end: (bounds.x + bounds.width - pad, y),
        color: colors::PANEL_BORDER,
        thickness: 1.0,
    });
    y += 12.0;

    // Métriques
    let rows = [
        ("Capitale", nation.capital.to_string()),
        ("Provinces", nation.provinces.to_string()),
        ("Population", format_compact_number(nation.population)),
        ("Stabilité", format_percent(nation.stability)),
        ("Légitimité", format_percent(nation.legitimacy)),
        ("Trésorerie", format_compact_number(nation.treasury as u64)),
        ("Purse", format_compact_number(nation.purse as u64)),
        ("PIB", format_compact_number(nation.gdp as u64)),
        (
            "Classement",
            nation
                .weekly_rank
                .map(|rank| format!("#{}", rank))
                .unwrap_or_else(|| "Non classé".into()),
        ),
        (
            "État",
            if nation.at_war {
                "EN GUERRE".into()
            } else {
                "En paix".into()
            },
        ),
    ];

    for (label, val) in rows {
        out.push(DrawCommand::Text {
            x: bounds.x + pad,
            y,
            text: format!("{}:", label),
            color: colors::TEXT_MUTED,
            scale: 1.0,
        });
        out.push(DrawCommand::Text {
            x: bounds.x + pad + 140.0,
            y,
            text: val,
            color: colors::TEXT_PRIMARY,
            scale: 1.0,
        });
        y += 20.0;
    }

    if !nation.research_domains.is_empty() {
        y += 8.0;
        out.push(DrawCommand::Text {
            x: bounds.x + pad,
            y,
            text: "Domaines de recherche".into(),
            color: colors::TEXT_HIGHLIGHT,
            scale: 1.0,
        });
        y += 20.0;
        for domain in &nation.research_domains {
            out.push(DrawCommand::Text {
                x: bounds.x + pad,
                y,
                text: domain.clone(),
                color: colors::TEXT_PRIMARY,
                scale: 0.85,
            });
            y += 18.0;
        }
    }
}
