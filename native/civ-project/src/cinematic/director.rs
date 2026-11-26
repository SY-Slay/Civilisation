//! Directeur cinématique orchestrant les séquences et la caméra.

use std::collections::VecDeque;

use super::overlay::Letterbox;
use super::shots::CinematicShot;
use super::CameraPose;
use crate::civ_core::math::Vec3;
use crate::ui::widgets::DrawCommand;

/// Distance maximale de caméra autorisée par rapport au joueur pour éviter les chunks vides.
pub const MAX_CAMERA_DISTANCE_FROM_PLAYER: f64 = 160.0;

/// Directeur de séquences de caméra.
#[derive(Clone, Debug, Default)]
pub struct CinematicDirector {
    queue: VecDeque<CinematicShot>,
    current: Option<CinematicShot>,
    elapsed_in_current: f32,
    player_anchor: Option<Vec3>,
}

impl CinematicDirector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Définit la position du joueur pour borner les déplacements de caméra.
    pub fn set_player_anchor(&mut self, anchor: Vec3) {
        self.player_anchor = Some(anchor);
    }

    /// Lance une nouvelle séquence cinématique (interrompt la précédente).
    pub fn play(&mut self, shots: impl IntoIterator<Item = CinematicShot>) {
        self.queue = shots.into_iter().collect();
        self.advance_to_next();
    }

    /// Arrête immédiatement la cinématique en cours.
    pub fn stop(&mut self) {
        self.queue.clear();
        self.current = None;
        self.elapsed_in_current = 0.0;
    }

    pub fn is_active(&self) -> bool {
        self.current.is_some()
    }

    /// Avance le temps de la séquence (appelé par client_tick ou frame).
    pub fn advance(&mut self, dt_seconds: f32) {
        if let Some(shot) = &self.current {
            self.elapsed_in_current += dt_seconds;
            if self.elapsed_in_current >= shot.duration_seconds {
                self.advance_to_next();
            }
        }
    }

    fn advance_to_next(&mut self) {
        self.current = self.queue.pop_front();
        self.elapsed_in_current = 0.0;
    }

    /// Calcule la pose actuelle de la caméra si une cinématique est en cours.
    pub fn current_camera_pose(&self) -> Option<CameraPose> {
        let shot = self.current.as_ref()?;
        let mut pose = shot.evaluate(self.elapsed_in_current);

        // Clamping par rapport à la position du joueur pour respecter la contrainte des chunks
        if let Some(anchor) = self.player_anchor {
            let dx = pose.position.x() - anchor.x();
            let dz = pose.position.z() - anchor.z();
            let dist_horiz = (dx * dx + dz * dz).sqrt();
            if dist_horiz > MAX_CAMERA_DISTANCE_FROM_PLAYER {
                let factor = MAX_CAMERA_DISTANCE_FROM_PLAYER / dist_horiz;
                pose.position = Vec3::new(
                    anchor.x() + dx * factor,
                    pose.position.y(),
                    anchor.z() + dz * factor,
                );
            }
        }

        Some(pose)
    }

    /// Doit-on masquer la main du joueur ?
    pub fn should_hide_hand(&self) -> bool {
        self.is_active()
    }

    /// Rendu de l'overlay cinématique (letterbox + sous-titres).
    pub fn render_overlay(&self, screen_w: f32, screen_h: f32, out: &mut Vec<DrawCommand>) {
        if !self.is_active() {
            return;
        }

        let lb = Letterbox::for_screen(screen_w, screen_h);
        lb.render_bars(out);

        if let Some(shot) = &self.current {
            lb.render_captions(shot.title.as_deref(), shot.subtitle.as_deref(), out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn director_sequence_progression() {
        let mut dir = CinematicDirector::new();
        assert!(!dir.is_active());

        let p1 = CameraPose::new(Vec3::new(0.0, 10.0, 0.0), 0.0, 0.0, 70.0);
        let s1 = CinematicShot::stationary(p1, 2.0);
        let s2 = CinematicShot::stationary(p1, 3.0);

        dir.play(vec![s1, s2]);
        assert!(dir.is_active());

        // Avance de 1s (dans s1)
        dir.advance(1.0);
        assert!(dir.is_active());

        // Avance de 1.5s supplémentaires (dépasse s1 (2s) et passe à s2)
        dir.advance(1.5);
        assert!(dir.is_active());

        // Avance de 3s supplémentaires (dépasse s2 (3s) -> fin)
        dir.advance(3.0);
        assert!(!dir.is_active());
        assert_eq!(dir.current_camera_pose(), None);
    }

    #[test]
    fn clamps_to_player_anchor() {
        let mut dir = CinematicDirector::new();
        dir.set_player_anchor(Vec3::new(0.0, 64.0, 0.0));

        let far_pose = CameraPose::new(Vec3::new(500.0, 64.0, 0.0), 0.0, 0.0, 70.0);
        dir.play(vec![CinematicShot::stationary(far_pose, 5.0)]);

        let pose = dir.current_camera_pose().unwrap();
        // Doit être clampé à MAX_CAMERA_DISTANCE_FROM_PLAYER (160)
        assert!((pose.position.x() - 160.0).abs() < 1e-3);
    }
}
