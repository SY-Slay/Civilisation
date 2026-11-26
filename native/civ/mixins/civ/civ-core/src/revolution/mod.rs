//! Révolutions, factions et guerre civile.

use serde::{Deserialize, Serialize};

use crate::effects::Term;
use crate::ids::{FactionId, IdeologyId, NationId, ProvinceId, RevolutionId};
use crate::text::LocalizedText;
use crate::time::GameDate;

/// Phase d'une révolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevolutionStatus {
    Simmering,
    Agitation,
    Uprising,
    CivilWar,
    Negotiated,
    Reform,
    Crushed,
    Victory,
    Frozen,
}

/// Cause structurelle d'une révolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevolutionCause {
    EconomicCrisis,
    Inequality,
    Corruption,
    WarExhaustion,
    FoodShortage,
    ReligiousTension,
    EthnicTension,
    IdeologicalPurge,
    FailedReform,
    ForeignSubversion,
    SuccessionCrisis,
    MilitaryDefeat,
}

/// Demande d'une faction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactionDemand {
    FreeElections,
    LandReform,
    Independence,
    Autonomy,
    RegimeChange,
    ReligiousLaw,
    Secularization,
    WorkersCouncil,
    MilitaryReform,
    Peace,
    Reparations,
}

/// Type de soutien étranger à une faction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FactionForeignSupportKind {
    Funding,
    Weapons,
    Training,
    SafeHaven,
    Diplomatic,
}

/// Contrôle d'une province pendant une révolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProvinceControl {
    Government,
    Contested,
    Revolution,
    LocalMilitia,
    Anarchy,
}

/// Stratégie de réponse du gouvernement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernmentStrategy {
    Negotiate,
    Reform,
    Suppress,
    MartialLaw,
    Concede,
    DivideAndRule,
}

/// Camp soutenu par une intervention.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevolutionSide {
    Government,
    Revolution,
    Mediator,
}

/// Nature d'une intervention étrangère.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevolutionInterventionKind {
    Funding,
    Weapons,
    Advisors,
    Volunteers,
    Sanctions,
    Recognition,
    Airstrikes,
}

/// Issue d'une révolution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevolutionOutcomeKind {
    StatusQuo,
    Reform,
    NewGovernment,
    Independence,
    Autonomy,
    Partition,
    Crushed,
}

/// Soutien étranger à une faction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FactionForeignSupport {
    pub nation: NationId,
    pub kind: FactionForeignSupportKind,
    pub amount: f64,
}

/// Faction organisée.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Faction {
    pub id: FactionId,
    pub name: LocalizedText,
    pub ideology: IdeologyId,
    pub support: f64,
    pub demands: Vec<FactionDemand>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub leader: Option<LocalizedText>,
    pub militia_size: u64,
    pub legitimacy: f64,
    #[serde(default)]
    pub foreign_support: Vec<FactionForeignSupport>,
}

/// Soutien d'une province à la révolution.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProvinceSupport {
    pub province: ProvinceId,
    pub support: f64,
    pub control: ProvinceControl,
}

/// Réponse du gouvernement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GovernmentResponse {
    pub strategy: GovernmentStrategy,
    pub intensity: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub started_date: Option<GameDate>,
    #[serde(default)]
    pub offered_terms: Vec<Term>,
}

/// Intervention d'une nation dans une révolution.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ForeignIntervention {
    pub nation: NationId,
    pub side: RevolutionSide,
    pub kind: RevolutionInterventionKind,
    pub amount: f64,
    #[serde(default)]
    pub covert: bool,
}

/// Issue enregistrée d'une révolution.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RevolutionOutcome {
    pub kind: RevolutionOutcomeKind,
    pub new_government: Option<crate::ids::GovernmentId>,
    pub new_ideology: Option<IdeologyId>,
    #[serde(default)]
    pub terms: Vec<Term>,
    #[serde(default)]
    pub recognized_by: Vec<NationId>,
}

/// Révolution complète.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Revolution {
    pub schema_version: u32,
    pub id: RevolutionId,
    pub nation: NationId,
    pub start_date: GameDate,
    pub end_date: Option<GameDate>,
    pub status: RevolutionStatus,
    pub causes: Vec<RevolutionCause>,
    pub factions: Vec<Faction>,
    pub support_by_province: Vec<ProvinceSupport>,
    pub controlled_provinces: Vec<ProvinceId>,
    pub government_response: GovernmentResponse,
    #[serde(default)]
    pub casualties: u64,
    #[serde(default)]
    pub foreign_interventions: Vec<ForeignIntervention>,
    pub outcome: Option<RevolutionOutcome>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Revolution {
    /// Vrai si la révolution est encore active.
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            RevolutionStatus::Simmering
                | RevolutionStatus::Agitation
                | RevolutionStatus::Uprising
                | RevolutionStatus::CivilWar
                | RevolutionStatus::Frozen
        )
    }

    /// Faction avec le plus grand soutien.
    pub fn dominant_faction(&self) -> Option<&Faction> {
        self.factions
            .iter()
            .max_by(|left, right| left.support.total_cmp(&right.support))
    }
}
