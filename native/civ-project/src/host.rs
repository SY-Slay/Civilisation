//! Frontière entre la simulation et Horizon.
//!
//! Ce trait est volontairement exempt de types Minecraft. L'adaptateur réel
//! (module `glue.rs`) l'implémente avec les API Horizon vérifiées ; les tests
//! utilisent `RecordingHost`.

use crate::civ_core::math::Vec3;

/// Capacités minimales offertes par l'hôte.
pub trait Host {
    /// Message d'information affiché au joueur.
    fn info(&mut self, message: &str);
    /// Avertissement non bloquant.
    fn warn(&mut self, message: &str);
    /// Erreur visible.
    fn error(&mut self, message: &str);
    /// Effet de particules à une position du monde.
    fn spawn_particle(&mut self, particle: &str, position: Vec3);
    /// Son à une position du monde.
    fn play_sound(&mut self, sound: &str, position: Vec3);
    /// Secousse de caméra.
    fn camera_shake(&mut self, intensity: f32, duration_ticks: u32);
    /// Envoi d'un message réseau aux clients.
    fn send_net(&mut self, channel: &str, payload: &[u8]);
    /// Demande d'écriture d'une sauvegarde par la couche Horizon.
    fn request_save(&mut self, save_name: &str, json: &str);
}

/// Hôte de test qui enregistre les appels.
#[derive(Clone, Debug, Default)]
pub struct RecordingHost {
    pub messages: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub particles: Vec<(String, Vec3)>,
    pub sounds: Vec<(String, Vec3)>,
    pub shakes: Vec<(f32, u32)>,
    pub network: Vec<(String, Vec<u8>)>,
    pub saves: Vec<(String, String)>,
}

impl Host for RecordingHost {
    fn info(&mut self, message: &str) {
        self.messages.push(message.to_owned());
    }

    fn warn(&mut self, message: &str) {
        self.warnings.push(message.to_owned());
    }

    fn error(&mut self, message: &str) {
        self.errors.push(message.to_owned());
    }

    fn spawn_particle(&mut self, particle: &str, position: Vec3) {
        self.particles.push((particle.to_owned(), position));
    }

    fn play_sound(&mut self, sound: &str, position: Vec3) {
        self.sounds.push((sound.to_owned(), position));
    }

    fn camera_shake(&mut self, intensity: f32, duration_ticks: u32) {
        self.shakes.push((intensity, duration_ticks));
    }

    fn send_net(&mut self, channel: &str, payload: &[u8]) {
        self.network.push((channel.to_owned(), payload.to_vec()));
    }

    fn request_save(&mut self, save_name: &str, json: &str) {
        self.saves.push((save_name.to_owned(), json.to_owned()));
    }
}
