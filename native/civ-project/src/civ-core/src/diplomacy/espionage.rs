//! Opérations d'influence et d'espionnage.

use serde::{Deserialize, Serialize};

use crate::effects::Effect;
use crate::ids::{EspionageId, NationId};
use crate::time::GameDate;

/// Type d'opération.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EspionageKind {
    Intel,
    Sabotage,
    Coup,
    Propaganda,
    Assassination,
    Cyber,
    FundRebels,
}

/// État d'avancement d'une opération.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EspionageStatus {
    Preparing,
    Active,
    Succeeded,
    Failed,
    Exposed,
    Cancelled,
}

/// Opération secrète suivie par le moteur.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EspionageOperation {
    pub id: EspionageId,
    pub actor: NationId,
    pub target: NationId,
    pub kind: EspionageKind,
    pub status: EspionageStatus,
    pub progress: f64,
    pub exposure: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_date: Option<GameDate>,
    #[serde(default)]
    pub effects: Vec<Effect>,
}

impl EspionageOperation {
    /// Vrai si l'opération est encore menée.
    pub fn is_running(&self) -> bool {
        matches!(
            self.status,
            EspionageStatus::Preparing | EspionageStatus::Active
        )
    }
}
