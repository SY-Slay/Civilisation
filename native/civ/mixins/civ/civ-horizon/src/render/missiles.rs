//! Rendu des trajectoires paraboliques de missiles balistiques.

use super::{LineStrip3D, RenderBatch3D};
use crate::civ_core::math::Vec3;
use crate::ui::theme::{colors, ColorRgba};

/// Trajectoire balistique calculée.
#[derive(Clone, Debug, PartialEq)]
pub struct BallisticTrajectory {
    pub origin: Vec3,
    pub target: Vec3,
    pub apex_altitude: f64,
    pub launch_tick: u64,
    pub arrival_tick: u64,
}

impl BallisticTrajectory {
    pub fn new(
        origin: Vec3,
        target: Vec3,
        apex_altitude: f64,
        launch_tick: u64,
        arrival_tick: u64,
    ) -> Self {
        Self {
            origin,
            target,
            apex_altitude,
            launch_tick,
            arrival_tick,
        }
    }

    /// Avancement normalisé [0.0..1.0] pour un tick donné.
    pub fn progress_at(&self, current_tick: u64) -> f64 {
        if current_tick <= self.launch_tick {
            0.0
        } else if current_tick >= self.arrival_tick {
            1.0
        } else {
            let total = (self.arrival_tick - self.launch_tick) as f64;
            (current_tick - self.launch_tick) as f64 / total.max(1.0)
        }
    }

    /// Position 3D à un avancement t in [0.0..1.0].
    pub fn position_at_progress(&self, t: f64) -> Vec3 {
        let t_clamped = t.clamp(0.0, 1.0);
        let x = self.origin.x() + t_clamped * (self.target.x() - self.origin.x());
        let z = self.origin.z() + t_clamped * (self.target.z() - self.origin.z());

        // Parabole : h(t) = 4 * apex * t * (1 - t)
        let base_y = self.origin.y() + t_clamped * (self.target.y() - self.origin.y());
        let arc_y = 4.0 * self.apex_altitude * t_clamped * (1.0 - t_clamped);
        Vec3::new(x, base_y + arc_y, z)
    }

    /// Position 3D au tick actuel.
    pub fn position_at_tick(&self, tick: u64) -> Vec3 {
        self.position_at_progress(self.progress_at(tick))
    }

    /// Génère les lignes de la traînée de vol pour le lot de rendu.
    pub fn append_trail_geometry(
        &self,
        current_tick: u64,
        segments: usize,
        out: &mut RenderBatch3D,
    ) {
        let max_t = self.progress_at(current_tick);
        if max_t <= 0.0 {
            return;
        }

        let mut strip = LineStrip3D::new(2.5, false);
        let step = max_t / (segments.max(2) as f64);

        for i in 0..=segments {
            let t = (i as f64 * step).min(max_t);
            let pos = self.position_at_progress(t);
            // La tête du missile est plus brillante que la queue
            let alpha = (t / max_t) as f32;
            let col = ColorRgba::new(
                colors::STATUS_TENSION.r,
                colors::STATUS_TENSION.g,
                colors::STATUS_TENSION.b,
                alpha.clamp(0.1, 1.0),
            );
            strip.push(pos, col);
        }

        out.push_strip(strip);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trajectory_interpolation() {
        let traj = BallisticTrajectory::new(
            Vec3::new(0.0, 60.0, 0.0),
            Vec3::new(100.0, 60.0, 0.0),
            50.0, // 50m d'apogée
            100,
            200,
        );

        assert_eq!(traj.progress_at(100), 0.0);
        assert_eq!(traj.progress_at(150), 0.5);
        assert_eq!(traj.progress_at(200), 1.0);

        let mid = traj.position_at_tick(150);
        assert_eq!(mid.x(), 50.0);
        // À mi-parcours : base_y (60) + 4 * 50 * 0.5 * 0.5 = 60 + 50 = 110
        assert_eq!(mid.y(), 110.0);
        assert_eq!(mid.z(), 0.0);
    }
}
