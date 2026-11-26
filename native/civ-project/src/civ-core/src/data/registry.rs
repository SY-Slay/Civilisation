//! Registres de contenu typés pour les cultures, religions, idéologies, gouvernements, ressources et unités.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::effects::Modifier;
use crate::error::{CoreError, CoreResult};
use crate::ids::{CultureId, GovernmentId, IdeologyId, ReligionId, ResourceId, UnitId};
use crate::text::LocalizedText;

/// Définition d'une culture fictive.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CultureDefinition {
    #[serde(default)]
    pub schema_version: u32,
    pub id: CultureId,
    pub name: LocalizedText,
    pub adjective: LocalizedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
    pub language: String,
    pub family: String,
    #[serde(default)]
    pub values: Vec<String>,
    #[serde(default)]
    pub traditions: Vec<String>,
    pub name_pool: String,
    pub color: String,
    #[serde(default)]
    pub modifiers: Vec<Modifier>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Définition d'une religion ou philosophie fictive.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReligionDefinition {
    #[serde(default)]
    pub schema_version: u32,
    pub id: ReligionId,
    pub name: LocalizedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
    pub family: String,
    #[serde(default)]
    pub tenets: Vec<String>,
    pub tolerance: f64,
    #[serde(default)]
    pub holy_sites: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clergy_title: Option<String>,
    #[serde(default)]
    pub modifiers: Vec<Modifier>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Définition d'une idéologie fictive.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IdeologyDefinition {
    #[serde(default)]
    pub schema_version: u32,
    pub id: IdeologyId,
    pub name: LocalizedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
    pub family: String,
    #[serde(default)]
    pub values: Vec<String>,
    #[serde(default)]
    pub government_preferences: Vec<String>,
    #[serde(default)]
    pub modifiers: Vec<Modifier>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Définition d'une forme de gouvernement fictive.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GovernmentDefinition {
    #[serde(default)]
    pub schema_version: u32,
    pub id: GovernmentId,
    pub name: LocalizedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
    pub category: String,
    pub legitimacy_source: String,
    pub succession: String,
    pub term_length_days: u32,
    #[serde(default)]
    pub modifiers: Vec<Modifier>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Définition d'une ressource économique.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceDefinition {
    #[serde(default)]
    pub schema_version: u32,
    pub id: ResourceId,
    pub name: LocalizedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
    pub category: String,
    pub unit: String,
    pub base_value: f64,
    #[serde(default)]
    pub strategic: bool,
    #[serde(default)]
    pub modifiers: Vec<Modifier>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Définition d'un type d'unité militaire.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitDefinition {
    #[serde(default)]
    pub schema_version: u32,
    pub id: UnitId,
    pub name: LocalizedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
    pub category: String,
    pub era: u8,
    pub cost: f64,
    pub upkeep: f64,
    pub manpower: u64,
    pub attack: f64,
    pub defense: f64,
    pub mobility: f64,
    #[serde(default)]
    pub range: f64,
    pub morale: f64,
    pub training_time_days: u32,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default)]
    pub counters: Vec<String>,
    #[serde(default)]
    pub weak_against: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Ensemble complet des registres de contenu d'un data-pack.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DataPackRegistries {
    pub cultures: BTreeMap<CultureId, CultureDefinition>,
    pub religions: BTreeMap<ReligionId, ReligionDefinition>,
    pub ideologies: BTreeMap<IdeologyId, IdeologyDefinition>,
    pub governments: BTreeMap<GovernmentId, GovernmentDefinition>,
    pub resources: BTreeMap<ResourceId, ResourceDefinition>,
    pub units: BTreeMap<UnitId, UnitDefinition>,
}

impl DataPackRegistries {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_culture(&mut self, item: CultureDefinition) -> CoreResult<()> {
        if self.cultures.contains_key(&item.id) {
            return Err(CoreError::data(format!("Culture dupliquée : {}", item.id)));
        }
        self.cultures.insert(item.id.clone(), item);
        Ok(())
    }

    pub fn insert_religion(&mut self, item: ReligionDefinition) -> CoreResult<()> {
        if self.religions.contains_key(&item.id) {
            return Err(CoreError::data(format!("Religion dupliquée : {}", item.id)));
        }
        self.religions.insert(item.id.clone(), item);
        Ok(())
    }

    pub fn insert_ideology(&mut self, item: IdeologyDefinition) -> CoreResult<()> {
        if self.ideologies.contains_key(&item.id) {
            return Err(CoreError::data(format!(
                "Idéologie dupliquée : {}",
                item.id
            )));
        }
        self.ideologies.insert(item.id.clone(), item);
        Ok(())
    }

    pub fn insert_government(&mut self, item: GovernmentDefinition) -> CoreResult<()> {
        if self.governments.contains_key(&item.id) {
            return Err(CoreError::data(format!(
                "Gouvernement dupliqué : {}",
                item.id
            )));
        }
        self.governments.insert(item.id.clone(), item);
        Ok(())
    }

    pub fn insert_resource(&mut self, item: ResourceDefinition) -> CoreResult<()> {
        if self.resources.contains_key(&item.id) {
            return Err(CoreError::data(format!(
                "Ressource dupliquée : {}",
                item.id
            )));
        }
        self.resources.insert(item.id.clone(), item);
        Ok(())
    }

    pub fn insert_unit(&mut self, item: UnitDefinition) -> CoreResult<()> {
        if self.units.contains_key(&item.id) {
            return Err(CoreError::data(format!("Unité dupliquée : {}", item.id)));
        }
        self.units.insert(item.id.clone(), item);
        Ok(())
    }
}
