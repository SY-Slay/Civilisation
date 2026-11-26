//! Conditions, effets, modificateurs et clauses de traités.
//!
//! Aucun effet JSON n'est exécuté directement : le cœur interprète uniquement
//! des `kind` connus, avec des arguments typés ou contrôlés par le validateur.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ids::NationId;
use crate::time::Tick;

/// Condition interprétable par le moteur déterministe.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Value>,
}

/// Effet différé ou immédiat interprété par le moteur.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Effect {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Value>,
    #[serde(default)]
    pub delay_ticks: u64,
}

/// Mode d'application d'un modificateur.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModifierMode {
    #[default]
    Add,
    Multiply,
    Set,
}

/// Modificateur numérique ciblant un chemin de donnée.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Modifier {
    pub target: String,
    pub value: f64,
    #[serde(default)]
    pub mode: ModifierMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<Condition>,
}

/// Clause générique d'un traité, d'une organisation ou d'une paix.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Term {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Value>,
}

/// Effet appliqué au tick indiqué, après résolution déterministe.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScheduledEffect {
    pub apply_tick: Tick,
    /// Nation cible mémorisée au moment de la planification.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nation: Option<NationId>,
    pub effect: Effect,
}

impl Modifier {
    /// Applique le modificateur à une valeur.
    pub fn apply(&self, current: f64) -> f64 {
        match self.mode {
            ModifierMode::Add => current + self.value,
            ModifierMode::Multiply => current * self.value,
            ModifierMode::Set => self.value,
        }
    }
}

#[cfg(all(test, not(feature = "embedded-core")))]
mod tests {
    use super::*;

    #[test]
    fn applique_les_trois_modes() {
        let add = Modifier {
            target: "military.morale".to_owned(),
            value: 5.0,
            mode: ModifierMode::Add,
            condition: None,
        };
        assert_eq!(add.apply(50.0), 55.0);
        let multiply = Modifier {
            mode: ModifierMode::Multiply,
            ..add.clone()
        };
        assert_eq!(multiply.apply(50.0), 250.0);
        let set = Modifier {
            mode: ModifierMode::Set,
            ..add
        };
        assert_eq!(set.apply(50.0), 5.0);
    }
}
