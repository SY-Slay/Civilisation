//! Modèle d'une nation fictive.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::economy::Economy;
use super::military::Military;
use super::technology::TechnologyState;
use crate::ids::{
    ArmyId, CultureId, GovernmentId, IdeologyId, MissileInstanceId, NationId, ProvinceId,
    ReligionId, RevolutionId, WarId,
};
use crate::text::LocalizedText;
use crate::time::GameDate;

fn default_growth_rate() -> f64 {
    0.005
}

fn default_true() -> bool {
    true
}

fn default_war_threshold() -> f64 {
    0.6
}

fn default_peace_threshold() -> f64 {
    0.3
}

/// Motif de drapeau dessiné par l'adaptateur.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlagPattern {
    Solid,
    HorizontalBands,
    VerticalBands,
    Diagonal,
    Cross,
    Canton,
    Chevron,
    Circle,
    Crescent,
    Starfield,
    Tribal,
    Industrial,
}

/// Description procédurale du drapeau.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Flag {
    pub palette: Vec<String>,
    pub pattern: FlagPattern,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emblem: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub motto: Option<LocalizedText>,
}

/// Dirigeant d'une nation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Leader {
    pub name: LocalizedText,
    pub title: LocalizedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub age: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<GameDate>,
    #[serde(default)]
    pub traits: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub portrait: Option<String>,
}

/// Population nationale agrégée.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NationPopulation {
    pub total: u64,
    pub workforce: u64,
    pub conscription_pool: u64,
    #[serde(default = "default_growth_rate")]
    pub growth_rate: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub urbanization: Option<f64>,
}

/// Archétype de l'IA déterministe.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AiArchetype {
    Merchant,
    Fortress,
    Warlord,
    Ideologue,
    Opportunist,
    Isolate,
    Hegemon,
    Revolutionary,
    Caretaker,
}

/// Ligne rouge déclenchant une réaction de l'IA.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RedLine {
    CapitalOccupied,
    CoreOccupied,
    AllyAttacked,
    NuclearFirstUse,
    BorderIncursion,
    TradeEmbargo,
    RegimeChangeAttempt,
    VassalAttacked,
}

/// Profil déterministe utilisé par le moteur de décision.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AiProfile {
    pub archetype: AiArchetype,
    pub aggression: f64,
    pub caution: f64,
    pub honor: f64,
    pub greed: f64,
    pub revanchism: f64,
    pub isolationism: f64,
    pub opportunism: f64,
    pub loyalty: f64,
    pub risk_tolerance: f64,
    pub patience: f64,
    #[serde(default = "default_war_threshold")]
    pub war_threshold: f64,
    #[serde(default = "default_peace_threshold")]
    pub peace_threshold: f64,
    #[serde(default)]
    pub priorities: BTreeMap<String, f64>,
    #[serde(default)]
    pub red_lines: Vec<RedLine>,
}

impl AiProfile {
    /// Priorité déclarée, ou zéro si absente.
    pub fn priority(&self, name: &str) -> f64 {
        self.priorities.get(name).copied().unwrap_or(0.0)
    }

    /// Vrai si la ligne rouge est présente.
    pub fn has_red_line(&self, line: RedLine) -> bool {
        self.red_lines.contains(&line)
    }
}

/// Type de revendication territoriale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimKind {
    Historical,
    Ethnic,
    Strategic,
    Economic,
    Religious,
    Revanchist,
}

/// Revendication sur une province.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Claim {
    pub province: ProvinceId,
    pub kind: ClaimKind,
    pub priority: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<GameDate>,
}

/// Type de suzeraineté.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubjectKind {
    Vassal,
    Protectorate,
    Puppet,
    Dominion,
    Client,
    Occupied,
}

/// Nation placée sous influence d'une autre.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Subject {
    pub nation: NationId,
    pub kind: SubjectKind,
    pub autonomy: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<GameDate>,
}

/// État fictif jouable ou contrôlé par l'IA.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Nation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<u32>,
    pub id: NationId,
    pub name: LocalizedText,
    pub adjective: LocalizedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_name: Option<LocalizedText>,
    pub color: String,
    pub flag: Flag,
    pub capital: ProvinceId,
    pub provinces: Vec<ProvinceId>,
    pub government: GovernmentId,
    pub ideology: IdeologyId,
    pub culture: CultureId,
    pub religion: ReligionId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    pub leader: Leader,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub founded: Option<GameDate>,
    pub population: NationPopulation,
    pub economy: Economy,
    pub military: Military,
    pub stability: f64,
    pub legitimacy: f64,
    pub prestige: f64,
    pub war_exhaustion: f64,
    pub unrest: f64,
    pub technology: TechnologyState,
    pub ai_profile: AiProfile,
    #[serde(default)]
    pub claims: Vec<Claim>,
    #[serde(default)]
    pub subjects: Vec<Subject>,
    pub overlord: Option<NationId>,
    #[serde(default)]
    pub allies: Vec<NationId>,
    #[serde(default)]
    pub wars: Vec<WarId>,
    #[serde(default)]
    pub revolutions: Vec<RevolutionId>,
    #[serde(default)]
    pub armies: Vec<ArmyId>,
    #[serde(default)]
    pub missiles: Vec<MissileInstanceId>,
    #[serde(default = "default_true")]
    pub is_playable: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Nation {
    /// Nombre de provinces contrôlées de jure.
    pub fn province_count(&self) -> usize {
        self.provinces.len()
    }

    /// Vrai si la nation possède encore au moins une province.
    pub fn is_alive(&self) -> bool {
        !self.provinces.is_empty()
    }

    /// Vrai si la province appartient à la nation.
    pub fn owns(&self, province: &ProvinceId) -> bool {
        self.provinces.contains(province)
    }

    /// Vrai si la nation est en guerre.
    pub fn is_at_war(&self) -> bool {
        !self.wars.is_empty()
    }

    /// Vrai si la nation est un sujet.
    pub fn is_subject(&self) -> bool {
        self.overlord.is_some()
    }

    /// Nom court, ou nom complet s'il n'existe pas.
    pub fn display_short_name(&self) -> &str {
        self.short_name
            .as_ref()
            .map(LocalizedText::resolve)
            .unwrap_or_else(|| self.name.resolve())
    }

    /// Revenu militaire approximatif, sans simulation de combat.
    pub fn mobilisation_capacity(&self) -> u64 {
        self.population
            .conscription_pool
            .min(self.military.manpower)
    }
}
