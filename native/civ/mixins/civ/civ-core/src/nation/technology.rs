//! État technologique d'une nation.

use serde::{Deserialize, Serialize};

use crate::ids::TechnologyId;

fn default_era() -> u8 {
    1
}

fn default_research_speed() -> f64 {
    1.0
}

/// Technologies recherchées et recherche en cours.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TechnologyState {
    #[serde(default = "default_era")]
    pub era: u8,
    #[serde(default)]
    pub researched: Vec<TechnologyId>,
    #[serde(default)]
    pub researching: Option<TechnologyId>,
    #[serde(default)]
    pub research_points: f64,
    #[serde(default = "default_research_speed")]
    pub research_speed: f64,
    /// Open/native research-domain progress. Names remain localizable in the UI.
    #[serde(default)]
    pub domains: Vec<ResearchDomainProgress>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchDomainKind {
    Military,
    Economy,
    Diplomacy,
    Infrastructure,
    Technology,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResearchDomainProgress {
    pub domain: ResearchDomainKind,
    #[serde(default)]
    pub level: u32,
    #[serde(default)]
    pub points: f64,
}

impl TechnologyState {
    /// Vrai si la technologie est déjà débloquée.
    pub fn has(&self, technology: &TechnologyId) -> bool {
        self.researched.contains(technology)
    }

    /// Ajoute une technologie sans doublon.
    pub fn unlock(&mut self, technology: TechnologyId) {
        if self.researching.as_ref() == Some(&technology) {
            self.researching = None;
        }
        if !self.researched.contains(&technology) {
            self.researched.push(technology);
        }
    }

    /// Ensure all five native research-domain entries exist for UI/save consumers.
    pub fn ensure_domains(&mut self) {
        for domain in [
            ResearchDomainKind::Military,
            ResearchDomainKind::Economy,
            ResearchDomainKind::Diplomacy,
            ResearchDomainKind::Infrastructure,
            ResearchDomainKind::Technology,
        ] {
            if !self
                .domains
                .iter()
                .any(|progress| progress.domain == domain)
            {
                self.domains.push(ResearchDomainProgress {
                    domain,
                    level: 0,
                    points: 0.0,
                });
            }
        }
    }

    /// Add completed research points to a domain.
    pub fn advance_domain(&mut self, domain: ResearchDomainKind, points: f64) {
        self.ensure_domains();
        if let Some(progress) = self
            .domains
            .iter_mut()
            .find(|progress| progress.domain == domain)
        {
            progress.points = (progress.points + points).max(0.0);
            progress.level = progress.level.saturating_add(1);
        }
    }
}
