//! Guerre : belligérants, fronts, batailles, armées et paix.
//!
//! Les structures sont complètes et sérialisables. La résolution détaillée
//! des combats appartient à l'étape suivante ; le cœur expose déjà les données
//! nécessaires à une reprise exacte d'une sauvegarde.

use serde::{Deserialize, Serialize};

use crate::effects::Term;
use crate::ids::{
    ArmyId, BattleId, FrontId, NationId, PeaceOfferId, ProvinceId, TreatyId, UnitTypeId, WarGoalId,
    WarId,
};
use crate::text::LocalizedText;
use crate::time::GameDate;

/// Phase d'une guerre.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarStatus {
    Planned,
    Declared,
    Active,
    Ceasefire,
    PeaceNegotiation,
    Ended,
    Stalemate,
}

/// Cause initiale d'une guerre.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CasusBelliKind {
    BorderDispute,
    ClaimedProvince,
    AllianceObligation,
    TradeConflict,
    Ideological,
    RegimeChange,
    Independence,
    ResourceAccess,
    Revenge,
    Preemptive,
}

/// Camp d'un belligérant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BelligerentSide {
    Attacker,
    Defender,
    CoBelligerent,
}

/// Objectif de guerre.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarGoalKind {
    ConquerProvince,
    AnnexNation,
    Vassalize,
    Puppet,
    RegimeChange,
    Reparations,
    Demilitarize,
    Independence,
    ResourceAccess,
    BorderRectification,
    Protect,
    Humiliate,
}

/// État d'un front.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrontStatus {
    Quiet,
    Active,
    Breakthrough,
    Collapsed,
    Encircled,
    Ceasefire,
}

/// Résultat d'une bataille.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BattleResult {
    AttackerVictory,
    DefenderVictory,
    Stalemate,
    Ongoing,
    Withdrawal,
}

/// Statut d'une offre de paix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PeaceOfferStatus {
    Offered,
    Accepted,
    Rejected,
    Expired,
    Countered,
}

/// Camp d'une intervention.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionSide {
    Attacker,
    Defender,
    Mediator,
}

/// Position d'une armée.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArmyStance {
    Defensive,
    Offensive,
    Mobile,
    Garrison,
    Retreating,
    Encircled,
}

/// Cause et cible d'une guerre.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CasusBelli {
    pub kind: CasusBelliKind,
    pub target: NationId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
    pub province: Option<ProvinceId>,
}

/// Nation engagée dans une guerre.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Belligerent {
    pub nation: NationId,
    pub side: BelligerentSide,
    pub joined_date: GameDate,
    pub war_support: f64,
    #[serde(default)]
    pub casualties: u64,
    #[serde(default)]
    pub occupied_provinces: Vec<ProvinceId>,
    #[serde(default)]
    pub contribution: f64,
}

/// Objectif poursuivi par un belligérant.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WarGoal {
    pub id: WarGoalId,
    pub kind: WarGoalKind,
    pub target: NationId,
    #[serde(default)]
    pub provinces: Vec<ProvinceId>,
    pub priority: f64,
    #[serde(default)]
    pub satisfied: bool,
    #[serde(default)]
    pub secret: bool,
}

/// Front géographique d'une guerre.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Front {
    pub id: FrontId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<LocalizedText>,
    pub provinces: Vec<ProvinceId>,
    pub attacker: NationId,
    pub defender: NationId,
    pub status: FrontStatus,
    #[serde(default)]
    pub momentum: f64,
    pub supply: f64,
    #[serde(default)]
    pub battles_won: u64,
    #[serde(default)]
    pub battles_lost: u64,
}

/// Bataille localisée.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Battle {
    pub id: BattleId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub front: Option<FrontId>,
    pub province: ProvinceId,
    pub start_date: GameDate,
    pub end_date: Option<GameDate>,
    pub attacker: NationId,
    pub defender: NationId,
    pub result: BattleResult,
    pub attacker_losses: u64,
    pub defender_losses: u64,
    #[serde(default = "default_terrain_modifier")]
    pub terrain_modifier: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<LocalizedText>,
}

fn default_terrain_modifier() -> f64 {
    1.0
}

/// Offre de paix et ses clauses.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PeaceOffer {
    pub id: PeaceOfferId,
    pub by: NationId,
    pub to: NationId,
    pub date: GameDate,
    pub terms: Vec<Term>,
    pub status: PeaceOfferStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_acceptance: Option<f64>,
}

/// Intervention d'une tierce nation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Intervention {
    pub nation: NationId,
    pub side: InterventionSide,
    pub date: GameDate,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<LocalizedText>,
}

/// Score militaire agrégé.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WarScore {
    #[serde(default)]
    pub attacker: f64,
    #[serde(default)]
    pub defender: f64,
}

/// Etat de controle d'une zone d'assaut.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssaultZoneStatus {
    Neutral,
    Contested,
    Controlled,
}

/// Zone contestee sur une carte d'assaut.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssaultZone {
    pub id: crate::ids::AssaultZoneId,
    pub province: ProvinceId,
    pub status: AssaultZoneStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub controller: Option<NationId>,
    #[serde(default)]
    pub attacker_progress: f64,
    #[serde(default)]
    pub defender_progress: f64,
    #[serde(default)]
    pub score: f64,
}

/// Déplacement d'une armée entre deux provinces.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ArmyMovement {
    pub from: ProvinceId,
    pub to: ProvinceId,
    pub progress: f64,
}

/// Général commandant une armée.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct General {
    pub name: LocalizedText,
    pub skill: f64,
    #[serde(default)]
    pub traits: Vec<String>,
}

/// Contingent d'un type d'unité.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnitInstance {
    #[serde(rename = "type")]
    pub unit_type: UnitTypeId,
    pub count: u64,
    pub experience: f64,
    pub morale: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub equipment: Option<f64>,
    #[serde(default)]
    pub wounded: u64,
}

/// Armée d'une nation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Army {
    pub id: ArmyId,
    pub nation: NationId,
    pub name: LocalizedText,
    pub location: ProvinceId,
    pub units: Vec<UnitInstance>,
    pub supply: f64,
    pub morale: f64,
    pub stance: ArmyStance,
    pub movement: Option<ArmyMovement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub general: Option<General>,
    pub engaged_in: Option<BattleId>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Guerre complète.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct War {
    pub schema_version: u32,
    pub id: WarId,
    pub name: LocalizedText,
    pub start_date: GameDate,
    pub end_date: Option<GameDate>,
    pub status: WarStatus,
    pub casus_belli: CasusBelli,
    pub belligerents: Vec<Belligerent>,
    pub war_goals: Vec<WarGoal>,
    pub fronts: Vec<Front>,
    pub battles: Vec<Battle>,
    #[serde(default)]
    pub armies: Vec<Army>,
    pub score: WarScore,
    pub peace_offers: Vec<PeaceOffer>,
    #[serde(default)]
    pub interventions: Vec<Intervention>,
    pub treaty_id: Option<TreatyId>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub assault_zones: Vec<AssaultZone>,
}

impl War {
    /// Vrai si la guerre est encore active.
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            WarStatus::Declared | WarStatus::Active | WarStatus::PeaceNegotiation
        )
    }

    /// Vrai si la nation participe à la guerre.
    pub fn has_belligerent(&self, nation: &NationId) -> bool {
        self.belligerents
            .iter()
            .any(|belligerent| &belligerent.nation == nation)
    }

    /// Camp d'une nation, si elle participe.
    pub fn side_of(&self, nation: &NationId) -> Option<BelligerentSide> {
        self.belligerents
            .iter()
            .find(|belligerent| &belligerent.nation == nation)
            .map(|belligerent| belligerent.side)
    }

    /// Somme des pertes déclarées.
    pub fn total_casualties(&self) -> u64 {
        self.belligerents
            .iter()
            .map(|belligerent| belligerent.casualties)
            .sum()
    }
}

impl Army {
    /// Effectif total de l'armée.
    pub fn total_units(&self) -> u64 {
        self.units.iter().map(|unit| unit.count).sum()
    }
}
