//! Traités et clauses diplomatiques.

use serde::{Deserialize, Serialize};

use crate::effects::Term;
use crate::ids::{NationId, TreatyId};
use crate::text::LocalizedText;
use crate::time::{GameDate, Tick};

/// Type de traité.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TreatyKind {
    Peace,
    Alliance,
    NonAggression,
    Trade,
    Defensive,
    Ceasefire,
    Vassalage,
    Independence,
    Demilitarization,
    Reparations,
    CustomsUnion,
    Research,
    NuclearBan,
}

/// Cycle de vie d'un traité.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TreatyStatus {
    Proposed,
    Signed,
    Active,
    Suspended,
    Violated,
    Expired,
    Cancelled,
}

/// Violation enregistrée d'un traité.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TreatyViolation {
    pub by: NationId,
    pub tick: Tick,
    pub description: LocalizedText,
}

/// Traité multilatéral ou bilatéral.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Treaty {
    pub id: TreatyId,
    pub name: LocalizedText,
    pub kind: TreatyKind,
    pub parties: Vec<NationId>,
    #[serde(default)]
    pub signatories: Vec<NationId>,
    pub terms: Vec<Term>,
    pub start_date: GameDate,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_date: Option<GameDate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_days: Option<u32>,
    pub status: TreatyStatus,
    #[serde(default)]
    pub secret: bool,
    #[serde(default)]
    pub guarantors: Vec<NationId>,
    #[serde(default)]
    pub violations: Vec<TreatyViolation>,
}

impl Treaty {
    /// Vrai si le traité est encore en vigueur.
    pub fn is_active(&self) -> bool {
        matches!(self.status, TreatyStatus::Active | TreatyStatus::Signed)
    }

    /// Vrai si toutes les parties sont des nations connues.
    pub fn has_party(&self, nation: &NationId) -> bool {
        self.parties.contains(nation)
    }
}
