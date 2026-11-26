//! Résolution des fronts, batailles, occupations et paix.

use crate::ids::{BattleId, NationId, ProvinceId, WarId};
use crate::sim::diplomacy::{end_war, relation_mut};
use crate::sim::{clamp, nation_index, nation_power, SimEvent, SimEventKind, Simulation};
use crate::war::{
    AssaultZoneStatus, Battle, BattleResult, BelligerentSide, FrontStatus, War, WarStatus,
};

/// Résout un jour de guerre pour toutes les guerres actives.
pub(crate) fn tick(sim: &mut Simulation<'_>, events: &mut Vec<SimEvent>) {
    let active: Vec<WarId> = sim
        .wars
        .iter()
        .filter(|war| war.is_active())
        .map(|war| war.id.clone())
        .collect();
    for war_id in active {
        let Some(index) = sim.wars.iter().position(|war| war.id == war_id) else {
            continue;
        };
        process_war(sim, index, events);
    }
}

fn process_war(sim: &mut Simulation<'_>, index: usize, events: &mut Vec<SimEvent>) {
    let mut war = sim.wars[index].clone();
    if !war.is_active() {
        return;
    }
    let Some(attacker) = war
        .belligerents
        .iter()
        .find(|belligerent| belligerent.side == BelligerentSide::Attacker)
        .map(|belligerent| belligerent.nation.clone())
    else {
        return;
    };
    let Some(defender) = war
        .belligerents
        .iter()
        .find(|belligerent| belligerent.side == BelligerentSide::Defender)
        .map(|belligerent| belligerent.nation.clone())
    else {
        return;
    };
    if nation_index(sim, &attacker).is_none() || nation_index(sim, &defender).is_none() {
        return;
    }

    for front_index in 0..war.fronts.len() {
        let front_id = war.fronts[front_index].id.clone();
        let ongoing = war.battles.iter().position(|battle| {
            battle.front.as_ref() == Some(&front_id) && battle.result == BattleResult::Ongoing
        });
        if let Some(battle_index) = ongoing {
            resolve_battle(sim, &mut war, front_index, battle_index, events);
        } else if sim.tick.as_u64() % (5 * u64::from(sim.ticks_per_day)) == 0 {
            start_battle(sim, &mut war, front_index);
        }
        if !war.is_active() {
            break;
        }
    }

    for belligerent in &mut war.belligerents {
        if belligerent.side == BelligerentSide::Attacker {
            belligerent.war_support = clamp(belligerent.war_support + 0.02, 0.0, 100.0);
        }
    }
    sim.wars[index] = war.clone();

    let duration = sim.date.days_since(war.start_date);
    let exhausted = war.belligerents.iter().all(|belligerent| {
        sim.nations
            .iter()
            .find(|nation| nation.id == belligerent.nation)
            .is_some_and(|nation| nation.war_exhaustion >= 25.0)
    });
    let decisive = (war.score.attacker - war.score.defender).abs() >= 35.0;
    if (duration >= 180 && (exhausted || decisive)) || duration >= 720 {
        let winner = if war.score.attacker >= war.score.defender {
            attacker.clone()
        } else {
            defender
        };
        if let Err(error) = end_war(sim, &war.id, &winner, events) {
            events.push(SimEvent::nation(
                sim.tick,
                sim.date,
                SimEventKind::System,
                &attacker,
                format!("Échec de la négociation de paix : {error}"),
            ));
        }
    }
}

fn start_battle(sim: &mut Simulation<'_>, war: &mut War, front_index: usize) {
    let front = &war.fronts[front_index];
    let target = front
        .provinces
        .iter()
        .find(|province| {
            sim.world
                .province(province)
                .is_some_and(|value| value.controller.as_ref() == Some(&front.defender))
        })
        .or_else(|| front.provinces.first())
        .cloned();
    let Some(province) = target else {
        return;
    };
    let terrain_modifier = sim
        .world
        .province(&province)
        .map(|value| 1.0 + value.terrain.defense_bonus() / 100.0)
        .unwrap_or(1.0);
    let battle = Battle {
        id: BattleId::new(format!("battle_{}_{}", war.id, war.battles.len() + 1))
            .unwrap_or_else(|_| BattleId::new("battle_generated").expect("identifiant valide")),
        front: Some(front.id.clone()),
        province,
        start_date: sim.date,
        end_date: None,
        attacker: front.attacker.clone(),
        defender: front.defender.clone(),
        result: BattleResult::Ongoing,
        attacker_losses: 0,
        defender_losses: 0,
        terrain_modifier,
        notes: None,
    };
    war.battles.push(battle);
}

fn resolve_battle(
    sim: &mut Simulation<'_>,
    war: &mut War,
    front_index: usize,
    battle_index: usize,
    events: &mut Vec<SimEvent>,
) {
    let (attacker, defender) = {
        let front = &war.fronts[front_index];
        (front.attacker.clone(), front.defender.clone())
    };
    let Some(attacker_nation) = sim.nations.iter().find(|nation| nation.id == attacker) else {
        return;
    };
    let Some(defender_nation) = sim.nations.iter().find(|nation| nation.id == defender) else {
        return;
    };
    let attacker_power = nation_power(attacker_nation);
    let defender_power = nation_power(defender_nation);
    let front = &war.fronts[front_index];
    let terrain = war.battles[battle_index].terrain_modifier;
    let attacker_strength = attacker_power
        * (1.0 + front.momentum / 200.0)
        * (front.supply / 100.0)
        * (0.85 + sim.rng.unit_f64() * 0.3);
    let defender_strength = defender_power
        * (1.0 - front.momentum / 200.0)
        * (front.supply / 100.0)
        * terrain
        * (0.85 + sim.rng.unit_f64() * 0.3);
    let difference = attacker_strength - defender_strength;
    let result = if difference > defender_strength * 0.08 {
        BattleResult::AttackerVictory
    } else if difference < -attacker_strength * 0.08 {
        BattleResult::DefenderVictory
    } else {
        BattleResult::Stalemate
    };
    let base_loss = ((attacker_power.min(defender_power) / 120.0).max(1.0)) as u64;
    let attacker_losses = match result {
        BattleResult::AttackerVictory => base_loss / 3,
        BattleResult::DefenderVictory => base_loss,
        BattleResult::Stalemate => base_loss / 2,
        _ => 0,
    };
    let defender_losses = match result {
        BattleResult::AttackerVictory => base_loss,
        BattleResult::DefenderVictory => base_loss / 3,
        BattleResult::Stalemate => base_loss / 2,
        _ => 0,
    };
    {
        let battle = &mut war.battles[battle_index];
        battle.result = result;
        battle.end_date = Some(sim.date);
        battle.attacker_losses = attacker_losses;
        battle.defender_losses = defender_losses;
    }
    let front = &mut war.fronts[front_index];
    match result {
        BattleResult::AttackerVictory => {
            front.momentum = clamp(front.momentum + 12.0, -100.0, 100.0);
            front.status = FrontStatus::Breakthrough;
            front.battles_won += 1;
            war.score.attacker += 5.0;
            war.score.defender = (war.score.defender - 1.0).max(0.0);
        }
        BattleResult::DefenderVictory => {
            front.momentum = clamp(front.momentum - 12.0, -100.0, 100.0);
            front.status = FrontStatus::Collapsed;
            front.battles_lost += 1;
            war.score.defender += 5.0;
            war.score.attacker = (war.score.attacker - 1.0).max(0.0);
        }
        BattleResult::Stalemate => {
            front.status = FrontStatus::Active;
        }
        _ => {}
    }
    front.supply = clamp(front.supply - 3.0, 10.0, 100.0);

    for nation in &mut sim.nations {
        if nation.id == attacker {
            nation.military.manpower = nation.military.manpower.saturating_sub(attacker_losses);
            nation.military.army_size = nation
                .military
                .army_size
                .saturating_sub(attacker_losses / 2);
            nation.military.morale = clamp(
                nation.military.morale
                    + if result == BattleResult::AttackerVictory {
                        1.0
                    } else {
                        -1.0
                    },
                0.0,
                100.0,
            );
        } else if nation.id == defender {
            nation.military.manpower = nation.military.manpower.saturating_sub(defender_losses);
            nation.military.army_size = nation
                .military
                .army_size
                .saturating_sub(defender_losses / 2);
            nation.military.morale = clamp(
                nation.military.morale
                    + if result == BattleResult::DefenderVictory {
                        1.0
                    } else {
                        -1.0
                    },
                0.0,
                100.0,
            );
        }
    }
    for belligerent in &mut war.belligerents {
        if belligerent.nation == attacker {
            belligerent.casualties = belligerent.casualties.saturating_add(attacker_losses);
        } else if belligerent.nation == defender {
            belligerent.casualties = belligerent.casualties.saturating_add(defender_losses);
        }
    }

    if result == BattleResult::AttackerVictory {
        let province = war.battles[battle_index].province.clone();
        if let Some(value) = sim.world.province_mut(&province) {
            if value.owner.as_ref().is_some_and(|owner| owner == &defender)
                && !sim
                    .nations
                    .iter()
                    .any(|nation| nation.id == defender && nation.capital == province)
            {
                value.controller = Some(attacker.clone());
                value.unrest = clamp(value.unrest + 12.0, 0.0, 100.0);
                if let Some(belligerent) = war
                    .belligerents
                    .iter_mut()
                    .find(|belligerent| belligerent.nation == attacker)
                {
                    if !belligerent.occupied_provinces.contains(&province) {
                        belligerent.occupied_provinces.push(province.clone());
                    }
                }
                events.push(SimEvent {
                    tick: sim.tick,
                    date: sim.date,
                    kind: SimEventKind::Occupation,
                    message: format!(
                        "{} occupe {} après une victoire sur le front.",
                        attacker, province
                    ),
                    nation: Some(attacker.clone()),
                    other: Some(defender.clone()),
                    war: Some(war.id.clone()),
                    province: Some(province.clone()),
                });
                if let Some(zone) = war
                    .assault_zones
                    .iter_mut()
                    .find(|zone| zone.province == province)
                {
                    zone.status = AssaultZoneStatus::Controlled;
                    zone.controller = Some(attacker.clone());
                    zone.attacker_progress = 100.0;
                    zone.defender_progress = 0.0;
                    zone.score = 10.0;
                    war.score.attacker += zone.score;
                }
            }
        }
    }

    let tick = sim.tick;
    if let Some(relation) = relation_mut(sim, &attacker, &defender) {
        relation.border_tension = 100.0;
        relation.last_change_tick = Some(tick);
    }
    if result != BattleResult::Stalemate {
        events.push(SimEvent {
            tick: sim.tick,
            date: sim.date,
            kind: SimEventKind::Battle,
            message: format!(
                "Bataille de {} : {:?} ({} / {} pertes).",
                war.battles[battle_index].province, result, attacker_losses, defender_losses
            ),
            nation: Some(attacker),
            other: Some(defender),
            war: Some(war.id.clone()),
            province: Some(war.battles[battle_index].province.clone()),
        });
    }
}

/// Vrai si une guerre est encore active (helper public pour l'adaptateur).
pub fn is_war_active(war: &War) -> bool {
    war.status == WarStatus::Active || war.status == WarStatus::Declared
}

/// Camp opposé à une nation dans une guerre.
pub fn opposing_side(war: &War, nation: &NationId) -> Option<BelligerentSide> {
    match war.side_of(nation)? {
        BelligerentSide::Attacker | BelligerentSide::CoBelligerent => {
            Some(BelligerentSide::Defender)
        }
        BelligerentSide::Defender => Some(BelligerentSide::Attacker),
    }
}

/// Province cible la plus probable d'une guerre.
pub fn target_province(war: &War) -> Option<&ProvinceId> {
    war.war_goals
        .iter()
        .find_map(|goal| goal.provinces.first())
        .or_else(|| war.fronts.first().and_then(|front| front.provinces.first()))
}
