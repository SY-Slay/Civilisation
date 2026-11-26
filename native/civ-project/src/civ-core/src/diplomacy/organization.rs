//! Organisations internationales et votes.

use serde::{Deserialize, Serialize};

use crate::effects::Term;
use crate::ids::{NationId, OrganizationId};
use crate::text::LocalizedText;
use crate::time::GameDate;

/// Type d'organisation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationKind {
    Alliance,
    TradeBloc,
    CustomsUnion,
    DefensePact,
    Federation,
    Council,
    Cartel,
    ResearchNetwork,
}

/// Vote d'une nation dans une organisation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoteChoice {
    Yes,
    No,
    Abstain,
}

/// Voix pondérée.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrganizationVote {
    pub nation: NationId,
    pub weight: f64,
    pub vote: VoteChoice,
}

/// Organisation internationale.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Organization {
    pub id: OrganizationId,
    pub name: LocalizedText,
    pub kind: OrganizationKind,
    pub members: Vec<NationId>,
    pub founding_date: GameDate,
    #[serde(default)]
    pub rules: Vec<Term>,
    pub leader: Option<NationId>,
    pub cohesion: f64,
    #[serde(default)]
    pub votes: Vec<OrganizationVote>,
    #[serde(default = "default_active")]
    pub active: bool,
    /// Treasury used by a pact/alliance; 0.0 is a valid neutral default.
    #[serde(default)]
    pub treasury: f64,
    /// Optional stable policy terms (governance, economy and diplomacy).
    #[serde(default)]
    pub economy_terms: Vec<Term>,
    #[serde(default)]
    pub diplomacy_terms: Vec<Term>,
}

fn default_active() -> bool {
    true
}

impl Organization {
    /// Vrai si la nation est membre.
    pub fn has_member(&self, nation: &NationId) -> bool {
        self.members.contains(nation)
    }

    /// Poids total des votes exprimés.
    pub fn total_vote_weight(&self) -> f64 {
        self.votes.iter().map(|vote| vote.weight).sum()
    }
}
