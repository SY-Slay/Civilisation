//! Économie d'une nation.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ids::{NationId, ResourceId};
use crate::time::GameDate;

fn default_tax_rate() -> f64 {
    0.25
}

fn default_inflation() -> f64 {
    0.03
}

fn default_unemployment() -> f64 {
    0.07
}

/// Répartition sectorielle du PIB.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EconomySectors {
    pub agriculture: f64,
    pub industry: f64,
    pub services: f64,
    pub technology: f64,
}

/// Sanction subie par une nation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SanctionRef {
    pub by: NationId,
    pub severity: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<GameDate>,
}

/// Économie agrégée d'une nation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Economy {
    pub gdp: f64,
    pub treasury: f64,
    /// National purse used for weekly/discretionary country spending.
    #[serde(default)]
    pub purse: f64,
    #[serde(default = "default_tax_rate")]
    pub tax_rate: f64,
    #[serde(default = "default_inflation")]
    pub inflation: f64,
    #[serde(default = "default_unemployment")]
    pub unemployment: f64,
    #[serde(default)]
    pub trade_balance: f64,
    #[serde(default)]
    pub debt: f64,
    pub sectors: EconomySectors,
    #[serde(default)]
    pub resources: BTreeMap<ResourceId, f64>,
    #[serde(default)]
    pub sanctions: Vec<SanctionRef>,
}

impl Economy {
    /// Stock disponible pour une ressource.
    pub fn stockpile(&self, resource: &ResourceId) -> f64 {
        self.resources.get(resource).copied().unwrap_or(0.0)
    }

    /// Revenu fiscal annuel estimé avant corruption et exemptions.
    pub fn estimated_tax_income(&self) -> f64 {
        self.gdp * self.tax_rate
    }

    /// Poids total des secteurs, utile pour normaliser les données.
    pub fn sector_total(&self) -> f64 {
        self.sectors.agriculture
            + self.sectors.industry
            + self.sectors.services
            + self.sectors.technology
    }
}
