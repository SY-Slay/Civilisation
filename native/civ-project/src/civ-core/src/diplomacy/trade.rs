//! Routes commerciales.

use serde::{Deserialize, Serialize};

use crate::ids::{NationId, ResourceId, TradeRouteId};

/// État d'une route commerciale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeStatus {
    Open,
    Restricted,
    Blockaded,
    Closed,
}

/// Flux commercial entre deux nations.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TradeRoute {
    pub id: TradeRouteId,
    pub a: NationId,
    pub b: NationId,
    pub resource: ResourceId,
    pub volume: f64,
    pub status: TradeStatus,
    pub blockaded_by: Option<NationId>,
}

impl TradeRoute {
    /// Vrai si le flux peut circuler.
    pub fn is_open(&self) -> bool {
        self.status == TradeStatus::Open && self.blockaded_by.is_none()
    }

    /// Vrai si la nation participe au flux.
    pub fn involves(&self, nation: &NationId) -> bool {
        &self.a == nation || &self.b == nation
    }
}
