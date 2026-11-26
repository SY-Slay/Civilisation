//! Terrain de province et ses effets de base.

use serde::{Deserialize, Serialize};

/// Terrain principal d'une province.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Terrain {
    Plains,
    Hills,
    Mountains,
    Forest,
    Rainforest,
    Desert,
    Steppe,
    Tundra,
    Swamp,
    Coast,
    Island,
    Urban,
    Ruins,
}

impl Terrain {
    /// Coût de déplacement relatif, avant infrastructures.
    pub const fn movement_cost(self) -> f64 {
        match self {
            Self::Plains => 1.0,
            Self::Hills => 1.4,
            Self::Mountains => 2.2,
            Self::Forest => 1.5,
            Self::Rainforest => 2.0,
            Self::Desert => 1.6,
            Self::Steppe => 1.1,
            Self::Tundra => 1.8,
            Self::Swamp => 2.4,
            Self::Coast => 1.0,
            Self::Island => 1.2,
            Self::Urban => 0.9,
            Self::Ruins => 1.6,
        }
    }

    /// Bonus défensif additif en pourcentage.
    pub const fn defense_bonus(self) -> f64 {
        match self {
            Self::Mountains => 35.0,
            Self::Hills => 18.0,
            Self::Forest => 15.0,
            Self::Rainforest => 20.0,
            Self::Swamp => 22.0,
            Self::Urban => 30.0,
            Self::Ruins => 25.0,
            Self::Tundra => 8.0,
            _ => 0.0,
        }
    }

    /// Multiplicateur d'approvisionnement.
    pub const fn supply_modifier(self) -> f64 {
        match self {
            Self::Desert => 0.65,
            Self::Tundra => 0.75,
            Self::Mountains => 0.7,
            Self::Swamp => 0.8,
            Self::Plains => 1.1,
            Self::Urban => 1.2,
            _ => 1.0,
        }
    }
}

#[cfg(all(test, not(feature = "embedded-core")))]
mod tests {
    use super::*;

    #[test]
    fn les_montagnes_freinent_et_protegent() {
        assert!(Terrain::Mountains.movement_cost() > Terrain::Plains.movement_cost());
        assert!(Terrain::Mountains.defense_bonus() > Terrain::Plains.defense_bonus());
    }
}
