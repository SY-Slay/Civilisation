//! Générateur pseudo-aléatoire déterministe.
//!
//! L'algorithme `splitmix64_counter_v1` est un SplitMix64 à compteur :
//! l'état complet tient dans `(seed, counter, stream)`. Deux exécutions avec
//! les mêmes valeurs produisent exactement la même séquence, sans dépendre de
//! l'ordre des appels système ni d'une horloge.

use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};

/// Identifiant de l'algorithme enregistré dans les sauvegardes.
pub const RNG_ALGORITHM: &str = "splitmix64_counter_v1";

const GOLDEN_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;
const MIX_A: u64 = 0xBF58_476D_1CE4_E5B9;
const MIX_B: u64 = 0x94D0_49BB_1331_11EB;
const STREAM_GAMMA: u64 = 0x9E37_79B9_7F4A_7C15;

/// État sérialisable du générateur.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RngState {
    pub algorithm: String,
    pub seed: u64,
    pub counter: u64,
    pub stream: u32,
}

impl RngState {
    /// Construit un état pour l'algorithme courant.
    pub fn new(seed: u64, stream: u32, counter: u64) -> Self {
        Self {
            algorithm: RNG_ALGORITHM.to_owned(),
            seed,
            counter,
            stream,
        }
    }
}

/// Générateur déterministe à compteur.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeterministicRng {
    seed: u64,
    counter: u64,
    stream: u32,
}

impl DeterministicRng {
    /// Crée un générateur à partir d'une graine et d'un flux.
    pub fn new(seed: u64, stream: u32) -> Self {
        Self {
            seed,
            counter: 0,
            stream,
        }
    }

    /// Restaure un générateur depuis une sauvegarde.
    pub fn from_state(state: &RngState) -> CoreResult<Self> {
        if state.algorithm != RNG_ALGORITHM {
            return Err(CoreError::UnsupportedRng(state.algorithm.clone()));
        }
        if state.seed > i64::MAX as u64 {
            return Err(CoreError::data("graine RNG hors bornes"));
        }
        Ok(Self {
            seed: state.seed,
            counter: state.counter,
            stream: state.stream,
        })
    }

    /// Exporte l'état complet.
    pub fn state(&self) -> RngState {
        RngState::new(self.seed, self.stream, self.counter)
    }

    /// Graine initiale.
    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Flux initial.
    pub fn stream(&self) -> u32 {
        self.stream
    }

    /// Compteur courant, nombre d'appels déjà effectués.
    pub fn counter(&self) -> u64 {
        self.counter
    }

    /// Produit le prochain entier 64 bits.
    pub fn next_u64(&mut self) -> u64 {
        let stream = u64::from(self.stream).wrapping_mul(STREAM_GAMMA);
        let mut value = self
            .seed
            .wrapping_add(stream)
            .wrapping_add(self.counter.wrapping_mul(GOLDEN_GAMMA));
        self.counter = self.counter.wrapping_add(1);
        value = (value ^ (value >> 30)).wrapping_mul(MIX_A);
        value = (value ^ (value >> 27)).wrapping_mul(MIX_B);
        value ^ (value >> 31)
    }

    /// Produit le prochain entier 32 bits.
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Produit un booléen équilibré.
    pub fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1) == 1
    }

    /// Produit un flottant dans `[0, 1)`.
    pub fn unit_f64(&mut self) -> f64 {
        const SCALE: f64 = 1.0 / ((1u64 << 53) as f64);
        ((self.next_u64() >> 11) as f64) * SCALE
    }

    /// Réussit avec la probabilité `probability`, bornée à `[0, 1]`.
    pub fn chance(&mut self, probability: f64) -> bool {
        if probability <= 0.0 {
            return false;
        }
        if probability >= 1.0 {
            return true;
        }
        self.unit_f64() < probability
    }

    /// Entier uniforme dans `[min, max)` ; retourne `min` si l'intervalle est vide.
    pub fn range_u64(&mut self, min: u64, max: u64) -> u64 {
        if max <= min {
            return min;
        }
        min.wrapping_add(self.next_u64() % (max - min))
    }

    /// Entier signé uniforme dans `[min, max)`.
    pub fn range_i64(&mut self, min: i64, max: i64) -> i64 {
        if max <= min {
            return min;
        }
        let span = (i128::from(max) - i128::from(min)) as u64;
        (i128::from(min) + i128::from(self.next_u64() % span)) as i64
    }

    /// Choisit un élément dans une tranche non vide.
    pub fn choose<'a, T>(&mut self, values: &'a [T]) -> Option<&'a T> {
        if values.is_empty() {
            return None;
        }
        let index = self.range_u64(0, values.len() as u64) as usize;
        values.get(index)
    }

    /// Mélange une tranche avec l'algorithme de Fisher-Yates.
    pub fn shuffle<T>(&mut self, values: &mut [T]) {
        if values.len() < 2 {
            return;
        }
        for index in (1..values.len()).rev() {
            let other = self.range_u64(0, (index + 1) as u64) as usize;
            values.swap(index, other);
        }
    }
}

#[cfg(all(test, not(feature = "embedded-core")))]
mod tests {
    use super::*;

    #[test]
    fn meme_graine_meme_sequence() {
        let mut a = DeterministicRng::new(12345, 7);
        let mut b = DeterministicRng::new(12345, 7);
        for _ in 0..256 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn flux_different_sequence_differente() {
        let mut a = DeterministicRng::new(12345, 1);
        let mut b = DeterministicRng::new(12345, 2);
        assert_ne!(a.next_u64(), b.next_u64());
    }

    #[test]
    fn reprise_depuis_etat() {
        let mut original = DeterministicRng::new(99, 3);
        for _ in 0..10 {
            original.next_u64();
        }
        let state = original.state();
        let mut restored = DeterministicRng::from_state(&state).unwrap();
        assert_eq!(state.counter, 10);
        for _ in 0..32 {
            assert_eq!(original.next_u64(), restored.next_u64());
        }
    }

    #[test]
    fn bornes_des_intervalles() {
        let mut rng = DeterministicRng::new(1, 0);
        for _ in 0..1000 {
            let value = rng.range_u64(10, 20);
            assert!((10..20).contains(&value));
            let signed = rng.range_i64(-5, 5);
            assert!((-5..5).contains(&signed));
            let unit = rng.unit_f64();
            assert!((0.0..1.0).contains(&unit));
        }
    }

    #[test]
    fn refuse_un_algorithme_inconnu() {
        let state = RngState {
            algorithm: "linear_congruential_v0".to_owned(),
            seed: 1,
            counter: 0,
            stream: 0,
        };
        assert!(DeterministicRng::from_state(&state).is_err());
    }
}
