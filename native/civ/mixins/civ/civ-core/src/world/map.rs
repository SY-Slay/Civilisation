//! Carte statique et index de lecture du monde.

use serde::{Deserialize, Serialize};

use super::{Border, ClimateZone, Province, Region, Strait};
use crate::ids::{BorderId, NationId, ProvinceId, RegionId, StraitId};

/// Dimensions de la carte en blocs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSize {
    pub width: u32,
    pub height: u32,
}

/// Algorithme de génération de carte.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GenerationAlgorithm {
    #[default]
    NaturalRegionsV1,
    HandAuthoredV1,
}

/// Paramètres de génération conservés pour la reproductibilité.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Generation {
    #[serde(default)]
    pub algorithm: GenerationAlgorithm,
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_border_noise")]
    pub border_noise: f64,
    #[serde(default = "default_coast_complexity")]
    pub coast_complexity: f64,
}

fn default_version() -> u32 {
    1
}

fn default_border_noise() -> f64 {
    0.35
}

fn default_coast_complexity() -> f64 {
    0.55
}

/// Monde statique chargé depuis un scénario ou une sauvegarde.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct World {
    pub schema_version: u32,
    pub seed: u64,
    pub size: WorldSize,
    pub sea_level: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<Generation>,
    pub provinces: Vec<Province>,
    pub borders: Vec<Border>,
    pub regions: Vec<Region>,
    pub straits: Vec<Strait>,
    #[serde(default)]
    pub climate_zones: Vec<ClimateZone>,
}

impl World {
    /// Valide les invariants du monde.
    pub fn validate(&self) -> crate::error::CoreResult<()> {
        crate::validation::validate_world(self)
    }

    /// Retourne une province par identifiant.
    pub fn province(&self, id: &ProvinceId) -> Option<&Province> {
        self.provinces.iter().find(|province| &province.id == id)
    }

    /// Retourne une province mutable par identifiant.
    pub fn province_mut(&mut self, id: &ProvinceId) -> Option<&mut Province> {
        self.provinces
            .iter_mut()
            .find(|province| &province.id == id)
    }

    /// Retourne une frontière par identifiant.
    pub fn border(&self, id: &BorderId) -> Option<&Border> {
        self.borders.iter().find(|border| &border.id == id)
    }

    /// Retourne une région par identifiant.
    pub fn region(&self, id: &RegionId) -> Option<&Region> {
        self.regions.iter().find(|region| &region.id == id)
    }

    /// Retourne un détroit par identifiant.
    pub fn strait(&self, id: &StraitId) -> Option<&Strait> {
        self.straits.iter().find(|strait| &strait.id == id)
    }

    /// Voisins déclarés d'une province.
    pub fn neighbors(&self, id: &ProvinceId) -> Option<&[ProvinceId]> {
        self.province(id)
            .map(|province| province.neighbors.as_slice())
    }

    /// Vrai si deux provinces sont adjacentes.
    pub fn are_adjacent(&self, first: &ProvinceId, second: &ProvinceId) -> bool {
        self.province(first)
            .map(|province| province.neighbors.contains(second))
            .unwrap_or(false)
    }

    /// Vrai si la province est contrôlée par la nation.
    pub fn is_controlled_by(&self, province: &ProvinceId, nation: &NationId) -> bool {
        self.province(province)
            .map(|value| value.is_controlled_by(nation))
            .unwrap_or(false)
    }

    /// Nombre de provinces.
    pub fn province_count(&self) -> usize {
        self.provinces.len()
    }

    /// Population totale de toutes les provinces.
    pub fn total_population(&self) -> u64 {
        self.provinces
            .iter()
            .map(|province| province.population.total)
            .sum()
    }

    /// Provinces dont le propriétaire est la nation.
    pub fn provinces_owned_by<'a>(
        &'a self,
        nation: &'a NationId,
    ) -> impl Iterator<Item = &'a Province> + 'a {
        self.provinces
            .iter()
            .filter(move |province| province.owner.as_ref() == Some(nation))
    }
}

#[cfg(all(test, not(feature = "embedded-core")))]
mod tests {
    use super::*;

    #[test]
    fn generation_par_defaut() {
        let generation = Generation {
            algorithm: GenerationAlgorithm::NaturalRegionsV1,
            version: 1,
            border_noise: 0.35,
            coast_complexity: 0.55,
        };
        assert_eq!(generation.version, 1);
    }
}
