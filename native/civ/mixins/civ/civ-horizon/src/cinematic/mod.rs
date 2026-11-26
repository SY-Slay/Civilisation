//! Système cinématique (caméra, plans, directeur, letterbox).
//!
//! Ce module gère les angles de vue dramatiques lors des déclarations de guerre,
//! lancements de missiles ou traités majeurs, sans dépendance directe à OpenGL/Vulkan.

pub mod director;
pub mod overlay;
pub mod shots;

use crate::civ_core::math::Vec3;

/// Pose complète de la caméra dans l'espace 3D.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraPose {
    pub position: Vec3,
    pub yaw_deg: f32,
    pub pitch_deg: f32,
    pub roll_deg: f32,
    pub fov_deg: f32,
}

impl CameraPose {
    pub const fn new(position: Vec3, yaw_deg: f32, pitch_deg: f32, fov_deg: f32) -> Self {
        Self {
            position,
            yaw_deg,
            pitch_deg,
            roll_deg: 0.0,
            fov_deg,
        }
    }

    /// Interpolation linéaire entre deux poses.
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let t_clamped = t.clamp(0.0, 1.0);
        let pos = Vec3::new(
            self.position.x() + (other.position.x() - self.position.x()) * t_clamped as f64,
            self.position.y() + (other.position.y() - self.position.y()) * t_clamped as f64,
            self.position.z() + (other.position.z() - self.position.z()) * t_clamped as f64,
        );
        let yaw = self.yaw_deg + (other.yaw_deg - self.yaw_deg) * t_clamped;
        let pitch = self.pitch_deg + (other.pitch_deg - self.pitch_deg) * t_clamped;
        let roll = self.roll_deg + (other.roll_deg - self.roll_deg) * t_clamped;
        let fov = self.fov_deg + (other.fov_deg - self.fov_deg) * t_clamped;

        Self {
            position: pos,
            yaw_deg: yaw,
            pitch_deg: pitch,
            roll_deg: roll,
            fov_deg: fov,
        }
    }
}
