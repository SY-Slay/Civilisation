//! Forces armées et doctrine militaire.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ids::MissileTypeId;

/// Doctrine militaire principale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Doctrine {
    Defensive,
    Mobile,
    Attrition,
    Blitz,
    Naval,
    Aerospace,
    Asymmetric,
    Conscription,
}

/// Ordre de bataille agrégé d'une nation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Military {
    pub manpower: u64,
    pub army_size: u64,
    #[serde(default)]
    pub navy_size: u64,
    #[serde(default)]
    pub airforce_size: u64,
    pub readiness: f64,
    pub morale: f64,
    pub doctrine: Doctrine,
    #[serde(default)]
    pub nuclear_capable: bool,
    #[serde(default)]
    pub missile_stockpile: BTreeMap<MissileTypeId, u64>,
}

impl Military {
    /// Nombre total de plateformes navales et aériennes.
    pub fn total_platforms(&self) -> u64 {
        self.navy_size.saturating_add(self.airforce_size)
    }

    /// Nombre de missiles d'un type donné.
    pub fn missile_count(&self, missile_type: &MissileTypeId) -> u64 {
        self.missile_stockpile
            .get(missile_type)
            .copied()
            .unwrap_or(0)
    }
}
