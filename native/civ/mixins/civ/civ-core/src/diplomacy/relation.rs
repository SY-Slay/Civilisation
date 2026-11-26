//! Relations bilatérales.

use serde::{Deserialize, Serialize};

use crate::effects::Modifier;
use crate::ids::NationId;
use crate::time::Tick;

/// Statut diplomatique d'une paire de nations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationStatus {
    Neutral,
    Friendly,
    Allied,
    Hostile,
    AtWar,
    Truce,
    Vassal,
    Overlord,
    Sanctioned,
}

/// Relation stockée une seule fois par paire triée.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Relation {
    pub a: NationId,
    pub b: NationId,
    #[serde(default)]
    pub opinion: f64,
    pub trust: f64,
    pub fear: f64,
    pub trade_dependence: f64,
    pub border_tension: f64,
    pub status: RelationStatus,
    #[serde(default)]
    pub embassy: bool,
    #[serde(default = "default_contact")]
    pub contact: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_change_tick: Option<Tick>,
    #[serde(default)]
    pub modifiers: Vec<Modifier>,
}

fn default_contact() -> bool {
    true
}

impl Relation {
    /// Vrai si les identifiants sont triés dans l'ordre canonique.
    pub fn is_canonical(&self) -> bool {
        self.a < self.b
    }

    /// Vrai si la relation concerne la nation.
    pub fn involves(&self, nation: &NationId) -> bool {
        &self.a == nation || &self.b == nation
    }

    /// Retourne l'autre nation.
    pub fn other(&self, nation: &NationId) -> Option<&NationId> {
        if &self.a == nation {
            Some(&self.b)
        } else if &self.b == nation {
            Some(&self.a)
        } else {
            None
        }
    }
}
