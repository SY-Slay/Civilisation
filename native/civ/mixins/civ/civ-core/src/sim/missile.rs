//! Vol, interception et impact des missiles.

use crate::error::{CoreError, CoreResult};
use crate::ids::{MissileInstanceId, MissileTypeId, NationId, ProvinceId};
use crate::math::{Vec2, Vec3};
use crate::missile::{
    DamageReport, Interceptor, InterceptorResult, MissileInstance, MissileStatus, MissileTarget,
    MissileType, TargetKind, Trajectory, WarheadKind,
};
use crate::sim::{clamp, next_id, province_center, SimEvent, SimEventKind, Simulation};

/// Lance un missile depuis la capitale de la nation émettrice.
pub fn launch(
    sim: &mut Simulation<'_>,
    owner: &NationId,
    missile_type: &MissileTypeId,
    target_province: &ProvinceId,
    events: &mut Vec<SimEvent>,
) -> CoreResult<MissileInstanceId> {
    let definition = sim
        .data
        .missile(missile_type)
        .cloned()
        .ok_or_else(|| CoreError::data(format!("type de missile inconnu : {missile_type}")))?;
    let owner_index = sim
        .nations
        .iter()
        .position(|nation| &nation.id == owner)
        .ok_or_else(|| CoreError::data(format!("nation inconnue : {owner}")))?;
    if !sim.nations[owner_index].is_alive() {
        return Err(CoreError::data(
            "une nation sans territoire ne peut pas tirer",
        ));
    }
    let launcher = sim.nations[owner_index].capital.clone();
    let launcher_center = province_center(sim, &launcher)
        .ok_or_else(|| CoreError::data("capitale sans centre de province"))?;
    let target_center = province_center(sim, target_province)
        .ok_or_else(|| CoreError::data(format!("province cible inconnue : {target_province}")))?;
    let distance_km = launcher_center.distance(target_center) / 1000.0;
    if distance_km > definition.range_km {
        return Err(CoreError::data(format!(
            "cible hors de portée : {:.0} km pour {:.0} km de portée",
            distance_km, definition.range_km
        )));
    }
    let stock = sim.nations[owner_index]
        .military
        .missile_stockpile
        .get_mut(missile_type)
        .ok_or_else(|| CoreError::data(format!("aucun stock pour {missile_type}")))?;
    if *stock == 0 {
        return Err(CoreError::data(format!("stock épuisé pour {missile_type}")));
    }
    *stock -= 1;

    let flight_hours = (distance_km / definition.speed_kmh).max(0.05);
    let flight_days = (flight_hours / 24.0).ceil().max(1.0) as u64;
    let arrival_tick = sim
        .tick
        .saturating_add(flight_days.saturating_mul(u64::from(sim.ticks_per_day)));
    let id = MissileInstanceId::new(next_id(
        "missile",
        sim.missiles.iter().map(|missile| missile.id.to_string()),
    ))?;
    let midpoint = launcher_center.lerp(target_center, 0.5);
    let apogee_km = (distance_km * 0.08).max(5.0);
    let trajectory = Trajectory {
        points: vec![
            Vec3::new(launcher_center.x(), 64.0, launcher_center.z()),
            Vec3::new(midpoint.x(), 64.0 + apogee_km * 1000.0, midpoint.z()),
            Vec3::new(target_center.x(), 64.0, target_center.z()),
        ],
        apogee_km: Some(apogee_km),
        terminal_speed_kmh: Some(definition.speed_kmh),
    };
    let detected_by = sim
        .world
        .province(target_province)
        .and_then(|province| province.owner.clone())
        .into_iter()
        .collect();
    let instance = MissileInstance {
        id: id.clone(),
        missile_type: definition.id.clone(),
        owner: owner.clone(),
        launcher_province: launcher,
        target: MissileTarget {
            kind: TargetKind::Province,
            province: target_province.clone(),
            position: Some(Vec3::new(target_center.x(), 64.0, target_center.z())),
            entity_id: None,
        },
        launch_tick: sim.tick,
        arrival_tick,
        status: MissileStatus::InFlight,
        trajectory: Some(trajectory),
        interceptors: Vec::new(),
        damage_report: None,
        detected_by,
    };
    sim.missiles.push(instance);
    if let Some(nation) = sim.nations.iter_mut().find(|nation| &nation.id == owner) {
        nation.missiles.push(id.clone());
    }
    events.push(SimEvent {
        tick: sim.tick,
        date: sim.date,
        kind: SimEventKind::Missile,
        message: format!(
            "{} lance un missile {} vers {}.",
            owner,
            definition.name.resolve(),
            target_province
        ),
        nation: Some(owner.clone()),
        other: sim
            .world
            .province(target_province)
            .and_then(|province| province.owner.clone()),
        war: None,
        province: Some(target_province.clone()),
    });
    Ok(id)
}

/// Résout les missiles arrivés à échéance.
pub(crate) fn tick(sim: &mut Simulation<'_>, events: &mut Vec<SimEvent>) {
    let mut index = 0;
    while index < sim.missiles.len() {
        let mut missile = sim.missiles[index].clone();
        if !missile.is_in_flight() {
            index += 1;
            continue;
        }
        if sim.tick >= missile.arrival_tick {
            resolve_missile(sim, &mut missile, events);
            let owner = missile.owner.clone();
            let missile_id = missile.id.clone();
            let in_flight = missile.is_in_flight();
            sim.missiles[index] = missile;
            if !in_flight {
                if let Some(nation) = sim.nations.iter_mut().find(|nation| nation.id == owner) {
                    nation.missiles.retain(|id| id != &missile_id);
                }
            }
        }
        index += 1;
    }
}

fn resolve_missile(
    sim: &mut Simulation<'_>,
    missile: &mut MissileInstance,
    events: &mut Vec<SimEvent>,
) {
    let Some(definition) = sim.data.missile(&missile.missile_type).cloned() else {
        missile.status = MissileStatus::Failed;
        events.push(SimEvent::nation(
            sim.tick,
            sim.date,
            SimEventKind::Missile,
            &missile.owner,
            format!("Missile {} perdu : définition absente.", missile.id),
        ));
        return;
    };
    let defender = sim
        .world
        .province(&missile.target.province)
        .and_then(|province| {
            province
                .controller
                .clone()
                .or_else(|| province.owner.clone())
        });
    if let Some(defender) = &defender {
        let readiness = sim
            .nations
            .iter()
            .find(|nation| &nation.id == defender)
            .map(|nation| nation.military.readiness)
            .unwrap_or(30.0);
        let base = definition
            .interception
            .as_ref()
            .and_then(|profile| profile.base_chance)
            .unwrap_or(0.12);
        let terminal_penalty = if definition
            .interception
            .as_ref()
            .is_some_and(|profile| profile.terminal_maneuver)
        {
            0.75
        } else {
            1.0
        };
        let chance = clamp(
            (base * (1.0 - definition.defense_penetration * 0.6) + readiness / 1000.0)
                * terminal_penalty,
            0.0,
            0.85,
        );
        let result = if sim.rng.chance(chance) {
            InterceptorResult::Hit
        } else {
            InterceptorResult::Miss
        };
        missile.interceptors.push(Interceptor {
            by: defender.clone(),
            tick: sim.tick,
            chance,
            result,
        });
        if result == InterceptorResult::Hit {
            missile.status = MissileStatus::Intercepted;
            events.push(SimEvent {
                tick: sim.tick,
                date: sim.date,
                kind: SimEventKind::Missile,
                message: format!(
                    "Missile {} intercepté par {} avant {}.",
                    missile.id, defender, missile.target.province
                ),
                nation: Some(defender.clone()),
                other: Some(missile.owner.clone()),
                war: None,
                province: Some(missile.target.province.clone()),
            });
            return;
        }
    }

    let province = sim.world.province_mut(&missile.target.province);
    let Some(province) = province else {
        missile.status = MissileStatus::Failed;
        return;
    };
    let warhead = &definition.warhead;
    let total = province.population.total as f64;
    let casualty_rate =
        (0.001 + warhead.blast_radius_blocks / 20_000.0 + warhead.yield_kt / 20_000.0)
            .clamp(0.0005, 0.08);
    let casualties = ((total * casualty_rate).round() as u64).min(province.population.total);
    province.population.total = province.population.total.saturating_sub(casualties);
    province.population.urban = province.population.urban.saturating_sub(casualties / 2);
    province.population.rural = province.population.rural.saturating_sub(casualties / 2);
    province.infrastructure = clamp(
        province.infrastructure
            - (4.0 + warhead.blast_radius_blocks / 10.0 + warhead.yield_kt / 10.0),
        0.0,
        100.0,
    );
    province.unrest = clamp(province.unrest + 12.0, 0.0, 100.0);
    let escalation = clamp(
        warhead.escalation_delta
            + if warhead.kind == WarheadKind::Nuclear {
                20.0
            } else {
                0.0
            },
        0.0,
        100.0,
    );
    let report = DamageReport {
        casualties,
        infrastructure_damage: clamp(
            4.0 + warhead.blast_radius_blocks / 10.0 + warhead.yield_kt / 10.0,
            0.0,
            100.0,
        ),
        provinces_affected: vec![missile.target.province.clone()],
        escalation,
        notes: Some(crate::text::LocalizedText::plain(
            "Rapport d'impact calculé par le cœur déterministe.",
        )),
    };
    missile.damage_report = Some(report.clone());
    missile.status = MissileStatus::Impact;
    if let Some(owner) = sim
        .nations
        .iter_mut()
        .find(|nation| nation.id == missile.owner)
    {
        owner.war_exhaustion = clamp(owner.war_exhaustion + escalation * 0.2, 0.0, 100.0);
    }
    if let Some(target_owner) = province.owner.clone() {
        if let Some(nation) = sim
            .nations
            .iter_mut()
            .find(|nation| nation.id == target_owner)
        {
            nation.stability = clamp(nation.stability - escalation * 0.15, 0.0, 100.0);
            nation.unrest = clamp(nation.unrest + escalation * 0.2, 0.0, 100.0);
        }
    }
    events.push(SimEvent {
        tick: sim.tick,
        date: sim.date,
        kind: SimEventKind::Missile,
        message: format!(
            "Impact de {} sur {} : {} victime(s), escalade {:.0}%.",
            missile.id, missile.target.province, casualties, escalation
        ),
        nation: Some(missile.owner.clone()),
        other: defender,
        war: None,
        province: Some(missile.target.province.clone()),
    });
}

/// Distance approximative entre deux provinces, en kilomètres fictifs.
pub fn range_between(sim: &Simulation<'_>, first: &ProvinceId, second: &ProvinceId) -> Option<f64> {
    let a: Vec2 = province_center(sim, first)?;
    let b: Vec2 = province_center(sim, second)?;
    Some(a.distance(b) / 1000.0)
}

/// Recherche un type de missile par identifiant.
pub fn missile_type<'a>(sim: &'a Simulation<'_>, id: &MissileTypeId) -> Option<&'a MissileType> {
    sim.data.missile(id)
}
