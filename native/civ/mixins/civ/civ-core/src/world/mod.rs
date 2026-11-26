//! Monde statique : carte, topologie, régions et détroits.

pub mod border;
pub mod map;
pub mod polygon;
pub mod province;
pub mod resources;
pub mod terrain;

pub use border::{Border, BorderKind};
pub use map::{Generation, GenerationAlgorithm, World, WorldSize};
pub use polygon::{Polygon, Polyline, Vec2, Vec3};
pub use province::{
    CultureShare, Province, ProvinceClimate, ProvinceCulture, ProvincePopulation, ProvinceReligion,
    ReligionShare,
};
pub use resources::ResourceDeposit;
pub use terrain::Terrain;

use serde::{Deserialize, Serialize};

use crate::ids::{ClimateZoneId, NationId, ProvinceId, RegionId, StraitId};
use crate::text::LocalizedText;

/// Intervalle numérique fermé.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}

impl Range {
    /// Vrai si la valeur appartient à l'intervalle.
    pub fn contains(self, value: f64) -> bool {
        value >= self.min && value <= self.max
    }
}

/// Type de région géographique.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RegionKind {
    Continent,
    Subcontinent,
    Island,
    Archipelago,
    Peninsula,
    Basin,
    Highland,
}

/// Région nommée regroupant des provinces.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Region {
    pub id: RegionId,
    pub name: LocalizedText,
    pub kind: RegionKind,
    pub provinces: Vec<ProvinceId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub climate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategic_note: Option<LocalizedText>,
}

/// Détroit ou passage maritime.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Strait {
    pub id: StraitId,
    pub name: LocalizedText,
    pub provinces: Vec<ProvinceId>,
    pub width_blocks: u32,
    pub navigable: bool,
    #[serde(default)]
    pub chokepoint: bool,
    pub blocked_by: Option<NationId>,
}

/// Zone climatique de référence.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClimateZone {
    pub id: ClimateZoneId,
    pub name: LocalizedText,
    pub temperature_range: Range,
    pub rainfall_range: Range,
    #[serde(default)]
    pub biomes: Vec<String>,
}

impl Region {
    /// Vrai si la région contient la province.
    pub fn contains(&self, province: &ProvinceId) -> bool {
        self.provinces.contains(province)
    }
}

impl Strait {
    /// Vrai si le détroit est utilisable par une flotte.
    pub fn is_open(&self) -> bool {
        self.navigable && self.blocked_by.is_none()
    }
}
