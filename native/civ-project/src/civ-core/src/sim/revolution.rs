//! Progression des révolutions et des guerres civiles.

use crate::ids::{FactionId, IdeologyId, NationId, RevolutionId};
use crate::revolution::{
    Faction, FactionDemand, GovernmentResponse, GovernmentStrategy, ProvinceControl,
    ProvinceSupport, Revolution, RevolutionCause, RevolutionOutcome, RevolutionOutcomeKind,
    RevolutionStatus,
};
use crate::sim::{clamp, is_at_war, next_id, SimEvent, SimEventKind, Simulation};
use crate::text::LocalizedText;

/// Fait progresser les révolutions et en déclenche de nouvelles.
pub(crate) fn tick(sim: &mut Simulation<'_>, events: &mut Vec<SimEvent>) {
    let nation_ids: Vec<NationId> = sim
        .nations
        .iter()
        .filter(|nation| nation.is_alive())
        .map(|nation| nation.id.clone())
        .collect();
    for nation_id in nation_ids {
        let active_index = sim
            .revolutions
            .iter()
            .position(|revolution| revolution.nation == nation_id && revolution.is_active());
        if let Some(index) = active_index {
            process_revolution(sim, index, events);
        } else if should_start(sim, &nation_id) {
            start_revolution(sim, &nation_id, events);
        }
    }
}

fn should_start(sim: &Simulation<'_>, nation_id: &NationId) -> bool {
    let Some(nation) = sim.nations.iter().find(|nation| &nation.id == nation_id) else {
        return false;
    };
    if !nation.is_alive() {
        return false;
    }
    let recent = sim.revolutions.iter().any(|revolution| {
        &revolution.nation == nation_id
            && revolution
                .end_date
                .is_some_and(|end| sim.date.days_since(end) < 720)
    });
    if recent {
        return false;
    }
    let war = is_at_war(sim, nation_id);
    let pressure = (100.0 - nation.stability) * 0.45
        + nation.unrest * 0.35
        + nation.war_exhaustion * 0.25
        + (100.0 - nation.legitimacy) * 0.15
        + if war { 8.0 } else { 0.0 };
    pressure >= 42.0 && (war || nation.unrest >= 45.0 || nation.stability <= 35.0)
}

pub(crate) fn start_revolution(
    sim: &mut Simulation<'_>,
    nation_id: &NationId,
    events: &mut Vec<SimEvent>,
) {
    let Some(nation) = sim.nations.iter().find(|nation| &nation.id == nation_id) else {
        return;
    };
    let id = match RevolutionId::new(next_id(
        "rev",
        sim.revolutions
            .iter()
            .map(|revolution| revolution.id.to_string()),
    )) {
        Ok(value) => value,
        Err(_) => return,
    };
    let faction_id = match FactionId::new(next_id(
        "faction",
        sim.revolutions.iter().flat_map(|revolution| {
            revolution
                .factions
                .iter()
                .map(|faction| faction.id.to_string())
        }),
    )) {
        Ok(value) => value,
        Err(_) => return,
    };
    let support = clamp(
        (nation.unrest * 0.6 + (100.0 - nation.stability) * 0.4) / 100.0,
        0.2,
        0.9,
    );
    let mut causes = Vec::new();
    if is_at_war(sim, nation_id) {
        causes.push(RevolutionCause::WarExhaustion);
    }
    if nation.unrest > 55.0 {
        causes.push(RevolutionCause::EconomicCrisis);
    }
    if nation.legitimacy < 40.0 {
        causes.push(RevolutionCause::Corruption);
    }
    if causes.is_empty() {
        causes.push(RevolutionCause::Inequality);
    }
    let mut support_by_province = Vec::new();
    for province_id in &nation.provinces {
        let province_support = sim
            .world
            .province(province_id)
            .map(|province| {
                clamp(
                    province.unrest / 100.0 * 0.7 + (100.0 - nation.stability) / 100.0 * 0.3,
                    0.05,
                    0.95,
                )
            })
            .unwrap_or(0.1);
        support_by_province.push(ProvinceSupport {
            province: province_id.clone(),
            support: province_support,
            control: ProvinceControl::Government,
        });
    }
    let strategy = if nation.ai_profile.aggression > 0.65 {
        GovernmentStrategy::Suppress
    } else if nation.ai_profile.honor > 0.5 {
        GovernmentStrategy::Negotiate
    } else {
        GovernmentStrategy::Reform
    };
    let status = if support >= 0.55 {
        RevolutionStatus::Uprising
    } else {
        RevolutionStatus::Agitation
    };
    let revolution = Revolution {
        schema_version: 1,
        id: id.clone(),
        nation: nation_id.clone(),
        start_date: sim.date,
        end_date: None,
        status,
        causes,
        factions: vec![Faction {
            id: faction_id,
            name: LocalizedText::plain(format!(
                "Mouvement populaire de {}",
                nation.display_short_name()
            )),
            ideology: IdeologyId::new("popular_front").unwrap_or_else(|_| nation.ideology.clone()),
            support,
            demands: vec![FactionDemand::FreeElections, FactionDemand::LandReform],
            leader: None,
            militia_size: (nation.population.total as f64 * support * 0.002) as u64,
            legitimacy: clamp((100.0 - nation.legitimacy) / 100.0, 0.1, 0.9),
            foreign_support: Vec::new(),
        }],
        support_by_province,
        controlled_provinces: Vec::new(),
        government_response: GovernmentResponse {
            strategy,
            intensity: 0.5,
            started_date: Some(sim.date),
            offered_terms: Vec::new(),
        },
        casualties: 0,
        foreign_interventions: Vec::new(),
        outcome: None,
        tags: vec!["generated".to_owned()],
    };
    sim.revolutions.push(revolution);
    if let Some(nation) = sim
        .nations
        .iter_mut()
        .find(|nation| &nation.id == nation_id)
    {
        nation.revolutions.push(id.clone());
        nation.unrest = clamp(nation.unrest + 5.0, 0.0, 100.0);
    }
    events.push(SimEvent::nation(
        sim.tick,
        sim.date,
        SimEventKind::Revolution,
        nation_id,
        format!("Une révolution commence : {}", id),
    ));
}

fn process_revolution(sim: &mut Simulation<'_>, index: usize, events: &mut Vec<SimEvent>) {
    let mut revolution = sim.revolutions[index].clone();
    if !revolution.is_active() {
        return;
    }
    let Some(nation) = sim
        .nations
        .iter()
        .find(|nation| nation.id == revolution.nation)
    else {
        return;
    };
    let average_support = if revolution.support_by_province.is_empty() {
        revolution
            .factions
            .iter()
            .map(|faction| faction.support)
            .sum::<f64>()
            / revolution.factions.len().max(1) as f64
    } else {
        revolution
            .support_by_province
            .iter()
            .map(|support| support.support)
            .sum::<f64>()
            / revolution.support_by_province.len() as f64
    };
    let war_pressure = if is_at_war(sim, &revolution.nation) {
        0.01
    } else {
        -0.005
    };
    let suppression = if revolution.government_response.strategy == GovernmentStrategy::Suppress
        || revolution.government_response.strategy == GovernmentStrategy::MartialLaw
    {
        revolution.government_response.intensity * 0.02
    } else {
        0.0
    };
    let support_delta = 0.005 + war_pressure + (nation.unrest / 100.0) * 0.01 - suppression;
    for faction in &mut revolution.factions {
        faction.support = clamp(faction.support + support_delta, 0.0, 1.0);
        faction.legitimacy = clamp(faction.legitimacy + support_delta * 0.5, 0.0, 1.0);
    }
    for support in &mut revolution.support_by_province {
        support.support = clamp(support.support + support_delta, 0.0, 1.0);
        support.control = if support.support >= 0.65 {
            ProvinceControl::Revolution
        } else if support.support >= 0.4 {
            ProvinceControl::Contested
        } else {
            ProvinceControl::Government
        };
    }
    revolution.controlled_provinces = revolution
        .support_by_province
        .iter()
        .filter(|support| support.control == ProvinceControl::Revolution)
        .map(|support| support.province.clone())
        .collect();
    revolution.government_response.intensity = clamp(
        revolution.government_response.intensity
            + if nation.stability < 30.0 {
                0.005
            } else {
                -0.002
            },
        0.0,
        1.0,
    );
    revolution.casualties = revolution
        .casualties
        .saturating_add((average_support * 20.0) as u64);

    let age = sim.date.days_since(revolution.start_date);
    revolution.status = match revolution.status {
        RevolutionStatus::Simmering if age >= 20 => RevolutionStatus::Agitation,
        RevolutionStatus::Agitation if age >= 45 || average_support >= 0.55 => {
            RevolutionStatus::Uprising
        }
        RevolutionStatus::Uprising if age >= 75 || average_support >= 0.65 => {
            RevolutionStatus::CivilWar
        }
        status => status,
    };

    let end = if revolution.status == RevolutionStatus::CivilWar
        && average_support >= 0.72
        && nation.stability < 35.0
    {
        Some(RevolutionOutcomeKind::NewGovernment)
    } else if average_support <= 0.18
        && revolution.government_response.intensity >= 0.65
        && age >= 90
    {
        Some(RevolutionOutcomeKind::Crushed)
    } else if age >= 365 {
        Some(if average_support >= 0.5 {
            RevolutionOutcomeKind::Reform
        } else {
            RevolutionOutcomeKind::StatusQuo
        })
    } else {
        None
    };

    if let Some(kind) = end {
        revolution.end_date = Some(sim.date);
        revolution.status = match kind {
            RevolutionOutcomeKind::Crushed | RevolutionOutcomeKind::StatusQuo => {
                RevolutionStatus::Crushed
            }
            RevolutionOutcomeKind::NewGovernment => RevolutionStatus::Victory,
            _ => RevolutionStatus::Reform,
        };
        revolution.outcome = Some(RevolutionOutcome {
            kind,
            new_government: None,
            new_ideology: None,
            terms: Vec::new(),
            recognized_by: Vec::new(),
        });
        if let Some(nation) = sim
            .nations
            .iter_mut()
            .find(|nation| nation.id == revolution.nation)
        {
            nation.revolutions.retain(|id| id != &revolution.id);
            match kind {
                RevolutionOutcomeKind::NewGovernment => {
                    nation.stability = clamp(nation.stability + 12.0, 0.0, 100.0);
                    nation.legitimacy = clamp(nation.legitimacy + 8.0, 0.0, 100.0);
                    nation.unrest = clamp(nation.unrest - 15.0, 0.0, 100.0);
                }
                RevolutionOutcomeKind::Reform => {
                    nation.stability = clamp(nation.stability + 6.0, 0.0, 100.0);
                    nation.unrest = clamp(nation.unrest - 8.0, 0.0, 100.0);
                }
                RevolutionOutcomeKind::Crushed | RevolutionOutcomeKind::StatusQuo => {
                    nation.stability = clamp(nation.stability - 3.0, 0.0, 100.0);
                    nation.unrest = clamp(nation.unrest + 4.0, 0.0, 100.0);
                }
                _ => {}
            }
        }
        events.push(SimEvent::nation(
            sim.tick,
            sim.date,
            SimEventKind::Revolution,
            &revolution.nation,
            format!("Révolution {} : {:?}.", revolution.id, kind),
        ));
    }

    sim.revolutions[index] = revolution;
}
