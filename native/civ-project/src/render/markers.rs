//! Marqueurs 3D dans le monde pour les capitales, fronts et points d'intérêt.

use super::{LineStrip3D, RenderBatch3D};
use crate::civ_core::math::Vec3;
use crate::ui::theme::{colors, ColorRgba};

/// Type de marqueur affiché dans le monde.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MarkerKind {
    Capital,
    FrontBattle,
    MissileImpact,
}

/// Marqueur positionné dans l'espace.
#[derive(Clone, Debug)]
pub struct WorldMarker {
    pub position: Vec3,
    pub kind: MarkerKind,
    pub color: ColorRgba,
    pub height: f64,
}

impl WorldMarker {
    pub fn capital(position: Vec3, color: ColorRgba) -> Self {
        Self {
            position,
            kind: MarkerKind::Capital,
            color,
            height: 8.0,
        }
    }

    pub fn battle(position: Vec3) -> Self {
        Self {
            position,
            kind: MarkerKind::FrontBattle,
            color: colors::STATUS_WAR,
            height: 6.0,
        }
    }

    /// Génère un faisceau vertical stylisé pour le marqueur.
    pub fn append_geometry(&self, out: &mut RenderBatch3D) {
        let mut strip = LineStrip3D::new(3.0, false);
        strip.push(self.position, self.color);
        strip.push(
            Vec3::new(
                self.position.x(),
                self.position.y() + self.height,
                self.position.z(),
            ),
            ColorRgba::new(self.color.r, self.color.g, self.color.b, 0.0), // Fondu vers le haut
        );
        out.push_strip(strip);
    }
}
