//! Génération de la géométrie des frontières de provinces.

use super::{LineStrip3D, RenderBatch3D};
use crate::civ_core::math::Vec3;
use crate::civ_core::world::Province;
use crate::ui::theme::{colors, ColorRgba};

/// Paramètres de génération des frontières.
#[derive(Clone, Copy, Debug)]
pub struct BorderConfig {
    pub y_elevation: f64,
    pub thickness: f32,
    pub occupied_alpha: f32,
}

impl Default for BorderConfig {
    fn default() -> Self {
        Self {
            y_elevation: 64.5,
            thickness: 2.0,
            occupied_alpha: 0.6,
        }
    }
}

/// Génère les bandes de lignes des frontières pour une liste de provinces.
pub fn build_borders_batch<'a, F>(
    provinces: impl IntoIterator<Item = &'a Province>,
    config: BorderConfig,
    mut nation_color: F,
    out: &mut RenderBatch3D,
) where
    F: FnMut(&crate::civ_core::ids::NationId) -> Option<ColorRgba>,
{
    for prov in provinces {
        let owner_col = match &prov.owner {
            Some(id) => nation_color(id).unwrap_or(colors::MAP_UNCLAIMED),
            None => colors::MAP_UNCLAIMED,
        };

        // Si la province est occupée par une autre nation, on modifie la teinte
        let color = if prov.controller.is_some() && prov.controller != prov.owner {
            ColorRgba::new(
                (owner_col.r + colors::STATUS_WAR.r) * 0.5,
                (owner_col.g + colors::STATUS_WAR.g) * 0.5,
                (owner_col.b + colors::STATUS_WAR.b) * 0.5,
                config.occupied_alpha,
            )
        } else {
            owner_col
        };

        let points = prov.polygon.points();
        if points.len() < 3 {
            continue;
        }

        let mut strip = LineStrip3D::new(config.thickness, true);
        for pt in points {
            let pos = Vec3::new(pt.x(), config.y_elevation, pt.z());
            strip.push(pos, color);
        }

        out.push_strip(strip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civ_core::scenario::Scenario;
    use std::path::Path;

    #[test]
    fn build_border_strips_from_scenario() {
        let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("civ-core")
            .join("tests")
            .join("fixtures")
            .join("scenario.json");

        if let Ok(scen) = Scenario::from_path(&fixture_path) {
            let mut batch = RenderBatch3D::new();
            build_borders_batch(
                &scen.world.provinces,
                BorderConfig::default(),
                |_| Some(ColorRgba::new(0.0, 1.0, 0.0, 1.0)),
                &mut batch,
            );

            assert!(!batch.lines.is_empty());
            for strip in &batch.lines {
                assert!(strip.vertices.len() >= 3);
                assert!(strip.looped);
            }
        }
    }
}
