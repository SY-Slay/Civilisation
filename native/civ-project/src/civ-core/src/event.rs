//! Événements narratifs et état persistant des déclencheurs.

use serde::{Deserialize, Serialize};

use crate::effects::{Condition, Effect, ScheduledEffect};
use crate::ids::{EventChoiceId, EventId, NationId, ProvinceId};
use crate::text::LocalizedText;
use crate::time::{GameDate, Tick};

fn default_weight() -> f64 {
    1.0
}

fn default_cooldown_days() -> u32 {
    30
}

fn default_max_fires() -> u32 {
    1
}

fn default_chance() -> f64 {
    1.0
}

fn default_ai_weight() -> f64 {
    1.0
}

/// Catégorie d'un événement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    Political,
    Economic,
    Military,
    Diplomatic,
    Revolution,
    Missile,
    Natural,
    Technological,
    Humanitarian,
}

/// Déclencheur data-driven.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventTrigger {
    #[serde(default)]
    pub conditions: Vec<Condition>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_date: Option<GameDate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_date: Option<GameDate>,
    #[serde(default)]
    pub nations: Vec<NationId>,
    #[serde(default)]
    pub provinces: Vec<ProvinceId>,
    #[serde(default)]
    pub requires_war: bool,
    #[serde(default)]
    pub requires_revolution: bool,
    #[serde(default = "default_chance")]
    pub chance: f64,
}

/// Choix proposé au joueur ou évalué par l'IA.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventChoice {
    pub id: EventChoiceId,
    pub text: LocalizedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tooltip: Option<LocalizedText>,
    #[serde(default)]
    pub requirements: Vec<Condition>,
    pub effects: Vec<Effect>,
    #[serde(default = "default_ai_weight")]
    pub ai_weight: f64,
    #[serde(default)]
    pub hidden: bool,
}

/// Définition complète d'un événement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventDefinition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<u32>,
    pub id: EventId,
    pub name: LocalizedText,
    pub description: LocalizedText,
    pub kind: EventKind,
    #[serde(default = "default_weight")]
    pub weight: f64,
    #[serde(default = "default_cooldown_days")]
    pub cooldown_days: u32,
    #[serde(default = "default_max_fires")]
    pub max_fires: u32,
    pub trigger: EventTrigger,
    pub choices: Vec<EventChoice>,
    pub ai_choice: Option<EventChoiceId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub severity: Option<f64>,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Événement actuellement ouvert.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActiveEvent {
    pub event: EventId,
    pub nation: NationId,
    pub start_tick: Tick,
    pub expires_tick: Tick,
    pub chosen: Option<EventChoiceId>,
}

/// Délai avant qu'un événement puisse se reproduire.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventCooldown {
    pub event: EventId,
    pub nation: NationId,
    pub ready_tick: Tick,
}

/// Nombre de déclenchements déjà effectués.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EventFiredCount {
    pub event: EventId,
    pub nation: NationId,
    pub count: u32,
}

/// État persistant du moteur d'événements.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EventState {
    #[serde(default)]
    pub active: Vec<ActiveEvent>,
    #[serde(default)]
    pub cooldowns: Vec<EventCooldown>,
    #[serde(default)]
    pub fired_counts: Vec<EventFiredCount>,
    /// Effets différés en attente d'application.
    #[serde(default)]
    pub pending: Vec<ScheduledEffect>,
}

impl EventState {
    /// Vrai si l'événement est en attente pour la nation.
    pub fn has_active(&self, event: &EventId, nation: &NationId) -> bool {
        self.active
            .iter()
            .any(|active| &active.event == event && &active.nation == nation)
    }

    /// Vrai si l'événement est en cooldown pour la nation.
    pub fn is_on_cooldown(&self, event: &EventId, nation: &NationId) -> bool {
        self.cooldowns
            .iter()
            .any(|cooldown| &cooldown.event == event && &cooldown.nation == nation)
    }

    /// Vrai si l'événement est encore bloqué au tick indiqué.
    pub fn is_on_cooldown_at(&self, event: &EventId, nation: &NationId, tick: Tick) -> bool {
        self.cooldowns.iter().any(|cooldown| {
            &cooldown.event == event && &cooldown.nation == nation && tick < cooldown.ready_tick
        })
    }

    /// Événement actif pour une nation, s'il existe.
    pub fn active_for(&self, nation: &NationId) -> Option<&ActiveEvent> {
        self.active
            .iter()
            .find(|active| &active.nation == nation && active.chosen.is_none())
    }

    /// Nombre de déclenchements enregistrés.
    pub fn fire_count(&self, event: &EventId, nation: &NationId) -> u32 {
        self.fired_counts
            .iter()
            .find(|count| &count.event == event && &count.nation == nation)
            .map(|count| count.count)
            .unwrap_or(0)
    }
}
