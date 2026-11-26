//! Requêtes diplomatiques proposées, acceptées ou refusées.

use serde::{Deserialize, Serialize};

use crate::effects::Term;
use crate::ids::{DiplomaticRequestId, NationId};
use crate::text::LocalizedText;
use crate::time::{GameDate, Tick};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiplomaticRequestKind {
    Alliance,
    Peace,
    NonAggression,
    Trade,
    DefensivePact,
    Ceasefire,
    JoinAlliance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiplomaticRequestStatus {
    Proposed,
    Accepted,
    Rejected,
    Cancelled,
    Expired,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DiplomaticRequest {
    pub id: DiplomaticRequestId,
    pub kind: DiplomaticRequestKind,
    pub from: NationId,
    pub to: NationId,
    pub title: LocalizedText,
    #[serde(default)]
    pub terms: Vec<Term>,
    pub created_date: GameDate,
    pub created_tick: Tick,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_tick: Option<Tick>,
    pub status: DiplomaticRequestStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_tick: Option<Tick>,
}
