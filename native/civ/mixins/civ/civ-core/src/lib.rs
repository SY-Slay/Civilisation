//! Civilisations — cœur de simulation pur et déterministe.
//!
//! Cette crate ne dépend ni de Minecraft, ni d'Horizon, ni d'un réseau. Toutes
//! les données sont sérialisables et toutes les décisions du moteur utilisent
//! le RNG à compteur documenté.

#![forbid(unsafe_code)]

pub mod data;
pub mod diplomacy;
pub mod effects;
pub mod error;
pub mod event;
pub mod ids;
pub mod math;
pub mod missile;
pub mod nation;
pub mod revolution;
pub mod rng;
pub mod save;
pub mod scenario;
pub mod sim;
pub mod text;
pub mod time;
pub mod validation;
pub mod war;
pub mod world;

pub use error::{CoreError, CoreResult};
pub use save::Save;
pub use scenario::{Difficulty, Scenario};
