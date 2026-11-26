//! Données éditables du moteur : technologies, missiles, événements et registres.
//!
//! Les définitions sont chargées depuis des fichiers JSON validés. Elles ne
//! contiennent aucun code exécutable : le moteur interprète uniquement les
//! champs typés et les `kind` connus. Le format JSON est volontairement
//! compatible avec les schémas de `schemas/`.

pub mod loader;
pub mod registry;
pub mod validation;

use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::effects::{Effect, Modifier};
use crate::error::{CoreError, CoreResult};
use crate::event::EventDefinition;
use crate::ids::TechnologyId;
use crate::missile::MissileType;
use crate::text::LocalizedText;

pub use loader::load_data_pack;
pub use registry::DataPackRegistries;
pub use validation::validate_scenario_against_registries;

/// Catégorie d'une technologie.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TechnologyCategory {
    Economy,
    Industry,
    Military,
    Naval,
    Aerospace,
    Missile,
    Computing,
    Energy,
    Medicine,
    Society,
    Intelligence,
}

/// Éléments débloqués par une technologie.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TechnologyUnlocks {
    #[serde(default)]
    pub units: Vec<String>,
    #[serde(default)]
    pub missiles: Vec<String>,
    #[serde(default)]
    pub buildings: Vec<String>,
    #[serde(default)]
    pub events: Vec<String>,
    #[serde(default)]
    pub modifiers: Vec<Modifier>,
}

/// Définition data-driven d'une technologie.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TechnologyDefinition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<u32>,
    pub id: TechnologyId,
    pub name: LocalizedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
    pub era: u8,
    pub category: TechnologyCategory,
    pub cost: f64,
    pub research_time_days: u32,
    #[serde(default)]
    pub prerequisites: Vec<TechnologyId>,
    #[serde(default)]
    pub effects: Vec<Effect>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unlocks: Option<TechnologyUnlocks>,
    #[serde(default)]
    pub repeatable: bool,
    #[serde(default = "default_max_level")]
    pub max_level: u32,
    #[serde(default = "default_ai_weight")]
    pub ai_weight: f64,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_max_level() -> u32 {
    1
}

fn default_ai_weight() -> f64 {
    1.0
}

/// Ensemble de définitions chargé au démarrage d'une partie.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GameData {
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub technologies: Vec<TechnologyDefinition>,
    #[serde(default)]
    pub missiles: Vec<MissileType>,
    #[serde(default)]
    pub events: Vec<EventDefinition>,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            schema_version: 1,
            technologies: Vec::new(),
            missiles: Vec::new(),
            events: Vec::new(),
        }
    }
}

impl GameData {
    /// Charge un fichier JSON et valide les définitions.
    pub fn from_path(path: &Path) -> CoreResult<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json_str(&content)
    }

    /// Charge une chaîne JSON et valide les définitions.
    pub fn from_json_str(input: &str) -> CoreResult<Self> {
        let data: Self = serde_json::from_str(input)?;
        data.validate()?;
        Ok(data)
    }

    /// Vérifie les identifiants et les références internes.
    pub fn validate(&self) -> CoreResult<()> {
        if self.schema_version == 0 {
            return Err(CoreError::data(
                "données de jeu : schema_version doit être >= 1",
            ));
        }

        let mut technology_ids = BTreeSet::new();
        for technology in &self.technologies {
            if !technology_ids.insert(technology.id.clone()) {
                return Err(CoreError::data(format!(
                    "technologie dupliquée : {}",
                    technology.id
                )));
            }
            if !(1..=12).contains(&technology.era) {
                return Err(CoreError::data(format!(
                    "technologie {} : ère invalide",
                    technology.id
                )));
            }
            if technology.cost < 0.0 || technology.research_time_days == 0 {
                return Err(CoreError::data(format!(
                    "technologie {} : coût ou durée invalide",
                    technology.id
                )));
            }
            if technology.max_level == 0 {
                return Err(CoreError::data(format!(
                    "technologie {} : max_level doit être >= 1",
                    technology.id
                )));
            }
            for effect in &technology.effects {
                if !is_known_effect_kind(&effect.kind) {
                    return Err(CoreError::data(format!(
                        "technologie {} : effet inconnu {}",
                        technology.id, effect.kind
                    )));
                }
            }
        }
        for technology in &self.technologies {
            for prerequisite in &technology.prerequisites {
                if !technology_ids.contains(prerequisite) {
                    return Err(CoreError::data(format!(
                        "technologie {} : prérequis inconnu {}",
                        technology.id, prerequisite
                    )));
                }
                if prerequisite == &technology.id {
                    return Err(CoreError::data(format!(
                        "technologie {} : auto-prérequis interdit",
                        technology.id
                    )));
                }
            }
        }

        let mut missile_ids = BTreeSet::new();
        for missile in &self.missiles {
            if !missile_ids.insert(missile.id.clone()) {
                return Err(CoreError::data(format!(
                    "missile dupliqué : {}",
                    missile.id
                )));
            }
            if missile.range_km <= 0.0 || missile.speed_kmh <= 0.0 {
                return Err(CoreError::data(format!(
                    "missile {} : portée ou vitesse invalide",
                    missile.id
                )));
            }
            if !(0.0..=1.0).contains(&missile.defense_penetration) {
                return Err(CoreError::data(format!(
                    "missile {} : pénétration invalide",
                    missile.id
                )));
            }
            if !(1..=5).contains(&missile.danger_tier) {
                return Err(CoreError::data(format!(
                    "missile {} : niveau de danger doit etre compris entre 1 et 5",
                    missile.id
                )));
            }
        }

        let mut event_ids = BTreeSet::new();
        for event in &self.events {
            if !event_ids.insert(event.id.clone()) {
                return Err(CoreError::data(format!(
                    "événement dupliqué : {}",
                    event.id
                )));
            }
            if event.choices.is_empty() {
                return Err(CoreError::data(format!(
                    "événement {} : aucun choix",
                    event.id
                )));
            }
            for condition in &event.trigger.conditions {
                if !is_known_condition_kind(&condition.kind) {
                    return Err(CoreError::data(format!(
                        "événement {} : condition inconnue {}",
                        event.id, condition.kind
                    )));
                }
            }
            let mut choice_ids = BTreeSet::new();
            for choice in &event.choices {
                if !choice_ids.insert(choice.id.clone()) {
                    return Err(CoreError::data(format!(
                        "événement {} : choix dupliqué {}",
                        event.id, choice.id
                    )));
                }
                for condition in &choice.requirements {
                    if !is_known_condition_kind(&condition.kind) {
                        return Err(CoreError::data(format!(
                            "événement {} : condition de choix inconnue {}",
                            event.id, condition.kind
                        )));
                    }
                }
                for effect in &choice.effects {
                    if !is_known_effect_kind(&effect.kind) {
                        return Err(CoreError::data(format!(
                            "événement {} : effet inconnu {}",
                            event.id, effect.kind
                        )));
                    }
                }
            }
            if let Some(choice) = &event.ai_choice {
                if !choice_ids.contains(choice) {
                    return Err(CoreError::data(format!(
                        "événement {} : ai_choice inconnu {}",
                        event.id, choice
                    )));
                }
            }
            if event.weight < 0.0 || event.cooldown_days == 0 || event.max_fires == 0 {
                return Err(CoreError::data(format!(
                    "événement {} : paramètres invalides",
                    event.id
                )));
            }
        }

        Ok(())
    }

    /// Recherche une technologie par identifiant.
    pub fn technology(&self, id: &TechnologyId) -> Option<&TechnologyDefinition> {
        self.technologies.iter().find(|value| &value.id == id)
    }

    /// Recherche un type de missile par identifiant.
    pub fn missile(&self, id: &crate::ids::MissileTypeId) -> Option<&MissileType> {
        self.missiles.iter().find(|value| &value.id == id)
    }

    /// Recherche un événement par identifiant.
    pub fn event(&self, id: &crate::ids::EventId) -> Option<&EventDefinition> {
        self.events.iter().find(|value| &value.id == id)
    }
}

fn is_known_condition_kind(kind: &str) -> bool {
    matches!(
        kind,
        "stability_below"
            | "stability_above"
            | "unrest_above"
            | "legitimacy_below"
            | "war_exhaustion_above"
            | "gdp_below"
            | "treasury_below"
            | "at_war"
            | "has_revolution"
            | "sanctioned"
            | "research_at_least"
            | "relation_opinion_below"
            | "border_tension_above"
    )
}

fn is_known_effect_kind(kind: &str) -> bool {
    matches!(
        kind,
        "stability"
            | "legitimacy"
            | "unrest"
            | "war_exhaustion"
            | "prestige"
            | "treasury"
            | "gdp"
            | "manpower"
            | "army_size"
            | "research_points"
            | "relation_opinion"
            | "opinion"
            | "border_tension"
            | "start_revolution"
            | "province_unrest"
    )
}

#[cfg(all(test, not(feature = "embedded-core")))]
mod tests {
    use super::*;
    use crate::ids::MissileTypeId;
    use crate::missile::{MissileKind, Warhead, WarheadKind};

    fn sample() -> GameData {
        GameData {
            schema_version: 1,
            technologies: vec![TechnologyDefinition {
                schema_version: Some(1),
                id: TechnologyId::new("tech_admin").unwrap(),
                name: LocalizedText::plain("Administration"),
                description: None,
                era: 1,
                category: TechnologyCategory::Society,
                cost: 100.0,
                research_time_days: 360,
                prerequisites: Vec::new(),
                effects: Vec::new(),
                unlocks: None,
                repeatable: false,
                max_level: 1,
                ai_weight: 1.0,
                tags: Vec::new(),
            }],
            missiles: vec![MissileType {
                id: MissileTypeId::new("missile_short").unwrap(),
                name: LocalizedText::plain("Éclair"),
                description: None,
                danger_tier: 1,
                kind: MissileKind::RocketArtillery,
                range_km: 300.0,
                speed_kmh: 3000.0,
                flight_profile: Default::default(),
                accuracy_m: 50.0,
                warhead: Warhead {
                    kind: WarheadKind::Conventional,
                    yield_kt: 0.0,
                    blast_radius_blocks: 20.0,
                    effects: Vec::new(),
                    fallout: false,
                    escalation_delta: 0.0,
                },
                cost: 100.0,
                build_time_days: 30,
                max_stock: 10,
                defense_penetration: 0.2,
                interception: None,
                launch_platforms: Vec::new(),
                explosion_effects: None,
                tags: Vec::new(),
            }],
            events: Vec::new(),
        }
    }

    #[test]
    fn valide_les_definitions() {
        sample().validate().unwrap();
    }

    #[test]
    fn refuse_un_prerequis_inconnu() {
        let mut data = sample();
        data.technologies[0]
            .prerequisites
            .push(TechnologyId::new("tech_missing").unwrap());
        assert!(data.validate().is_err());
    }

    #[test]
    fn refuse_les_kinds_inconnus() {
        assert!(is_known_condition_kind("at_war"));
        assert!(!is_known_condition_kind("moon_phase"));
        assert!(is_known_effect_kind("stability"));
        assert!(!is_known_effect_kind("orbital_laser"));
    }
}
