//! Actions diplomatiques actives : traités, sanctions et guerres.

use std::collections::BTreeSet;

use crate::diplomacy::{
    RelationStatus, Sanction, SanctionKind, SanctionStatus, Treaty, TreatyKind, TreatyStatus,
};
use crate::error::{CoreError, CoreResult};
use crate::ids::{AssaultZoneId, NationId, TreatyId, WarGoalId, WarId};
use crate::sim::{next_id, SimEvent, SimEventKind, Simulation};
use crate::text::LocalizedText;
use crate::war::{
    AssaultZone, AssaultZoneStatus, Belligerent, BelligerentSide, CasusBelli, CasusBelliKind,
    Front, FrontStatus, War, WarGoal, WarGoalKind, WarScore, WarStatus,
};

/// Vrai si un traité actif bloque une déclaration de guerre.
pub(crate) fn has_blocking_treaty(
    sim: &Simulation<'_>,
    first: &NationId,
    second: &NationId,
) -> bool {
    sim.diplomacy.treaties.iter().any(|treaty| {
        treaty.is_active()
            && treaty.has_party(first)
            && treaty.has_party(second)
            && matches!(
                treaty.kind,
                TreatyKind::Peace
                    | TreatyKind::NonAggression
                    | TreatyKind::Alliance
                    | TreatyKind::Defensive
                    | TreatyKind::Ceasefire
            )
    })
}

/// Relation mutable entre deux nations.
pub(crate) fn relation_mut<'b>(
    sim: &'b mut Simulation<'_>,
    first: &NationId,
    second: &NationId,
) -> Option<&'b mut crate::diplomacy::Relation> {
    sim.diplomacy
        .relations
        .iter_mut()
        .find(|relation| relation.involves(first) && relation.involves(second))
}

/// Signe un traité actif entre plusieurs nations.
pub(crate) fn sign_treaty(
    sim: &mut Simulation<'_>,
    kind: TreatyKind,
    parties: &[NationId],
    duration_days: Option<u32>,
    events: &mut Vec<SimEvent>,
) -> CoreResult<TreatyId> {
    let mut unique = BTreeSet::new();
    for party in parties {
        if !sim.nations.iter().any(|nation| &nation.id == party) {
            return Err(CoreError::data(format!("nation inconnue : {party}")));
        }
        unique.insert(party.clone());
    }
    if unique.len() < 2 {
        return Err(CoreError::data(
            "un traité exige au moins deux parties distinctes",
        ));
    }
    let parties: Vec<NationId> = unique.into_iter().collect();
    let id = TreatyId::new(next_id(
        "treaty",
        sim.diplomacy
            .treaties
            .iter()
            .map(|treaty| treaty.id.to_string()),
    ))?;
    let name = match kind {
        TreatyKind::Peace => "Traité de paix",
        TreatyKind::Alliance => "Alliance défensive",
        TreatyKind::NonAggression => "Pacte de non-agression",
        TreatyKind::Trade => "Accord commercial",
        TreatyKind::Defensive => "Pacte défensif",
        TreatyKind::Ceasefire => "Cessez-le-feu",
        TreatyKind::Vassalage => "Traité de vassalité",
        TreatyKind::Independence => "Traité d'indépendance",
        TreatyKind::Demilitarization => "Traité de démilitarisation",
        TreatyKind::Reparations => "Traité de réparations",
        TreatyKind::CustomsUnion => "Union douanière",
        TreatyKind::Research => "Réseau de recherche",
        TreatyKind::NuclearBan => "Interdiction nucléaire",
    };
    let treaty = Treaty {
        id: id.clone(),
        name: LocalizedText::plain(name),
        kind,
        parties: parties.clone(),
        signatories: parties.clone(),
        terms: Vec::new(),
        start_date: sim.date,
        end_date: None,
        duration_days,
        status: TreatyStatus::Active,
        secret: false,
        guarantors: Vec::new(),
        violations: Vec::new(),
    };
    sim.diplomacy.treaties.push(treaty);
    let tick = sim.tick;
    for party in &parties {
        if let Some(relation) = relation_mut(sim, &parties[0], party) {
            relation.opinion = (relation.opinion + 5.0).clamp(-100.0, 100.0);
            relation.trust = (relation.trust + 5.0).clamp(0.0, 100.0);
            relation.last_change_tick = Some(tick);
            if matches!(kind, TreatyKind::Alliance | TreatyKind::Defensive) {
                relation.status = RelationStatus::Allied;
            } else if relation.status == RelationStatus::Hostile {
                relation.status = RelationStatus::Neutral;
            }
        }
    }
    if matches!(kind, TreatyKind::Alliance | TreatyKind::Defensive) {
        for party in &parties {
            if let Some(nation) = sim.nations.iter_mut().find(|nation| &nation.id == party) {
                for other in &parties {
                    if other != party && !nation.allies.contains(other) {
                        nation.allies.push(other.clone());
                    }
                }
                nation.allies.sort();
                nation.allies.dedup();
            }
        }
    }
    events.push(SimEvent {
        tick: sim.tick,
        date: sim.date,
        kind: SimEventKind::Treaty,
        message: format!(
            "{} signé entre {}.",
            name,
            parties
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        ),
        nation: parties.first().cloned(),
        other: parties.get(1).cloned(),
        war: None,
        province: None,
    });
    Ok(id)
}

/// Ajoute une sanction active.
pub(crate) fn add_sanction(
    sim: &mut Simulation<'_>,
    by: &[NationId],
    target: &NationId,
    kind: SanctionKind,
    severity: f64,
    events: &mut Vec<SimEvent>,
) -> CoreResult<()> {
    if by.is_empty() {
        return Err(CoreError::data("une sanction exige au moins un émetteur"));
    }
    if !sim.nations.iter().any(|nation| &nation.id == target) {
        return Err(CoreError::data(format!("nation inconnue : {target}")));
    }
    if sim
        .diplomacy
        .sanctions
        .iter()
        .any(|sanction| sanction.is_active() && sanction.target == *target && sanction.kind == kind)
    {
        return Ok(());
    }
    let id = crate::ids::SanctionId::new(next_id(
        "sanction",
        sim.diplomacy
            .sanctions
            .iter()
            .map(|sanction| sanction.id.to_string()),
    ))?;
    sim.diplomacy.sanctions.push(Sanction {
        id,
        by: by.to_vec(),
        target: target.clone(),
        kind,
        severity: severity.clamp(0.0, 1.0),
        start_date: sim.date,
        end_date: None,
        status: SanctionStatus::Active,
        exemptions: Vec::new(),
    });
    events.push(SimEvent {
        tick: sim.tick,
        date: sim.date,
        kind: SimEventKind::Diplomacy,
        message: format!("Sanction {:?} adoptée contre {}.", kind, target),
        nation: by.first().cloned(),
        other: Some(target.clone()),
        war: None,
        province: None,
    });
    Ok(())
}

/// Déclare une guerre entre deux nations.
pub(crate) fn declare_war(
    sim: &mut Simulation<'_>,
    attacker: &NationId,
    defender: &NationId,
    target_province: Option<&crate::ids::ProvinceId>,
    events: &mut Vec<SimEvent>,
) -> CoreResult<WarId> {
    if attacker == defender {
        return Err(CoreError::data(
            "une nation ne peut pas se déclarer la guerre",
        ));
    }
    let attacker_index = sim
        .nations
        .iter()
        .position(|nation| &nation.id == attacker)
        .ok_or_else(|| CoreError::data(format!("nation inconnue : {attacker}")))?;
    let defender_index = sim
        .nations
        .iter()
        .position(|nation| &nation.id == defender)
        .ok_or_else(|| CoreError::data(format!("nation inconnue : {defender}")))?;
    if !sim.nations[attacker_index].is_alive() || !sim.nations[defender_index].is_alive() {
        return Err(CoreError::data(
            "une nation sans territoire ne peut pas faire la guerre",
        ));
    }
    if sim.wars.iter().any(|war| {
        war.is_active() && war.has_belligerent(attacker) && war.has_belligerent(defender)
    }) {
        return Err(CoreError::data("les deux nations sont déjà en guerre"));
    }
    if has_blocking_treaty(sim, attacker, defender) {
        return Err(CoreError::data(
            "un traité actif interdit cette déclaration de guerre",
        ));
    }
    let province = target_province
        .cloned()
        .or_else(|| first_claim_against(sim, attacker, defender))
        .or_else(|| first_owned_province(sim, defender));
    let casus_kind = if target_province.is_some() {
        CasusBelliKind::ClaimedProvince
    } else {
        CasusBelliKind::BorderDispute
    };
    let war_id = WarId::new(next_id(
        "war",
        sim.wars.iter().map(|war| war.id.to_string()),
    ))?;
    let front_provinces = border_provinces(sim, attacker, defender);
    let front_provinces = if front_provinces.is_empty() {
        province.iter().cloned().collect()
    } else {
        front_provinces
    };
    let mut war_goals = Vec::new();
    if let Some(province) = &province {
        war_goals.push(WarGoal {
            id: WarGoalId::new(format!("goal_{war_id}_1"))?,
            kind: WarGoalKind::ConquerProvince,
            target: defender.clone(),
            provinces: vec![province.clone()],
            priority: 1.0,
            satisfied: false,
            secret: false,
        });
    } else {
        war_goals.push(WarGoal {
            id: WarGoalId::new(format!("goal_{war_id}_1"))?,
            kind: WarGoalKind::Humiliate,
            target: defender.clone(),
            provinces: Vec::new(),
            priority: 0.6,
            satisfied: false,
            secret: false,
        });
    }
    let war = War {
        schema_version: 1,
        id: war_id.clone(),
        name: LocalizedText::plain(format!(
            "Guerre de {} contre {}",
            sim.nations[attacker_index].display_short_name(),
            sim.nations[defender_index].display_short_name()
        )),
        start_date: sim.date,
        end_date: None,
        status: WarStatus::Active,
        casus_belli: CasusBelli {
            kind: casus_kind,
            target: defender.clone(),
            description: Some(LocalizedText::plain(
                "Un différend frontalier dégénère en conflit ouvert.",
            )),
            province: province.clone(),
        },
        belligerents: vec![
            Belligerent {
                nation: attacker.clone(),
                side: BelligerentSide::Attacker,
                joined_date: sim.date,
                war_support: 60.0,
                casualties: 0,
                occupied_provinces: Vec::new(),
                contribution: 0.0,
            },
            Belligerent {
                nation: defender.clone(),
                side: BelligerentSide::Defender,
                joined_date: sim.date,
                war_support: 60.0,
                casualties: 0,
                occupied_provinces: Vec::new(),
                contribution: 0.0,
            },
        ],
        war_goals,
        fronts: vec![Front {
            id: crate::ids::FrontId::new(format!("front_{war_id}_1"))?,
            name: Some(LocalizedText::plain("Front principal")),
            provinces: front_provinces.clone(),
            attacker: attacker.clone(),
            defender: defender.clone(),
            status: FrontStatus::Active,
            momentum: 0.0,
            supply: 80.0,
            battles_won: 0,
            battles_lost: 0,
        }],
        battles: Vec::new(),
        armies: Vec::new(),
        score: WarScore::default(),
        peace_offers: Vec::new(),
        interventions: Vec::new(),
        treaty_id: None,
        tags: vec!["generated".to_owned()],
        assault_zones: front_provinces
            .iter()
            .enumerate()
            .map(|(index, province)| AssaultZone {
                id: AssaultZoneId::new(format!("zone_{war_id}_{index}")).unwrap_or_else(|_| {
                    AssaultZoneId::new("assault_zone").expect("identifiant valide")
                }),
                province: province.clone(),
                status: AssaultZoneStatus::Contested,
                controller: None,
                attacker_progress: 0.0,
                defender_progress: 0.0,
                score: 0.0,
            })
            .collect(),
    };
    sim.wars.push(war);
    for nation in &mut sim.nations {
        if (nation.id == *attacker || nation.id == *defender) && !nation.wars.contains(&war_id) {
            nation.wars.push(war_id.clone());
        }
    }
    let tick = sim.tick;
    if let Some(relation) = relation_mut(sim, attacker, defender) {
        relation.status = RelationStatus::AtWar;
        relation.opinion = (relation.opinion - 40.0).clamp(-100.0, 100.0);
        relation.trust = (relation.trust - 30.0).clamp(0.0, 100.0);
        relation.border_tension = 100.0;
        relation.last_change_tick = Some(tick);
    }
    events.push(SimEvent {
        tick: sim.tick,
        date: sim.date,
        kind: SimEventKind::War,
        message: format!(
            "{} déclare la guerre à {}.",
            sim.nations[attacker_index].display_short_name(),
            sim.nations[defender_index].display_short_name()
        ),
        nation: Some(attacker.clone()),
        other: Some(defender.clone()),
        war: Some(war_id.clone()),
        province,
    });
    Ok(war_id)
}

/// Termine une guerre et signe un traité de paix.
pub(crate) fn end_war(
    sim: &mut Simulation<'_>,
    war_id: &WarId,
    winner: &NationId,
    events: &mut Vec<SimEvent>,
) -> CoreResult<TreatyId> {
    let war_index = sim
        .wars
        .iter()
        .position(|war| &war.id == war_id)
        .ok_or_else(|| CoreError::data(format!("guerre inconnue : {war_id}")))?;
    if !sim.wars[war_index].is_active() {
        return Err(CoreError::data("cette guerre est déjà terminée"));
    }
    if !sim.wars[war_index].has_belligerent(winner) {
        return Err(CoreError::data(
            "le vainqueur ne participe pas à cette guerre",
        ));
    }
    let war = sim.wars[war_index].clone();
    let winner_side = war
        .side_of(winner)
        .ok_or_else(|| CoreError::data("camp du vainqueur introuvable"))?;
    let winner_side = if winner_side == BelligerentSide::Defender {
        BelligerentSide::Defender
    } else {
        BelligerentSide::Attacker
    };
    let loser_side = if winner_side == BelligerentSide::Attacker {
        BelligerentSide::Defender
    } else {
        BelligerentSide::Attacker
    };
    let loser = war
        .belligerents
        .iter()
        .find(|belligerent| belligerent.side == loser_side)
        .map(|belligerent| belligerent.nation.clone());

    let mut transferred = Vec::new();
    for province in &mut sim.world.provinces {
        let controlled_by_winner = province.controller.as_ref().is_some_and(|controller| {
            war.belligerents.iter().any(|belligerent| {
                &belligerent.nation == controller && belligerent.side == winner_side
            })
        });
        let owned_by_loser = province.owner.as_ref().is_some_and(|owner| {
            war.belligerents
                .iter()
                .any(|belligerent| &belligerent.nation == owner && belligerent.side == loser_side)
        });
        if controlled_by_winner && owned_by_loser {
            let is_capital = province.owner.as_ref().is_some_and(|owner| {
                sim.nations
                    .iter()
                    .any(|nation| &nation.id == owner && nation.capital == province.id)
            });
            if !is_capital {
                province.owner = Some(winner.clone());
                province.controller = Some(winner.clone());
                province.unrest = (province.unrest + 15.0).clamp(0.0, 100.0);
                transferred.push(province.id.clone());
            } else {
                province.controller = province.owner.clone();
            }
        } else if province.controller.as_ref().is_some_and(|controller| {
            war.belligerents
                .iter()
                .any(|belligerent| &belligerent.nation == controller)
        }) {
            province.controller = province.owner.clone();
        }
    }

    let treaty = sign_treaty(
        sim,
        TreatyKind::Peace,
        &war.belligerents
            .iter()
            .map(|belligerent| belligerent.nation.clone())
            .collect::<Vec<_>>(),
        Some(720),
        events,
    )?;
    if let Some(war) = sim.wars.get_mut(war_index) {
        war.status = WarStatus::Ended;
        war.end_date = Some(sim.date);
        war.treaty_id = Some(treaty.clone());
        for goal in &mut war.war_goals {
            if goal.target == *winner
                || goal
                    .provinces
                    .iter()
                    .any(|province| transferred.contains(province))
            {
                goal.satisfied = true;
            }
        }
    }
    for nation in &mut sim.nations {
        nation.wars.retain(|id| id != war_id);
        if &nation.id == winner {
            nation.prestige = (nation.prestige + 5.0).clamp(0.0, 100.0);
            nation.war_exhaustion = (nation.war_exhaustion - 10.0).clamp(0.0, 100.0);
        } else if Some(&nation.id) == loser.as_ref() {
            nation.prestige = (nation.prestige - 5.0).clamp(0.0, 100.0);
            nation.war_exhaustion = (nation.war_exhaustion + 5.0).clamp(0.0, 100.0);
        }
    }
    let tick = sim.tick;
    for belligerent in &war.belligerents {
        for other in &war.belligerents {
            if belligerent.nation != other.nation {
                if let Some(relation) = relation_mut(sim, &belligerent.nation, &other.nation) {
                    relation.status = RelationStatus::Truce;
                    relation.opinion = (relation.opinion + 10.0).clamp(-100.0, 100.0);
                    relation.trust = (relation.trust + 10.0).clamp(0.0, 100.0);
                    relation.border_tension = (relation.border_tension - 20.0).clamp(0.0, 100.0);
                    relation.last_change_tick = Some(tick);
                }
            }
        }
    }
    events.push(SimEvent {
        tick: sim.tick,
        date: sim.date,
        kind: SimEventKind::Peace,
        message: format!(
            "Paix signée : {} obtient {} province(s).",
            winner,
            transferred.len()
        ),
        nation: Some(winner.clone()),
        other: loser,
        war: Some(war_id.clone()),
        province: transferred.first().cloned(),
    });
    Ok(treaty)
}

fn first_claim_against(
    sim: &Simulation<'_>,
    attacker: &NationId,
    defender: &NationId,
) -> Option<crate::ids::ProvinceId> {
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

fn first_owned_province(sim: &Simulation<'_>, nation: &NationId) -> Option<crate::ids::ProvinceId> {
    sim.world
        .provinces
        .iter()
        .find(|province| province.owner.as_ref() == Some(nation))
        .map(|province| province.id.clone())
}

fn border_provinces(
    sim: &Simulation<'_>,
    attacker: &NationId,
    defender: &NationId,
) -> Vec<crate::ids::ProvinceId> {
    let mut result = Vec::new();
    for province in &sim.world.provinces {
        if province.owner.as_ref() != Some(attacker) {
            continue;
        }
        if province.neighbors.iter().any(|neighbor| {
            sim.world
                .province(neighbor)
                .is_some_and(|value| value.owner.as_ref() == Some(defender))
        }) {
            result.push(province.id.clone());
        }
    }
    result.sort();
    result
}
