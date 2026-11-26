//! Garanties d'indépendance.

use serde::{Deserialize, Serialize};

use crate::effects::Condition;
use crate::ids::{GuaranteeId, NationId};
use crate::time::GameDate;

/// Garantie accordée par une nation à une autre.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Guarantee {
    pub id: GuaranteeId,
    pub guarantor: NationId,
    pub protected: NationId,
    pub start_date: GameDate,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_date: Option<GameDate>,
    pub active: bool,
    #[serde(default)]
    pub conditions: Vec<Condition>,
}

impl Guarantee {
    /// Vrai si la garantie protège la nation.
    pub fn protects(&self, nation: &NationId) -> bool {
        self.active && &self.protected == nation
    }
}
