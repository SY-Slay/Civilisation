//! Économie, population, stabilité et recherche.
//!
//! Les formules sont volontairement simples et déterministes. Elles ne
//! cherchent pas à simuler une économie réelle : elles produisent des ordres de
//! grandeur stables, lisibles et faciles à équilibrer par les données.

use std::collections::BTreeMap;

use crate::data::GameData;
use crate::data::TechnologyCategory;
use crate::diplomacy::SanctionStatus;
use crate::ids::NationId;
use crate::nation::technology::ResearchDomainKind;
use crate::nation::Nation;
use crate::sim::{clamp, daily_rate, SimEvent, SimEventKind, Simulation};
use crate::time::{GameDate, Tick};

/// Applique un jour d'évolution économique et sociale.
pub(crate) fn tick(sim: &mut Simulation<'_>, events: &mut Vec<SimEvent>) {
    let war_flags: BTreeMap<NationId, bool> = sim
        .nations
        .iter()
        .map(|nation| {
            let at_war = sim
                .wars
                .iter()
                .any(|war| war.is_active() && war.has_belligerent(&nation.id));
            (nation.id.clone(), at_war)
        })
        .collect();
    let sanction_severity: BTreeMap<NationId, f64> = sim
        .nations
        .iter()
        .map(|nation| {
            let severity = sim
                .diplomacy
                .sanctions
                .iter()
                .filter(|sanction| {
                    sanction.status == SanctionStatus::Active && sanction.targets(&nation.id)
                })
                .map(|sanction| sanction.severity)
                .sum::<f64>()
                .clamp(0.0, 3.0);
            (nation.id.clone(), severity)
        })
        .collect();
    let trade_volume: BTreeMap<NationId, f64> = sim
        .diplomacy
        .trade_routes
        .iter()
        .filter(|route| route.is_open())
        .fold(BTreeMap::new(), |mut map, route| {
            *map.entry(route.a.clone()).or_insert(0.0) += route.volume;
            *map.entry(route.b.clone()).or_insert(0.0) += route.volume;
            map
        });

    let data = sim.data;
    let tick = sim.tick;
    let date = sim.date;
    for nation in &mut sim.nations {
        if !nation.is_alive() {
            continue;
        }
        nation.technology.ensure_domains();
        let at_war = war_flags.get(&nation.id).copied().unwrap_or(false);
        let sanctions = sanction_severity.get(&nation.id).copied().unwrap_or(0.0);
        let trade = trade_volume.get(&nation.id).copied().unwrap_or(0.0);

        update_population(nation);
        update_economy(nation, at_war, sanctions, trade);
        update_stability(nation, at_war, sanctions);
        update_research(data, tick, date, nation, events);
    }

    update_provinces(sim);
    recompute_national_population(sim);
}

fn update_population(nation: &mut Nation) {
    let annual = clamp(nation.population.growth_rate, -0.2, 0.2)
        + if nation.stability < 35.0 { -0.004 } else { 0.0 }
        + if nation.unrest > 65.0 { -0.003 } else { 0.0 };
    let growth = daily_rate(annual);
    let total = nation.population.total as f64;
    let next_total = (total * (1.0 + growth)).max(1.0);
    nation.population.total = next_total.round().max(1.0) as u64;

    let workforce_ratio = if total > 0.0 {
        nation.population.workforce as f64 / total
    } else {
        0.52
    };
    let conscription_ratio = if total > 0.0 {
        nation.population.conscription_pool as f64 / total
    } else {
        0.02
    };
    nation.population.workforce = (nation.population.total as f64 * workforce_ratio).round() as u64;
    nation.population.conscription_pool =
        (nation.population.total as f64 * conscription_ratio).round() as u64;
    nation.population.workforce = nation.population.workforce.min(nation.population.total);
    nation.population.conscription_pool = nation
        .population
        .conscription_pool
        .min(nation.population.total);
}

fn update_economy(nation: &mut Nation, at_war: bool, sanctions: f64, trade: f64) {
    let stability = (nation.stability - 50.0) / 100.0;
    let unrest = nation.unrest / 100.0;
    let tech = nation.technology.researched.len() as f64 * 0.0008;
    let war_penalty = if at_war { 0.025 } else { 0.0 };
    let sanction_penalty = sanctions * 0.02;
    let trade_bonus = (trade / 10_000_000.0).clamp(0.0, 0.01);
    let annual_growth =
        0.012 + tech + stability * 0.02 - unrest * 0.02 - war_penalty - sanction_penalty
            + trade_bonus;
    let growth = daily_rate(annual_growth.clamp(-0.15, 0.12));
    nation.economy.gdp = (nation.economy.gdp * (1.0 + growth)).max(1.0);

    let military_ratio = nation.military.army_size as f64 / nation.population.total.max(1) as f64;
    let spending_rate =
        0.17 + (military_ratio * 1.5).clamp(0.0, 0.08) + if at_war { 0.04 } else { 0.0 };
    let income = nation.economy.gdp * nation.economy.tax_rate / 360.0;
    let spending = nation.economy.gdp * spending_rate / 360.0;
    let balance = income - spending;
    if balance >= 0.0 {
        nation.economy.treasury += balance;
        if nation.economy.debt > 0.0 {
            let payment = (nation.economy.treasury * 0.02).min(nation.economy.debt);
            nation.economy.treasury -= payment;
            nation.economy.debt -= payment;
        }
    } else {
        let deficit = -balance;
        if nation.economy.treasury >= deficit {
            nation.economy.treasury -= deficit;
        } else {
            nation.economy.debt += deficit - nation.economy.treasury;
            nation.economy.treasury = 0.0;
        }
    }

    let target_inflation = 0.02 + if at_war { 0.015 } else { 0.0 } + sanctions * 0.02;
    nation.economy.inflation += (target_inflation - nation.economy.inflation) * 0.01;
    nation.economy.inflation = clamp(nation.economy.inflation, -0.5, 5.0);

    let target_unemployment = 0.05 + if at_war { 0.02 } else { 0.0 } + nation.unrest / 100.0 * 0.03;
    nation.economy.unemployment += (target_unemployment - nation.economy.unemployment) * 0.01;
    nation.economy.unemployment = clamp(nation.economy.unemployment, 0.0, 1.0);
    nation.economy.trade_balance = trade - sanctions * nation.economy.gdp * 0.01;
}

fn update_stability(nation: &mut Nation, at_war: bool, sanctions: f64) {
    let exhaustion = nation.war_exhaustion;
    let treasury_health = clamp(
        nation.economy.treasury / (nation.economy.gdp * 0.08).max(1.0),
        0.0,
        1.0,
    );
    let target_stability = 48.0 + (nation.legitimacy - 50.0) * 0.45 + treasury_health * 6.0
        - nation.unrest * 0.42
        - exhaustion * 0.22
        - sanctions * 12.0
        + if at_war { -4.0 } else { 2.0 };
    nation.stability += (target_stability - nation.stability).clamp(-0.12, 0.12);
    nation.stability = clamp(nation.stability, 0.0, 100.0);

    let target_legitimacy =
        50.0 + (nation.stability - 50.0) * 0.35 - exhaustion * 0.15 + nation.prestige * 0.08;
    nation.legitimacy += (target_legitimacy - nation.legitimacy).clamp(-0.06, 0.06);
    nation.legitimacy = clamp(nation.legitimacy, 0.0, 100.0);

    let target_unrest = 10.0
        + (100.0 - nation.stability) * 0.45
        + exhaustion * 0.2
        + sanctions * 20.0
        + (100.0 - nation.legitimacy) * 0.1;
    nation.unrest += (target_unrest - nation.unrest).clamp(-0.1, 0.12);
    nation.unrest = clamp(nation.unrest, 0.0, 100.0);

    if at_war {
        nation.war_exhaustion += 0.035;
    } else {
        nation.war_exhaustion -= 0.025;
    }
    nation.war_exhaustion = clamp(nation.war_exhaustion, 0.0, 100.0);

    let target_prestige = if at_war {
        nation.prestige - 0.01
    } else {
        nation.prestige + 0.005
    };
    nation.prestige = clamp(target_prestige, 0.0, 100.0);

    if nation.stability < 35.0 {
        nation.military.readiness = clamp(nation.military.readiness - 0.01, 0.0, 100.0);
        nation.military.morale = clamp(nation.military.morale - 0.01, 0.0, 100.0);
    } else if !at_war {
        nation.military.readiness = clamp(nation.military.readiness + 0.005, 0.0, 100.0);
        nation.military.morale = clamp(nation.military.morale + 0.004, 0.0, 100.0);
    }
}

fn update_research(
    data: &GameData,
    tick: Tick,
    date: GameDate,
    nation: &mut Nation,
    events: &mut Vec<SimEvent>,
) {
    let Some(technology_id) = nation.technology.researching.clone() else {
        return;
    };
    let Some(definition) = data.technology(&technology_id) else {
        nation.technology.researching = None;
        return;
    };
    let research_gain = nation.technology.research_speed
        * (0.5 + nation.economy.gdp / 1_000_000_000.0).clamp(0.5, 4.0);
    nation.technology.research_points += research_gain;
    if nation.technology.research_points >= definition.cost {
        nation.technology.research_points -= definition.cost;
        nation.technology.unlock(technology_id.clone());
        nation.technology.era = nation.technology.era.max(definition.era);
        nation.technology.researching = None;
        let domain = match definition.category {
            TechnologyCategory::Military
            | TechnologyCategory::Naval
            | TechnologyCategory::Aerospace
            | TechnologyCategory::Missile
            | TechnologyCategory::Intelligence => ResearchDomainKind::Military,
            TechnologyCategory::Economy
            | TechnologyCategory::Industry
            | TechnologyCategory::Energy
            | TechnologyCategory::Medicine
            | TechnologyCategory::Society => ResearchDomainKind::Economy,
            TechnologyCategory::Computing => ResearchDomainKind::Technology,
        };
        nation.technology.ensure_domains();
        nation.technology.advance_domain(domain, definition.cost);
        events.push(SimEvent::nation(
            tick,
            date,
            SimEventKind::Technology,
            &nation.id,
            format!(
                "{} achève la recherche de {}.",
                nation.display_short_name(),
                definition.name.resolve()
            ),
        ));
    }
}

fn update_provinces(sim: &mut Simulation<'_>) {
    let national: BTreeMap<NationId, (f64, f64)> = sim
        .nations
        .iter()
        .map(|nation| (nation.id.clone(), (nation.stability, nation.unrest)))
        .collect();
    for province in &mut sim.world.provinces {
        let annual = clamp(province.population.growth_rate, -0.2, 0.2)
            + if province.unrest > 60.0 { -0.004 } else { 0.0 };
        let growth = daily_rate(annual);
        let total = province.population.total as f64;
        let next_total = (total * (1.0 + growth)).max(1.0);
        province.population.total = next_total.round().max(1.0) as u64;
        let urban_ratio = if total > 0.0 {
            province.population.urban as f64 / total
        } else {
            0.4
        };
        province.population.urban = (province.population.total as f64 * urban_ratio).round() as u64;
        province.population.rural = province
            .population
            .total
            .saturating_sub(province.population.urban);

        let (stability, nation_unrest) = province
            .owner
            .as_ref()
            .and_then(|owner| national.get(owner).copied())
            .unwrap_or((50.0, 20.0));
        let occupation_penalty = if province.is_occupied() { 8.0 } else { 0.0 };
        let target_unrest =
            8.0 + (100.0 - stability) * 0.35 + nation_unrest * 0.25 + occupation_penalty;
        province.unrest += (target_unrest - province.unrest).clamp(-0.08, 0.1);
        province.unrest = clamp(province.unrest, 0.0, 100.0);

        if province.unrest > 60.0 || province.is_occupied() {
            province.infrastructure = clamp(province.infrastructure - 0.01, 0.0, 100.0);
        } else {
            province.infrastructure = clamp(province.infrastructure + 0.003, 0.0, 100.0);
        }
        province.supply = clamp(province.supply + 0.05, 0.0, 100.0);
    }
}

fn recompute_national_population(sim: &mut Simulation<'_>) {
    for nation in &mut sim.nations {
        if !nation.is_alive() {
            continue;
        }
        let total: u64 = nation
            .provinces
            .iter()
            .filter_map(|province| sim.world.province(province))
            .map(|province| province.population.total)
            .sum();
        if total == 0 {
            continue;
        }
        let workforce_ratio =
            nation.population.workforce as f64 / nation.population.total.max(1) as f64;
        let conscription_ratio =
            nation.population.conscription_pool as f64 / nation.population.total.max(1) as f64;
        nation.population.total = total;
        nation.population.workforce = (total as f64 * workforce_ratio).round() as u64;
        nation.population.conscription_pool = (total as f64 * conscription_ratio).round() as u64;
    }
}
