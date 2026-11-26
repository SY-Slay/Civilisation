//! Missiles : définitions de types et instances en vol.
//!
//! Le cœur ne connaît ni particules ni sons Minecraft. Il produit des
//! `DamageReport` ; l'adaptateur décide comment les rendre.

use serde::{Deserialize, Serialize};

use crate::effects::Effect;
use crate::ids::{MissileInstanceId, MissileTypeId, NationId, ProvinceId};
use crate::math::Vec3;
use crate::text::LocalizedText;
use crate::time::Tick;

/// Famille de missile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissileKind {
    Ballistic,
    Cruise,
    Hypersonic,
    Orbital,
    Drone,
    RocketArtillery,
}

/// Profil de vol.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlightProfile {
    Low,
    High,
    #[default]
    Ballistic,
    Orbital,
    Loitering,
}

/// Type de charge militaire.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarheadKind {
    Conventional,
    Nuclear,
    Chemical,
    Emp,
    Incendiary,
    Decoy,
}

/// Plateforme de lancement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaunchPlatform {
    Silo,
    MobileLauncher,
    Submarine,
    Ship,
    Aircraft,
    Space,
    GroundBattery,
}

/// État d'une instance de missile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissileStatus {
    Preparing,
    Fueled,
    Launched,
    InFlight,
    Terminal,
    Intercepted,
    Impact,
    Failed,
    Cancelled,
}

/// Nature de la cible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetKind {
    Province,
    City,
    Army,
    Infrastructure,
    MissileSite,
    Decoy,
}

/// Résultat d'une interception.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterceptorResult {
    Hit,
    Miss,
    Partial,
    Unknown,
}

/// Charge militaire et ses effets.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Warhead {
    pub kind: WarheadKind,
    #[serde(default)]
    pub yield_kt: f64,
    pub blast_radius_blocks: f64,
    #[serde(default)]
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub fallout: bool,
    #[serde(default)]
    pub escalation_delta: f64,
}

/// Paramètres d'interception d'un type de missile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InterceptionProfile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_chance: Option<f64>,
    #[serde(default)]
    pub counter_types: Vec<MissileTypeId>,
    #[serde(default)]
    pub terminal_maneuver: bool,
}

/// Effets visuels demandés à l'adaptateur.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExplosionEffectSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub particle: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flash_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shake_radius: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smoke_ticks: Option<u32>,
    #[serde(default)]
    pub orbital_laser_hook: bool,
}

/// Définition d'un type de missile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MissileType {
    pub id: MissileTypeId,
    pub name: LocalizedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<LocalizedText>,
    /// Open standardized classification 1 (least dangerous) to 5 (most dangerous).
    #[serde(default = "default_danger_tier")]
    pub danger_tier: u8,
    pub kind: MissileKind,
    pub range_km: f64,
    pub speed_kmh: f64,
    #[serde(default)]
    pub flight_profile: FlightProfile,
    pub accuracy_m: f64,
    pub warhead: Warhead,
    pub cost: f64,
    pub build_time_days: u32,
    #[serde(default = "default_max_stock")]
    pub max_stock: u32,
    pub defense_penetration: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interception: Option<InterceptionProfile>,
    #[serde(default)]
    pub launch_platforms: Vec<LaunchPlatform>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explosion_effects: Option<ExplosionEffectSpec>,
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_max_stock() -> u32 {
    99
}

fn default_danger_tier() -> u8 {
    1
}

/// Cible d'un missile.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MissileTarget {
    pub kind: TargetKind,
    pub province: ProvinceId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<Vec3>,
    pub entity_id: Option<crate::ids::ArmyId>,
}

/// Trajectoire visuelle et temporelle.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Trajectory {
    #[serde(default)]
    pub points: Vec<Vec3>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub apogee_km: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_speed_kmh: Option<f64>,
}

/// Tentative d'interception.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Interceptor {
    pub by: NationId,
    pub tick: Tick,
    pub chance: f64,
    pub result: InterceptorResult,
}

/// Conséquences d'un impact.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DamageReport {
    #[serde(default)]
    pub casualties: u64,
    #[serde(default)]
    pub infrastructure_damage: f64,
    #[serde(default)]
    pub provinces_affected: Vec<ProvinceId>,
    #[serde(default)]
    pub escalation: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<LocalizedText>,
}

/// Missile en vol ou déjà résolu.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MissileInstance {
    pub id: MissileInstanceId,
    #[serde(rename = "type")]
    pub missile_type: MissileTypeId,
    pub owner: NationId,
    pub launcher_province: ProvinceId,
    pub target: MissileTarget,
    pub launch_tick: Tick,
    pub arrival_tick: Tick,
    pub status: MissileStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trajectory: Option<Trajectory>,
    #[serde(default)]
    pub interceptors: Vec<Interceptor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_report: Option<DamageReport>,
    #[serde(default)]
    pub detected_by: Vec<NationId>,
}

impl MissileInstance {
    /// Vrai si le missile est encore en vol.
    pub fn is_in_flight(&self) -> bool {
        matches!(
            self.status,
            MissileStatus::Launched | MissileStatus::InFlight | MissileStatus::Terminal
        )
    }

    /// Vrai si le missile a atteint sa cible.
    pub fn has_impacted(&self) -> bool {
        self.status == MissileStatus::Impact
    }

    /// Vrai si le missile est arrivé à son tick d'arrivée.
    pub fn should_resolve(&self, tick: Tick) -> bool {
        self.is_in_flight() && tick >= self.arrival_tick
    }
}
