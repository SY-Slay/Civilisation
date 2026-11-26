//! Sanctions économiques et diplomatiques.

use serde::{Deserialize, Serialize};

use crate::ids::{NationId, SanctionId};
use crate::time::GameDate;

/// Domaine visé par une sanction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SanctionKind {
    Trade,
    Financial,
    Technology,
    Arms,
    Travel,
    Energy,
    Cultural,
}

/// Cycle de vie d'une sanction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SanctionStatus {
    Proposed,
    Active,
    Suspended,
    Lifted,
    Expired,
}

/// Sanction appliquée par une coalition à une cible.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Sanction {
    pub id: SanctionId,
    pub by: Vec<NationId>,
    pub target: NationId,
    pub kind: SanctionKind,
    pub severity: f64,
    pub start_date: GameDate,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_date: Option<GameDate>,
    pub status: SanctionStatus,
    #[serde(default)]
    pub exemptions: Vec<NationId>,
}

impl Sanction {
    /// Vrai si la sanction est actuellement appliquée.
    pub fn is_active(&self) -> bool {
        self.status == SanctionStatus::Active
    }

    /// Vrai si la nation cible est concernée.
    pub fn targets(&self, nation: &NationId) -> bool {
        &self.target == nation
    }
}
