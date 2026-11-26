//! Province : unité territoriale atomique du monde.

use serde::{Deserialize, Serialize};

use super::resources::ResourceDeposit;
use super::terrain::Terrain;
use crate::ids::{BorderId, CultureId, NationId, ProvinceId, ReligionId};
use crate::math::{Polygon, Vec2};
use crate::text::LocalizedText;

fn default_growth_rate() -> f64 {
    0.005
}

/// Climat local d'une province.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProvinceClimate {
    pub temperature_c: f64,
    pub rainfall_mm: f64,
    #[serde(default)]
    pub seasonality: f64,
}

/// Population d'une province.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProvincePopulation {
    pub total: u64,
    pub urban: u64,
    pub rural: u64,
    #[serde(default = "default_growth_rate")]
    pub growth_rate: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub density_per_km2: Option<f64>,
}

/// Part d'une culture ou d'une religion dans une province.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CultureShare {
    pub culture: CultureId,
    pub share: f64,
}

/// Part d'une religion dans une province.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReligionShare {
    pub religion: ReligionId,
    pub share: f64,
}

/// Composition culturelle d'une province.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProvinceCulture {
    pub primary: CultureId,
    #[serde(default)]
    pub minorities: Vec<CultureShare>,
}

/// Composition religieuse d'une province.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProvinceReligion {
    pub primary: ReligionId,
    #[serde(default)]
    pub minorities: Vec<ReligionShare>,
}

/// Province de la carte Civilisations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Province {
    pub id: ProvinceId,
    pub name: LocalizedText,
    pub polygon: Polygon,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub center: Option<Vec2>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_blocks: Option<u64>,
    #[serde(default)]
    pub neighbors: Vec<ProvinceId>,
    #[serde(default)]
    pub border_ids: Vec<BorderId>,
    pub terrain: Terrain,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub biome: Option<String>,
    pub climate: ProvinceClimate,
    pub population: ProvincePopulation,
    pub culture: ProvinceCulture,
    pub religion: ProvinceReligion,
    #[serde(default)]
    pub resources: Vec<ResourceDeposit>,
    pub owner: Option<NationId>,
    pub controller: Option<NationId>,
    #[serde(default)]
    pub claims: Vec<NationId>,
    pub capital_of: Option<NationId>,
    #[serde(default)]
    pub fortification: u8,
    pub unrest: f64,
    pub supply: f64,
    pub infrastructure: f64,
    pub strategic_value: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub victory_points: Option<f64>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Province {
    /// Vrai si la province est contrôlée par la nation indiquée.
    pub fn is_controlled_by(&self, nation: &NationId) -> bool {
        self.controller.as_ref() == Some(nation)
    }

    /// Vrai si la province est occupée par une nation différente du propriétaire.
    pub fn is_occupied(&self) -> bool {
        match (&self.owner, &self.controller) {
            (Some(owner), Some(controller)) => owner != controller,
            _ => false,
        }
    }

    /// Vrai si le terrain est littoral ou insulaire.
    pub fn is_coastal(&self) -> bool {
        matches!(self.terrain, Terrain::Coast | Terrain::Island)
    }
}

#[cfg(all(test, not(feature = "embedded-core")))]
mod tests {
    use super::*;
    use crate::math::Vec2;

    fn province(owner: &str, controller: &str) -> Province {
        Province {
            id: ProvinceId::new("prov_test").unwrap(),
            name: LocalizedText::plain("Test"),
            polygon: Polygon::new(vec![
                Vec2::new(0.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(1.0, 1.0),
            ])
            .unwrap(),
            center: None,
            area_blocks: None,
            neighbors: Vec::new(),
            border_ids: Vec::new(),
            terrain: Terrain::Plains,
            biome: None,
            climate: ProvinceClimate {
                temperature_c: 12.0,
                rainfall_mm: 700.0,
                seasonality: 0.0,
            },
            population: ProvincePopulation {
                total: 10,
                urban: 4,
                rural: 6,
                growth_rate: 0.005,
                density_per_km2: None,
            },
            culture: ProvinceCulture {
                primary: CultureId::new("ardennic").unwrap(),
                minorities: Vec::new(),
            },
            religion: ProvinceReligion {
                primary: ReligionId::new("rationalism").unwrap(),
                minorities: Vec::new(),
            },
            resources: Vec::new(),
            owner: Some(NationId::new(owner).unwrap()),
            controller: Some(NationId::new(controller).unwrap()),
            claims: Vec::new(),
            capital_of: None,
            fortification: 0,
            unrest: 0.0,
            supply: 100.0,
            infrastructure: 50.0,
            strategic_value: 0.0,
            victory_points: None,
            tags: Vec::new(),
        }
    }

    #[test]
    fn detecte_une_occupation() {
        assert!(!province("ardenne", "ardenne").is_occupied());
        assert!(province("ardenne", "kesh").is_occupied());
    }
}
