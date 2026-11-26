//! Définition et calcul des plans de caméra cinématiques.

use super::CameraPose;
use crate::civ_core::math::Vec3;

/// Type de mouvement de caméra.
#[derive(Clone, Debug, PartialEq)]
pub enum ShotKind {
    /// Plan fixe sur un point avec orientation constante.
    Stationary { pose: CameraPose },
    /// Travelling linéaire d'une pose A à une pose B.
    Flyover { start: CameraPose, end: CameraPose },
    /// Orbite circulaire autour d'un point cible.
    Orbit {
        center: Vec3,
        radius: f32,
        height: f32,
        start_angle_deg: f32,
        end_angle_deg: f32,
        pitch_deg: f32,
        fov_deg: f32,
    },
}

/// Un plan cinématique avec sa durée et son lissage.
#[derive(Clone, Debug, PartialEq)]
pub struct CinematicShot {
    pub kind: ShotKind,
    pub duration_seconds: f32,
    pub title: Option<String>,
    pub subtitle: Option<String>,
}

impl CinematicShot {
    pub fn stationary(pose: CameraPose, duration_seconds: f32) -> Self {
        Self {
            kind: ShotKind::Stationary { pose },
            duration_seconds,
            title: None,
            subtitle: None,
        }
    }

    pub fn flyover(start: CameraPose, end: CameraPose, duration_seconds: f32) -> Self {
        Self {
            kind: ShotKind::Flyover { start, end },
            duration_seconds,
            title: None,
            subtitle: None,
        }
    }

    pub fn orbit(center: Vec3, radius: f32, height: f32, duration_seconds: f32) -> Self {
        Self {
            kind: ShotKind::Orbit {
                center,
                radius,
                height,
                start_angle_deg: 0.0,
                end_angle_deg: 90.0,
                pitch_deg: 25.0,
                fov_deg: 70.0,
            },
            duration_seconds,
            title: None,
            subtitle: None,
        }
    }

    pub fn with_caption(mut self, title: impl Into<String>, subtitle: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Calcule la pose de la caméra au temps t (secondes).
    pub fn evaluate(&self, elapsed_seconds: f32) -> CameraPose {
        let t_norm = if self.duration_seconds <= 0.0 {
            1.0
        } else {
            (elapsed_seconds / self.duration_seconds).clamp(0.0, 1.0)
        };

        // Lissage smoothstep
        let smooth_t = t_norm * t_norm * (3.0 - 2.0 * t_norm);

        match &self.kind {
            ShotKind::Stationary { pose } => *pose,
            ShotKind::Flyover { start, end } => start.lerp(end, smooth_t),
            ShotKind::Orbit {
                center,
                radius,
                height,
                start_angle_deg,
                end_angle_deg,
                pitch_deg,
                fov_deg,
            } => {
                let angle_deg = start_angle_deg + (end_angle_deg - start_angle_deg) * smooth_t;
                let angle_rad = angle_deg.to_radians();
                let x = center.x() + (*radius as f64) * (angle_rad as f64).cos();
                let z = center.z() + (*radius as f64) * (angle_rad as f64).sin();
                let y = center.y() + *height as f64;

                let yaw = angle_deg + 180.0; // Regarde vers le centre
                CameraPose::new(Vec3::new(x, y, z), yaw, *pitch_deg, *fov_deg)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shot_evaluation() {
        let p1 = CameraPose::new(Vec3::new(0.0, 10.0, 0.0), 0.0, 0.0, 70.0);
        let p2 = CameraPose::new(Vec3::new(100.0, 20.0, 0.0), 90.0, 10.0, 70.0);
        let shot = CinematicShot::flyover(p1, p2, 10.0);

        let start = shot.evaluate(0.0);
        assert_eq!(start.position.x(), 0.0);

        let end = shot.evaluate(10.0);
        assert_eq!(end.position.x(), 100.0);

        let mid = shot.evaluate(5.0);
        assert!((mid.position.x() - 50.0).abs() < 1e-3);
    }
}
