//! Rendu géométrique dans le monde (frontières, marqueurs, missiles).
//!
//! Ce module produit des primitives 3D prêtes à être envoyées à la passe
//! `native_world_drawn` de Horizon.

pub mod borders;
pub mod markers;
pub mod missiles;

use crate::civ_core::math::Vec3;
use crate::ui::theme::ColorRgba;

/// Sommet 3D coloré pour le rendu des lignes et polygones dans le monde.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex3D {
    pub position: Vec3,
    pub color: ColorRgba,
}

impl Vertex3D {
    pub const fn new(position: Vec3, color: ColorRgba) -> Self {
        Self { position, color }
    }
}

/// Bande de lignes 3D connectées.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LineStrip3D {
    pub vertices: Vec<Vertex3D>,
    pub thickness: f32,
    pub looped: bool,
}

impl LineStrip3D {
    pub fn new(thickness: f32, looped: bool) -> Self {
        Self {
            vertices: Vec::new(),
            thickness,
            looped,
        }
    }

    pub fn push(&mut self, pos: Vec3, color: ColorRgba) {
        self.vertices.push(Vertex3D::new(pos, color));
    }
}

/// Lot de géométrie 3D compilé pour le rendu d'une frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderBatch3D {
    pub lines: Vec<LineStrip3D>,
}

impl RenderBatch3D {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn push_strip(&mut self, strip: LineStrip3D) {
        if strip.vertices.len() >= 2 {
            self.lines.push(strip);
        }
    }
}
