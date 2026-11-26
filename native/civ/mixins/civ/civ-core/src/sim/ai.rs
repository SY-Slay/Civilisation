//! IA déterministe à utilité.
//!
//! Chaque nation évalue un petit nombre d'actions avec des formules pures.
//! L'ordre d'itération est trié par identifiant et les rares tirages aléatoires
//! passent par le RNG de la partie. Aucun appel réseau ni modèle de langage
//! n'intervient.

use std::collections::BTreeSet;

use crate::diplomacy::{RelationStatus, SanctionKind, TreatyKind};
use crate::ids::{NationId, ProvinceId};
use crate::sim::diplomacy::{add_sanction, declare_war, has_blocking_treaty, sign_treaty};
use crate::sim::missile::{launch, range_between};
use crate::sim::{clamp, is_at_war, nation_power, SimEvent, SimEventKind, Simulation};

/// Exécute les décisions d'un jour.
pub(crate) fn tick(sim: &mut Simulation<'_>, events: &mut Vec<SimEvent>) {
    let mut nation_ids: Vec<NationId> = sim
        .nations
        .iter()
        .filter(|nation| nation.is_alive())
        .map(|nation| nation.id.clone())
        .collect();
    nation_ids.sort();
    for nation_id in nation_ids {
        if sim.player_nation.as_ref() == Some(&nation_id) {
            continue;
        }
        choose_research(sim, &nation_id);
        consider_diplomacy(sim, &nation_id, events);
        build_army(sim, &nation_id);
        if !is_at_war(sim, &nation_id) {
            consider_war(sim, &nation_id, events);
        }
        consider_missile(sim, &nation_id, events);
    }
}

fn choose_research(sim: &mut Simulation<'_>, nation_id: &NationId) {
    let Some(nation) = sim.nations.iter().find(|nation| &nation.id == nation_id) else {
        return;
    };
    if nation.technology.researching.is_some() {
        return;
    }
    let researched: BTreeSet<_> = nation.technology.researched.iter().cloned().collect();
    let era = nation.technology.era;
    let aggression = nation.ai_profile.aggression;
    let mut candidates: Vec<_> = sim
        .data
        .technologies
        .iter()
        .filter(|technology| {
            !researched.contains(&technology.id)
                && technology.era <= era.saturating_add(1)
                && technology
                    .prerequisites
                    .iter()
                    .all(|prerequisite| researched.contains(prerequisite))
        })
        .collect();
    candidates.sort_by(|left, right| {
        let left_score = technology_score(left, aggression);
        let right_score = technology_score(right, aggression);
        right_score
            .total_cmp(&left_score)
            .then(left.id.cmp(&right.id))
    });
    let Some(choice) = candidates.first() else {
        return;
    };
    let choice = choice.id.clone();
    if let Some(nation) = sim
        .nations
        .iter_mut()
        .find(|nation| &nation.id == nation_id)
    {
        nation.technology.researching = Some(choice);
    }
}

fn technology_score(technology: &crate::data::TechnologyDefinition, aggression: f64) -> f64 {
    let military_bonus = if matches!(
        technology.category,
        crate::data::TechnologyCategory::Military
            | crate::data::TechnologyCategory::Missile
            | crate::data::TechnologyCategory::Aerospace
    ) {
        aggression * 25.0
    } else {
        0.0
    };
    technology.ai_weight * 100.0 / (technology.cost.max(1.0) + 1.0) + military_bonus
}

fn consider_diplomacy(sim: &mut Simulation<'_>, nation_id: &NationId, events: &mut Vec<SimEvent>) {
    let Some(nation) = sim.nations.iter().find(|nation| &nation.id == nation_id) else {
        return;
    };
    let profile = nation.ai_profile.clone();
    let others: Vec<NationId> = sim
        .nations
        .iter()
        .filter(|other| &other.id != nation_id && other.is_alive())
        .map(|other| other.id.clone())
        .collect();
    for other in others {
        if is_at_war(sim, nation_id) || is_at_war(sim, &other) {
            continue;
        }
        if has_blocking_treaty(sim, nation_id, &other) {
            continue;
        }
        let Some(relation) = sim.diplomacy.relation(nation_id, &other).cloned() else {
            continue;
        };
        if relation.status == RelationStatus::Truce
            && relation.last_change_tick.is_some_and(|tick| {
                sim.tick.as_u64().saturating_sub(tick.as_u64()) < 720 * u64::from(sim.ticks_per_day)
            })
        {
            continue;
        }
        if relation.opinion > 60.0
            && relation.trust > 60.0
            && profile.honor > 0.55
            && !relation.status.eq(&RelationStatus::Allied)
        {
            let _ = sign_treaty(
                sim,
                TreatyKind::Alliance,
                &[nation_id.clone(), other.clone()],
                Some(1440),
                events,
            );
            continue;
        }
        if relation.opinion > 15.0 && relation.trust > 40.0 && relation.border_tension < 45.0 {
            let kind = if profile.greed > 0.55 {
                TreatyKind::Trade
            } else {
                TreatyKind::NonAggression
            };
            let _ = sign_treaty(
                sim,
                kind,
                &[nation_id.clone(), other.clone()],
                Some(1080),
                events,
            );
            continue;
        }
        if relation.opinion < -35.0
            && relation.border_tension > 45.0
            && profile.greed > 0.35
            && sim
                .diplomacy
                .sanctions
                .iter()
                .all(|sanction| !(sanction.is_active() && sanction.target == other))
        {
            let _ = add_sanction(
                sim,
                std::slice::from_ref(nation_id),
                &other,
                SanctionKind::Trade,
                0.45,
                events,
            );
        }
    }
}

fn build_army(sim: &mut Simulation<'_>, nation_id: &NationId) {
    let Some(index) = sim
        .nations
        .iter()
        .position(|nation| &nation.id == nation_id)
    else {
        return;
    };
    let nation = &sim.nations[index];
    if !nation.is_alive() {
        return;
    }
    let target = (nation.population.conscription_pool as f64 * 0.35) as u64;
    if nation.military.army_size >= target || target == 0 {
        return;
    }
    let available = (nation.economy.gdp * 0.005).max(1.0);
    let recruits = ((available / 500.0) as u64)
        .min(target.saturating_sub(nation.military.army_size))
        .min(2_000);
    if recruits == 0 || nation.economy.treasury < available {
        return;
    }
    if let Some(nation) = sim.nations.get_mut(index) {
        nation.military.army_size = nation.military.army_size.saturating_add(recruits);
        nation.military.manpower = nation.military.manpower.saturating_add(recruits / 2);
        nation.economy.treasury -= available;
        nation.military.readiness = clamp(nation.military.readiness + 0.2, 0.0, 100.0);
    }
}

fn consider_war(sim: &mut Simulation<'_>, nation_id: &NationId, events: &mut Vec<SimEvent>) {
    if sim.date.days_since(sim.start_date) < 30 {
        return;
    }
    let Some(nation) = sim.nations.iter().find(|nation| &nation.id == nation_id) else {
        return;
    };
    let profile = nation.ai_profile.clone();
    let own_power = nation_power(nation).max(1.0);
    let own_exhaustion = nation.war_exhaustion;
    let targets: Vec<NationId> = sim
        .nations
        .iter()
        .filter(|other| &other.id != nation_id && other.is_alive())
        .map(|other| other.id.clone())
        .collect();
    let mut best: Option<(f64, NationId, Option<ProvinceId>)> = None;
    for target in targets {
        if is_at_war(sim, &target) {
            continue;
        }
        if has_blocking_treaty(sim, nation_id, &target) {
            continue;
        }
        let Some(target_nation) = sim.nations.iter().find(|nation| nation.id == target) else {
            continue;
        };
        if nation.allies.contains(&target) {
            continue;
        }
        let Some(relation) = sim.diplomacy.relation(nation_id, &target) else {
            continue;
        };
        if relation.status == RelationStatus::Truce
            && relation.last_change_tick.is_some_and(|tick| {
                sim.tick.as_u64().saturating_sub(tick.as_u64()) < 720 * u64::from(sim.ticks_per_day)
            })
        {
            continue;
        }
        let target_power = nation_power(target_nation).max(1.0);
        let ratio = target_power / own_power;
        let claimed = claimed_province(sim, nation_id, &target);
        let claim_score = if claimed.is_some() { 45.0 } else { 0.0 };
        let tension = relation.border_tension * 0.35;
        let opportunity = (1.0 - target_nation.stability / 100.0) * 20.0;
        let weakness = (1.0 - ratio).clamp(0.0, 1.0) * 30.0;
        let aggression =
            profile.aggression * 30.0 + profile.revanchism * 15.0 + profile.opportunism * 10.0;
        let caution =
            profile.caution * ratio * 25.0 + profile.isolationism * 20.0 + own_exhaustion * 0.3;
        let honor = if target_nation
            .allies
            .iter()
            .any(|ally| sim.nations.iter().any(|nation| nation.id == *ally))
        {
            profile.honor * 10.0
        } else {
            0.0
        };
        let utility = claim_score + tension + opportunity + weakness + aggression - caution - honor;
        let difficulty_factor = match sim.difficulty {
            crate::scenario::Difficulty::Peaceful => 1.2,
            crate::scenario::Difficulty::Normal => 1.0,
            crate::scenario::Difficulty::Aggressive => 0.9,
            crate::scenario::Difficulty::Nightmare => 0.8,
        };
        let threshold = profile.war_threshold * 100.0 * difficulty_factor;
        if utility >= threshold && utility > 20.0 {
            let candidate = (utility, target, claimed);
            let replace = match best.as_ref() {
                Some((current, _, _)) => utility > *current,
                None => true,
            };
            if replace {
                best = Some(candidate);
            }
        }
    }
    if let Some((utility, target, province)) = best {
        if let Ok(war) = declare_war(sim, nation_id, &target, province.as_ref(), events) {
            events.push(SimEvent::nation(
                sim.tick,
                sim.date,
                SimEventKind::Diplomacy,
                nation_id,
                format!("Décision de guerre : utilité {utility:.1}, guerre {war}."),
            ));
        }
    }
}

fn claimed_province(
    sim: &Simulation<'_>,
    attacker: &NationId,
    defender: &NationId,
) -> Option<ProvinceId> {
    sim.nations
        .iter()
        .find(|nation| &nation.id == attacker)
        .and_then(|nation| {
            nation.claims.iter().find_map(|claim| {
                sim.world
                    .province(&claim.province)
                    .filter(|province| province.owner.as_ref() == Some(defender))
                    .map(|province| province.id.clone())
            })
        })
}

fn consider_missile(sim: &mut Simulation<'_>, nation_id: &NationId, events: &mut Vec<SimEvent>) {
    if !is_at_war(sim, nation_id) {
        return;
    }
    let recent = sim.missiles.iter().any(|missile| {
        &missile.owner == nation_id
            && sim
                .tick
                .as_u64()
                .saturating_sub(missile.launch_tick.as_u64())
                < 60 * u64::from(sim.ticks_per_day)
    });
    if recent {
        return;
    }
    let Some(nation) = sim.nations.iter().find(|nation| &nation.id == nation_id) else {
        return;
    };
    let missile_type = sim
        .data
        .missiles
        .iter()
        .find(|missile| nation.military.missile_count(&missile.id) > 0)
        .map(|missile| missile.id.clone());
    let Some(missile_type) = missile_type else {
        return;
    };
    let enemy_province = sim
        .world
        .provinces
        .iter()
        .filter(|province| {
            province.owner.as_ref().is_some_and(|owner| {
                owner != nation_id
                    && sim.wars.iter().any(|war| {
                        war.is_active()
                            && war.has_belligerent(nation_id)
                            && war.has_belligerent(owner)
                    })
            })
        })
        .filter(|province| {
            sim.nations
                .iter()
                .find(|nation| nation.id == *nation_id)
                .is_some_and(|nation| nation.capital != province.id)
        })
        .min_by(|left, right| left.id.cmp(&right.id))
        .map(|province| province.id.clone());
    let Some(enemy_province) = enemy_province else {
        return;
    };
    let Some(nation) = sim.nations.iter().find(|nation| &nation.id == nation_id) else {
        return;
    };
    let Some(distance) = range_between(sim, &nation.capital, &enemy_province) else {
        return;
    };
    let Some(definition) = sim.data.missile(&missile_type) else {
        return;
    };
    if distance > definition.range_km {
        return;
    }
    let _ = launch(sim, nation_id, &missile_type, &enemy_province, events);
}
