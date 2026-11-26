//! Diplomatie : relations, traités, organisations, sanctions et influence.

pub mod espionage;
pub mod guarantee;
pub mod organization;
pub mod relation;
pub mod request;
pub mod sanction;
pub mod trade;
pub mod treaty;

pub use espionage::{EspionageKind, EspionageOperation, EspionageStatus};
pub use guarantee::Guarantee;
pub use organization::{Organization, OrganizationKind, OrganizationVote, VoteChoice};
pub use relation::{Relation, RelationStatus};
pub use request::{DiplomaticRequest, DiplomaticRequestKind, DiplomaticRequestStatus};
pub use sanction::{Sanction, SanctionKind, SanctionStatus};
pub use trade::{TradeRoute, TradeStatus};
pub use treaty::{Treaty, TreatyKind, TreatyStatus, TreatyViolation};

use serde::{Deserialize, Serialize};

use crate::ids::NationId;

/// État diplomatique complet d'une partie.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Diplomacy {
    pub schema_version: u32,
    #[serde(default)]
    pub relations: Vec<Relation>,
    #[serde(default)]
    pub treaties: Vec<Treaty>,
    #[serde(default)]
    pub organizations: Vec<Organization>,
    #[serde(default)]
    pub sanctions: Vec<Sanction>,
    #[serde(default)]
    pub guarantees: Vec<Guarantee>,
    #[serde(default)]
    pub espionage_operations: Vec<EspionageOperation>,
    #[serde(default)]
    pub trade_routes: Vec<TradeRoute>,
    #[serde(default)]
    pub requests: Vec<DiplomaticRequest>,
}

impl Diplomacy {
    /// Relation entre deux nations, quel que soit l'ordre.
    pub fn relation(&self, first: &NationId, second: &NationId) -> Option<&Relation> {
        self.relations
            .iter()
            .find(|relation| relation.involves(first) && relation.involves(second))
    }

    /// Relation mutable entre deux nations.
    pub fn relation_mut(&mut self, first: &NationId, second: &NationId) -> Option<&mut Relation> {
        self.relations
            .iter_mut()
            .find(|relation| relation.involves(first) && relation.involves(second))
    }

    /// Vrai si les deux nations sont formellement en guerre.
    pub fn at_war(&self, first: &NationId, second: &NationId) -> bool {
        self.relation(first, second)
            .map(|relation| relation.status == RelationStatus::AtWar)
            .unwrap_or(false)
    }

    /// Traités actifs impliquant une nation.
    pub fn active_treaties_for<'a>(
        &'a self,
        nation: &'a NationId,
    ) -> impl Iterator<Item = &'a Treaty> + 'a {
        self.treaties
            .iter()
            .filter(move |treaty| treaty.is_active() && treaty.has_party(nation))
    }
}
