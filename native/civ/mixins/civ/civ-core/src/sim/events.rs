//! Déclencheurs, choix et effets différés des événements.

use serde_json::Value;

use crate::effects::{Condition, Effect, ScheduledEffect};
use crate::error::{CoreError, CoreResult};
use crate::event::{
    ActiveEvent, EventChoice, EventCooldown, EventDefinition, EventFiredCount, EventTrigger,
};
use crate::ids::{EventChoiceId, EventId, NationId, ProvinceId};
use crate::sim::diplomacy::relation_mut;
use crate::sim::{clamp, is_at_war, SimEvent, SimEventKind, Simulation};

/// Applique les événements en attente et déclenche les nouveaux.
pub(crate) fn tick(sim: &mut Simulation<'_>, events: &mut Vec<SimEvent>) -> CoreResult<()> {
    process_pending(sim, events)?;
    expire_active(sim, events);

    let nation_ids: Vec<NationId> = sim
        .nations
        .iter()
        .filter(|nation| nation.is_alive())
        .map(|nation| nation.id.clone())
        .collect();
    for nation_id in nation_ids {
        if sim.event_state.active_for(&nation_id).is_some() {
            continue;
        }
        let Some(event_id) = choose_definition(sim, &nation_id) else {
            continue;
        };
        let Some(definition) = sim.data.event(&event_id).cloned() else {
            continue;
        };
        if sim.player_nation.as_ref() == Some(&nation_id) {
            let active = ActiveEvent {
                event: definition.id.clone(),
                nation: nation_id.clone(),
                start_tick: sim.tick,
                expires_tick: sim.tick.saturating_add(30 * u64::from(sim.ticks_per_day)),
                chosen: None,
            };
            sim.event_state.active.push(active);
            events.push(SimEvent::nation(
                sim.tick,
                sim.date,
                SimEventKind::Event,
                &nation_id,
                format!("Événement en attente : {}", definition.name.resolve()),
            ));
        } else {
            let choice = choose_ai_choice(sim, &nation_id, &definition);
            apply_choice(sim, &nation_id, &definition, &choice, events)?;
        }
    }
    Ok(())
}

fn process_pending(sim: &mut Simulation<'_>, events: &mut Vec<SimEvent>) -> CoreResult<()> {
    let due: Vec<ScheduledEffect> = sim
        .event_state
        .pending
        .iter()
        .filter(|scheduled| scheduled.apply_tick <= sim.tick)
        .cloned()
        .collect();
    for scheduled in due {
        apply_effect(sim, &scheduled.effect, scheduled.nation.as_ref(), events)?;
        sim.event_state.pending.retain(|value| {
            value.apply_tick != scheduled.apply_tick
                || value.nation != scheduled.nation
                || value.effect != scheduled.effect
        });
    }
    Ok(())
}

fn expire_active(sim: &mut Simulation<'_>, events: &mut Vec<SimEvent>) {
    let expired: Vec<ActiveEvent> = sim
        .event_state
        .active
        .iter()
        .filter(|active| active.chosen.is_none() && sim.tick >= active.expires_tick)
        .cloned()
        .collect();
    for active in expired {
        sim.event_state
            .active
            .retain(|value| value.event != active.event || value.nation != active.nation);
        events.push(SimEvent::nation(
            sim.tick,
            sim.date,
            SimEventKind::Event,
            &active.nation,
            format!("Événement expiré sans décision : {}", active.event),
        ));
    }
}

fn choose_definition(sim: &mut Simulation<'_>, nation: &NationId) -> Option<EventId> {
    let mut definitions: Vec<EventDefinition> = sim.data.events.clone();
    definitions.sort_by(|left, right| left.id.cmp(&right.id));
    for definition in definitions {
        if sim.event_state.fire_count(&definition.id, nation) >= definition.max_fires {
            continue;
        }
        if sim
            .event_state
            .is_on_cooldown_at(&definition.id, nation, sim.tick)
        {
            continue;
        }
        if !trigger_matches(sim, nation, &definition.trigger) {
            continue;
        }
        if sim.rng.chance(definition.trigger.chance) {
            return Some(definition.id);
        }
    }
    None
}

fn trigger_matches(sim: &Simulation<'_>, nation: &NationId, trigger: &EventTrigger) -> bool {
    if trigger.min_date.is_some_and(|date| sim.date < date)
        || trigger.max_date.is_some_and(|date| sim.date > date)
    {
        return false;
    }
    if !trigger.nations.is_empty() && !trigger.nations.contains(nation) {
        return false;
    }
    if !trigger.provinces.is_empty() {
        let owns_one = sim
            .nations
            .iter()
            .find(|value| &value.id == nation)
            .is_some_and(|value| {
                trigger
                    .provinces
                    .iter()
                    .any(|province| value.provinces.contains(province))
            });
        if !owns_one {
            return false;
        }
    }
    if trigger.requires_war && !is_at_war(sim, nation) {
        return false;
    }
    if trigger.requires_revolution
        && !sim
            .revolutions
            .iter()
            .any(|revolution| &revolution.nation == nation && revolution.is_active())
    {
        return false;
    }
    trigger
        .conditions
        .iter()
        .all(|condition| condition_matches(sim, nation, condition))
}

fn condition_matches(sim: &Simulation<'_>, nation: &NationId, condition: &Condition) -> bool {
    let Some(value) = sim.nations.iter().find(|value| &value.id == nation) else {
        return false;
    };
    let threshold = arg_f64(&condition.args, "value")
        .or_else(|| arg_f64(&condition.args, "delta"))
        .unwrap_or(0.0);
    match condition.kind.as_str() {
        "stability_below" => value.stability < threshold,
        "stability_above" => value.stability > threshold,
        "unrest_above" => value.unrest > threshold,
        "legitimacy_below" => value.legitimacy < threshold,
        "war_exhaustion_above" => value.war_exhaustion > threshold,
        "gdp_below" => value.economy.gdp < threshold,
        "treasury_below" => value.economy.treasury < threshold,
        "at_war" => is_at_war(sim, nation),
        "has_revolution" => sim
            .revolutions
            .iter()
            .any(|revolution| &revolution.nation == nation && revolution.is_active()),
        "sanctioned" => !value.economy.sanctions.is_empty(),
        "research_at_least" => value.technology.researched.len() as f64 >= threshold,
        "relation_opinion_below" => {
            if let Some(other) = arg_other_nation(&condition.args, nation) {
                sim.diplomacy
                    .relation(nation, &other)
                    .is_some_and(|relation| relation.opinion < threshold)
            } else {
                sim.diplomacy
                    .relations
                    .iter()
                    .filter(|relation| relation.involves(nation))
                    .any(|relation| relation.opinion < threshold)
            }
        }
        "border_tension_above" => {
            if let Some(other) = arg_other_nation(&condition.args, nation) {
                sim.diplomacy
                    .relation(nation, &other)
                    .is_some_and(|relation| relation.border_tension > threshold)
            } else {
                sim.diplomacy
                    .relations
                    .iter()
                    .filter(|relation| relation.involves(nation))
                    .any(|relation| relation.border_tension > threshold)
            }
        }
        _ => false,
    }
}

fn choose_ai_choice(
    sim: &Simulation<'_>,
    nation: &NationId,
    definition: &EventDefinition,
) -> EventChoiceId {
    if let Some(choice) = &definition.ai_choice {
        return choice.clone();
    }
    let mut choices: Vec<&EventChoice> = definition
        .choices
        .iter()
        .filter(|choice| {
            !choice.hidden
                && choice
                    .requirements
                    .iter()
                    .all(|condition| condition_matches(sim, nation, condition))
        })
        .collect();
    choices.sort_by(|left, right| {
        right
            .ai_weight
            .total_cmp(&left.ai_weight)
            .then(left.id.cmp(&right.id))
    });
    choices
        .first()
        .map(|choice| choice.id.clone())
        .unwrap_or_else(|| definition.choices[0].id.clone())
}

fn apply_choice(
    sim: &mut Simulation<'_>,
    nation: &NationId,
    definition: &EventDefinition,
    choice_id: &EventChoiceId,
    events: &mut Vec<SimEvent>,
) -> CoreResult<()> {
    let choice = definition
        .choices
        .iter()
        .find(|choice| &choice.id == choice_id)
        .ok_or_else(|| CoreError::data(format!("choix inconnu : {choice_id}")))?;
    for effect in &choice.effects {
        if effect.delay_ticks == 0 {
            apply_effect(sim, effect, Some(nation), events)?;
        } else {
            sim.event_state.pending.push(ScheduledEffect {
                apply_tick: sim
                    .tick
                    .saturating_add(effect.delay_ticks * u64::from(sim.ticks_per_day)),
                nation: Some(nation.clone()),
                effect: effect.clone(),
            });
        }
    }
    sim.event_state
        .active
        .retain(|active| active.event != definition.id || active.nation != *nation);
    if let Some(cooldown) = sim
        .event_state
        .cooldowns
        .iter_mut()
        .find(|cooldown| cooldown.event == definition.id && cooldown.nation == *nation)
    {
        cooldown.ready_tick = sim
            .tick
            .saturating_add(u64::from(definition.cooldown_days) * u64::from(sim.ticks_per_day));
    } else {
        sim.event_state.cooldowns.push(EventCooldown {
            event: definition.id.clone(),
            nation: nation.clone(),
            ready_tick: sim
                .tick
                .saturating_add(u64::from(definition.cooldown_days) * u64::from(sim.ticks_per_day)),
        });
    }
    if let Some(count) = sim
        .event_state
        .fired_counts
        .iter_mut()
        .find(|count| count.event == definition.id && count.nation == *nation)
    {
        count.count = count.count.saturating_add(1);
    } else {
        sim.event_state.fired_counts.push(EventFiredCount {
            event: definition.id.clone(),
            nation: nation.clone(),
            count: 1,
        });
    }
    events.push(SimEvent::nation(
        sim.tick,
        sim.date,
        SimEventKind::Event,
        nation,
        format!(
            "Événement {} : choix {} appliqué.",
            definition.id, choice.id
        ),
    ));
    Ok(())
}

/// Résout l'événement actif du joueur pour une nation.
pub fn resolve(
    sim: &mut Simulation<'_>,
    nation: &NationId,
    choice_id: &EventChoiceId,
    events: &mut Vec<SimEvent>,
) -> CoreResult<()> {
    let active = sim
        .event_state
        .active_for(nation)
        .cloned()
        .ok_or_else(|| CoreError::data("aucun événement actif pour cette nation"))?;
    let definition = sim
        .data
        .event(&active.event)
        .cloned()
        .ok_or_else(|| CoreError::data("définition d'événement absente"))?;
    apply_choice(sim, nation, &definition, choice_id, events)
}

fn apply_effect(
    sim: &mut Simulation<'_>,
    effect: &Effect,
    nation: Option<&NationId>,
    events: &mut Vec<SimEvent>,
) -> CoreResult<()> {
    let target = arg_nation(&effect.args).or_else(|| nation.cloned());
    let Some(target) = target else {
        return Err(CoreError::data("effet sans nation cible"));
    };
    let delta = arg_f64(&effect.args, "delta")
        .or_else(|| arg_f64(&effect.args, "value"))
        .unwrap_or(0.0);
    match effect.kind.as_str() {
        "stability" => {
            if let Some(nation) = sim.nations.iter_mut().find(|value| value.id == target) {
                nation.stability = clamp(nation.stability + delta, 0.0, 100.0);
            }
        }
        "legitimacy" => {
            if let Some(nation) = sim.nations.iter_mut().find(|value| value.id == target) {
                nation.legitimacy = clamp(nation.legitimacy + delta, 0.0, 100.0);
            }
        }
        "unrest" => {
            if let Some(nation) = sim.nations.iter_mut().find(|value| value.id == target) {
                nation.unrest = clamp(nation.unrest + delta, 0.0, 100.0);
            }
        }
        "war_exhaustion" => {
            if let Some(nation) = sim.nations.iter_mut().find(|value| value.id == target) {
                nation.war_exhaustion = clamp(nation.war_exhaustion + delta, 0.0, 100.0);
            }
        }
        "prestige" => {
            if let Some(nation) = sim.nations.iter_mut().find(|value| value.id == target) {
                nation.prestige = clamp(nation.prestige + delta, 0.0, 100.0);
            }
        }
        "treasury" => {
            if let Some(nation) = sim.nations.iter_mut().find(|value| value.id == target) {
                nation.economy.treasury = (nation.economy.treasury + delta).max(0.0);
            }
        }
        "gdp" => {
            if let Some(nation) = sim.nations.iter_mut().find(|value| value.id == target) {
                nation.economy.gdp = (nation.economy.gdp + delta).max(1.0);
            }
        }
        "manpower" => {
            if let Some(nation) = sim.nations.iter_mut().find(|value| value.id == target) {
                let value = delta.max(0.0) as u64;
                nation.military.manpower = nation.military.manpower.saturating_add(value);
            }
        }
        "army_size" => {
            if let Some(nation) = sim.nations.iter_mut().find(|value| value.id == target) {
                let value = delta.max(0.0) as u64;
                nation.military.army_size = nation.military.army_size.saturating_add(value);
            }
        }
        "research_points" => {
            if let Some(nation) = sim.nations.iter_mut().find(|value| value.id == target) {
                nation.technology.research_points =
                    (nation.technology.research_points + delta).max(0.0);
            }
        }
        "relation_opinion" | "opinion" => {
            let Some(other) = arg_other_nation(&effect.args, &target)
                .or_else(|| most_relevant_other(sim, &target))
            else {
                return Err(CoreError::data("effet d'opinion sans autre nation"));
            };
            let tick = sim.tick;
            if let Some(relation) = relation_mut(sim, &target, &other) {
                relation.opinion = clamp(relation.opinion + delta, -100.0, 100.0);
                relation.last_change_tick = Some(tick);
            }
        }
        "border_tension" => {
            let Some(other) = arg_other_nation(&effect.args, &target)
                .or_else(|| most_relevant_other(sim, &target))
            else {
                return Err(CoreError::data("effet de tension sans autre nation"));
            };
            let tick = sim.tick;
            if let Some(relation) = relation_mut(sim, &target, &other) {
                relation.border_tension = clamp(relation.border_tension + delta, 0.0, 100.0);
                relation.last_change_tick = Some(tick);
            }
        }
        "start_revolution" => {
            super::revolution::start_revolution(sim, &target, events);
        }
        "province_unrest" => {
            if let Some(province) = arg_province(&effect.args) {
                if let Some(value) = sim.world.province_mut(&province) {
                    value.unrest = clamp(value.unrest + delta, 0.0, 100.0);
                }
            }
        }
        _ => {
            return Err(CoreError::data(format!("effet inconnu : {}", effect.kind)));
        }
    }
    Ok(())
}

/// Nation adverse la plus pertinente pour un effet bilatéral sans cible explicite.
///
/// On privilégie la tension frontalière la plus forte, puis l'opinion la plus
/// basse, puis l'ordre lexicographique : le résultat reste déterministe.
fn most_relevant_other(sim: &Simulation<'_>, nation: &NationId) -> Option<NationId> {
    let mut candidates: Vec<&crate::diplomacy::Relation> = sim
        .diplomacy
        .relations
        .iter()
        .filter(|relation| relation.involves(nation))
        .collect();
    candidates.sort_by(|left, right| {
        right
            .border_tension
            .total_cmp(&left.border_tension)
            .then(left.opinion.total_cmp(&right.opinion))
            .then(left.other(nation).cmp(&right.other(nation)))
    });
    candidates
        .first()
        .and_then(|relation| relation.other(nation).cloned())
}

fn arg_f64(args: &Option<Value>, key: &str) -> Option<f64> {
    match args {
        Some(Value::Number(value)) => value.as_f64(),
        Some(Value::Object(map)) => map.get(key).and_then(Value::as_f64),
        _ => None,
    }
}

fn arg_nation(args: &Option<Value>) -> Option<NationId> {
    arg_named_nation(args, "nation")
}

fn arg_named_nation(args: &Option<Value>, key: &str) -> Option<NationId> {
    let Some(Value::Object(map)) = args else {
        return None;
    };
    map.get(key)
        .and_then(Value::as_str)
        .and_then(|value| NationId::new(value).ok())
}

/// Nation adverse d'un effet ou d'une condition bilatérale.
///
/// `other` est la forme explicite. `other_nation` est accepté comme alias.
/// `nation` reste accepté pour les données existantes, sauf s'il désigne la
/// nation cible elle-même.
fn arg_other_nation(args: &Option<Value>, target: &NationId) -> Option<NationId> {
    arg_named_nation(args, "other")
        .or_else(|| arg_named_nation(args, "other_nation"))
        .or_else(|| arg_nation(args).filter(|nation| nation != target))
}

fn arg_province(args: &Option<Value>) -> Option<ProvinceId> {
    let Some(Value::Object(map)) = args else {
        return None;
    };
    map.get("province")
        .and_then(Value::as_str)
        .and_then(|value| ProvinceId::new(value).ok())
}

#[cfg(all(test, not(feature = "embedded-core")))]
mod tests {
    use super::*;
    use serde_json::json;

    fn nation(value: &str) -> NationId {
        NationId::new(value).expect("identifiant de nation valide")
    }

    #[test]
    fn choisit_la_nation_adverse_explicite() {
        let explicit = Some(json!({ "nation": "ardenne", "other": "kesh" }));
        assert_eq!(
            arg_other_nation(&explicit, &nation("ardenne")),
            Some(nation("kesh"))
        );

        let legacy = Some(json!({ "nation": "kesh" }));
        assert_eq!(
            arg_other_nation(&legacy, &nation("ardenne")),
            Some(nation("kesh"))
        );

        let self_reference = Some(json!({ "nation": "ardenne" }));
        assert_eq!(arg_other_nation(&self_reference, &nation("ardenne")), None);
    }
}
