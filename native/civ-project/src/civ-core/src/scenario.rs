//! Scénario initial du jeu.
//!
//! Le scénario est l'état de départ complet. Il rassemble le monde, les nations,
//! la diplomatie et les éventuelles guerres/revolutions en cours. Le cœur ne
//! contient aucune logique d'initialisation procédurale : le fichier JSON est
//! chargé tel quel et validé.

use serde::{Deserialize, Serialize};

use crate::diplomacy::Diplomacy;
use crate::ids::ScenarioId;
use crate::missile::MissileInstance;
use crate::nation::Nation;
use crate::text::LocalizedText;
use crate::time::GameDate;
use crate::war::War;
use crate::world::World;

/// Niveau de difficulté choisi par le joueur.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Difficulty {
    Peaceful,
    Normal,
    Aggressive,
    Nightmare,
}

/// Scénario complet.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    pub schema_version: u32,
    pub id: ScenarioId,
    pub name: LocalizedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
    pub start_date: GameDate,
    pub seed: u64,
    pub world: World,
    pub nations: Vec<Nation>,
    pub diplomacy: Diplomacy,
    #[serde(default)]
    pub wars: Vec<War>,
    #[serde(default)]
    pub revolutions: Vec<crate::revolution::Revolution>,
    #[serde(default)]
    pub armies: Vec<crate::war::Army>,
    #[serde(default)]
    pub missiles_in_flight: Vec<MissileInstance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub player_nation: Option<crate::ids::NationId>,
    pub difficulty: Difficulty,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Scenario {
    /// Charge un scénario depuis un fichier JSON.
    pub fn from_path(path: &std::path::Path) -> crate::error::CoreResult<Self> {
        let data = std::fs::read_to_string(path)?;
        let scenario: Self = serde_json::from_str(&data)?;
        scenario.validate()?;
        Ok(scenario)
    }

    /// Validation complète du scénario.
    pub fn validate(&self) -> crate::error::CoreResult<()> {
        crate::validation::validate_scenario(self)
    }
}

#[cfg(all(test, not(feature = "embedded-core")))]
mod tests {
    use super::*;
    use crate::ids::NationId;

    #[test]
    fn charge_un_fichier_de_fixtures() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("scenario.json");
        let scenario = Scenario::from_path(&path).expect("lecture du scénario");
        assert_eq!(scenario.world.province_count(), 8);
        // La nation Ardenne doit être présente.
        let ardenne = scenario
            .nations
            .iter()
            .find(|n| n.id == NationId::new("ardenne").unwrap())
            .expect("nation ardenne");
        assert!(!ardenne.provinces.is_empty());
    }
}
