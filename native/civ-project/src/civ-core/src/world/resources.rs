//! Gisements de ressources d'une province.

use serde::{Deserialize, Serialize};

use crate::ids::ResourceId;

/// Gisement ou stock naturel dans une province.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResourceDeposit {
    pub resource: ResourceId,
    pub abundance: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserve: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extraction_rate: Option<f64>,
}
