//! Chargeur de pack de données depuis le système de fichiers.

use serde::{Deserialize, Serialize};
use std::path::Path;

use super::registry::{
    CultureDefinition, DataPackRegistries, GovernmentDefinition, IdeologyDefinition,
    ReligionDefinition, ResourceDefinition, UnitDefinition,
};
use super::validation::validate_scenario_against_registries;
use super::GameData;
use crate::error::{CoreError, CoreResult};
use crate::scenario::Scenario;
use crate::text::LocalizedText;

/// Manifeste du pack de données.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataPackManifest {
    pub schema_version: u32,
    pub id: String,
    pub version: String,
    pub game: String,
    pub horizon: String,
    pub name: LocalizedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
    pub content: DataPackContent,
}

/// Chemins de contenu déclarés dans le manifeste.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataPackContent {
    pub config: String,
    pub scenarios: Vec<String>,
    pub nations: Vec<String>,
    pub provinces: Vec<String>,
    pub cultures: Vec<String>,
    pub religions: Vec<String>,
    pub ideologies: Vec<String>,
    pub governments: Vec<String>,
    pub resources: Vec<String>,
    pub technologies: Vec<String>,
    pub units: Vec<String>,
    pub missiles: Vec<String>,
    pub events: Vec<String>,
    pub names: Vec<String>,
    pub localization: Vec<String>,
}

/// Charge un data-pack complet à partir de son répertoire racine.
pub fn load_data_pack(
    root: &Path,
) -> CoreResult<(DataPackManifest, DataPackRegistries, Scenario, GameData)> {
    let manifest_path = root.join("manifest.json");
    if !manifest_path.exists() {
        return Err(CoreError::data(format!(
            "manifest.json introuvable dans {}",
            root.display()
        )));
    }

    let manifest_str = std::fs::read_to_string(&manifest_path)?;
    let manifest: DataPackManifest = serde_json::from_str(&manifest_str)?;

    let mut registries = DataPackRegistries::new();

    // 1. Cultures
    for rel in &manifest.content.cultures {
        let path = root.join(rel);
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            if let Ok(list) = serde_json::from_str::<Vec<CultureDefinition>>(&content) {
                for item in list {
                    registries.insert_culture(item)?;
                }
            } else if let Ok(single) = serde_json::from_str::<CultureDefinition>(&content) {
                registries.insert_culture(single)?;
            }
        }
    }

    // 2. Religions
    for rel in &manifest.content.religions {
        let path = root.join(rel);
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            if let Ok(list) = serde_json::from_str::<Vec<ReligionDefinition>>(&content) {
                for item in list {
                    registries.insert_religion(item)?;
                }
            } else if let Ok(single) = serde_json::from_str::<ReligionDefinition>(&content) {
                registries.insert_religion(single)?;
            }
        }
    }

    // 3. Idéologies
    for rel in &manifest.content.ideologies {
        let path = root.join(rel);
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            if let Ok(list) = serde_json::from_str::<Vec<IdeologyDefinition>>(&content) {
                for item in list {
                    registries.insert_ideology(item)?;
                }
            } else if let Ok(single) = serde_json::from_str::<IdeologyDefinition>(&content) {
                registries.insert_ideology(single)?;
            }
        }
    }

    // 4. Gouvernements
    for rel in &manifest.content.governments {
        let path = root.join(rel);
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            if let Ok(list) = serde_json::from_str::<Vec<GovernmentDefinition>>(&content) {
                for item in list {
                    registries.insert_government(item)?;
                }
            } else if let Ok(single) = serde_json::from_str::<GovernmentDefinition>(&content) {
                registries.insert_government(single)?;
            }
        }
    }

    // 5. Ressources
    for rel in &manifest.content.resources {
        let path = root.join(rel);
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            if let Ok(list) = serde_json::from_str::<Vec<ResourceDefinition>>(&content) {
                for item in list {
                    registries.insert_resource(item)?;
                }
            } else if let Ok(single) = serde_json::from_str::<ResourceDefinition>(&content) {
                registries.insert_resource(single)?;
            }
        }
    }

    // 6. Unités
    for rel in &manifest.content.units {
        let path = root.join(rel);
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            if let Ok(list) = serde_json::from_str::<Vec<UnitDefinition>>(&content) {
                for item in list {
                    registries.insert_unit(item)?;
                }
            } else if let Ok(single) = serde_json::from_str::<UnitDefinition>(&content) {
                registries.insert_unit(single)?;
            }
        }
    }

    // 7. Technologies, Missiles, Événements -> GameData
    let mut game_data = GameData::default();
    for rel in &manifest.content.technologies {
        let path = root.join(rel);
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            if let Ok(techs) = serde_json::from_str::<Vec<super::TechnologyDefinition>>(&content) {
                game_data.technologies.extend(techs);
            } else if let Ok(single) = serde_json::from_str::<super::TechnologyDefinition>(&content)
            {
                game_data.technologies.push(single);
            }
        }
    }
    for rel in &manifest.content.missiles {
        let path = root.join(rel);
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            if let Ok(missiles) = serde_json::from_str::<Vec<crate::missile::MissileType>>(&content)
            {
                game_data.missiles.extend(missiles);
            } else if let Ok(single) = serde_json::from_str::<crate::missile::MissileType>(&content)
            {
                game_data.missiles.push(single);
            }
        }
    }
    for rel in &manifest.content.events {
        let path = root.join(rel);
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            if let Ok(events) = serde_json::from_str::<Vec<crate::event::EventDefinition>>(&content)
            {
                game_data.events.extend(events);
            } else if let Ok(single) =
                serde_json::from_str::<crate::event::EventDefinition>(&content)
            {
                game_data.events.push(single);
            }
        }
    }

    // 8. Scénario par défaut
    let scenario_rel = manifest
        .content
        .scenarios
        .first()
        .ok_or_else(|| CoreError::data("Aucun scénario déclaré dans le pack"))?;
    let scenario_path = root.join(scenario_rel);
    let scenario_str = std::fs::read_to_string(&scenario_path)?;
    let scenario: Scenario = serde_json::from_str(&scenario_str)?;

    // Validation croisée
    validate_scenario_against_registries(&scenario, &registries, &game_data)?;

    Ok((manifest, registries, scenario, game_data))
}
