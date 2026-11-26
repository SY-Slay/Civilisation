//! Modèles de données prêts à être rendus par un écran Horizon.
//!
//! Aucun appel de rendu n'est effectué ici : l'adaptateur transforme ces
//! structures en widgets natifs.

pub mod diplomacy;
pub mod map;
pub mod missile;
pub mod nation;
pub mod theme;
pub mod war;
pub mod widgets;

use crate::civ_core::ids::{NationId, ProvinceId};
use crate::civ_core::nation::Nation;
use crate::civ_core::time::{GameDate, Tick};
use crate::civ_core::war::{BelligerentSide, War};
use serde::{Deserialize, Serialize};

use crate::state::Session;

/// Marqueur de province sur la carte.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MapMarker {
    pub province: ProvinceId,
    pub name: String,
    pub owner: Option<NationId>,
    pub controller: Option<NationId>,
    pub x: f64,
    pub z: f64,
    #[serde(default)]
    pub polygon: Vec<[f64; 2]>,
    pub terrain: String,
    pub unrest: f64,
    pub contested: bool,
    pub assault_score: f64,
}

/// Résumé d'une nation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NationSummary {
    pub id: NationId,
    pub name: String,
    pub short_name: String,
    pub color: String,
    pub capital: ProvinceId,
    pub provinces: usize,
    pub population: u64,
    pub stability: f64,
    pub legitimacy: f64,
    pub at_war: bool,
    pub is_player: bool,
    pub treasury: f64,
    pub purse: f64,
    pub gdp: f64,
    pub weekly_rank: Option<u32>,
    pub previous_weekly_rank: Option<u32>,
    pub research_domains: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DiplomaticRequestSummary {
    pub id: String,
    pub kind: String,
    pub from: NationId,
    pub to: NationId,
    pub title: String,
    pub status: String,
    pub expires_tick: Option<Tick>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssaultZoneSummary {
    pub id: String,
    pub province: ProvinceId,
    pub status: String,
    pub controller: Option<NationId>,
    pub attacker_progress: f64,
    pub defender_progress: f64,
    pub score: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AllianceSummary {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub members: Vec<NationId>,
    pub leader: Option<NationId>,
    pub treasury: f64,
    pub cohesion: f64,
}

/// Compteurs diplomatiques.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiplomacySummary {
    pub relations: usize,
    pub active_treaties: usize,
    pub organizations: usize,
    pub sanctions: usize,
    pub guarantees: usize,
    pub war_pairs: usize,
}

/// Résumé d'une guerre.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WarSummary {
    pub id: String,
    pub name: String,
    pub status: String,
    pub attackers: Vec<String>,
    pub defenders: Vec<String>,
    pub casualties: u64,
    pub fronts: usize,
    pub battles: usize,
}

/// Résumé d'un missile en vol.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MissileSummary {
    pub id: String,
    pub owner: NationId,
    pub target: ProvinceId,
    pub status: String,
    pub arrival_tick: Tick,
    pub in_flight: bool,
    pub danger_tier: u8,
}

/// Modèle complet consommé par les écrans.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiModel {
    pub date: GameDate,
    pub tick: Tick,
    pub player_nation: Option<NationId>,
    pub nations: Vec<NationSummary>,
    pub map: Vec<MapMarker>,
    pub diplomacy: DiplomacySummary,
    pub wars: Vec<WarSummary>,
    pub missiles: Vec<MissileSummary>,
    pub diplomatic_requests: Vec<DiplomaticRequestSummary>,
    pub assault_zones: Vec<AssaultZoneSummary>,
    pub alliances: Vec<AllianceSummary>,
    pub weekly_ranking: Vec<(NationId, u32, Option<u32>, f64)>,
}

impl UiModel {
    /// Construit le modèle depuis une session.
    pub fn from_session(session: &Session) -> Self {
        let save = session.save();
        let player = session.player_nation().cloned();
        let nations = save
            .nations
            .iter()
            .map(|nation| nation_summary(nation, player.as_ref(), &save.weekly_ranking))
            .collect();
        let map = save
            .world
            .provinces
            .iter()
            .map(|province| {
                let center = province
                    .center
                    .unwrap_or_else(|| province.polygon.centroid());
                let zone = save
                    .wars
                    .iter()
                    .flat_map(|war| war.assault_zones.iter())
                    .find(|zone| zone.province == province.id);
                MapMarker {
                    province: province.id.clone(),
                    name: province.name.resolve().to_owned(),
                    owner: province.owner.clone(),
                    controller: province.controller.clone(),
                    x: center.x(),
                    z: center.z(),
                    polygon: province
                        .polygon
                        .points()
                        .iter()
                        .map(|point| [point.x(), point.z()])
                        .collect(),
                    terrain: format!("{:?}", province.terrain).to_ascii_lowercase(),
                    unrest: province.unrest,
                    contested: zone
                        .is_some_and(|zone| format!("{:?}", zone.status).eq("Contested")),
                    assault_score: zone.map(|zone| zone.score).unwrap_or(0.0),
                }
            })
            .collect();
        let diplomacy = DiplomacySummary {
            relations: save.diplomacy.relations.len(),
            active_treaties: save
                .diplomacy
                .treaties
                .iter()
                .filter(|treaty| treaty.is_active())
                .count(),
            organizations: save.diplomacy.organizations.len(),
            sanctions: save.diplomacy.sanctions.len(),
            guarantees: save.diplomacy.guarantees.len(),
            war_pairs: save
                .diplomacy
                .relations
                .iter()
                .filter(|relation| {
                    relation.status == crate::civ_core::diplomacy::RelationStatus::AtWar
                })
                .count(),
        };
        let wars = save.wars.iter().map(war_summary).collect();
        let missiles = save
            .missiles_in_flight
            .iter()
            .map(|missile| {
                let danger_tier = session
                    .data()
                    .missile(&missile.missile_type)
                    .map(|definition| definition.danger_tier)
                    .unwrap_or(1);
                MissileSummary {
                    id: missile.id.to_string(),
                    owner: missile.owner.clone(),
                    target: missile.target.province.clone(),
                    status: format!("{:?}", missile.status).to_ascii_lowercase(),
                    arrival_tick: missile.arrival_tick,
                    in_flight: missile.is_in_flight(),
                    danger_tier,
                }
            })
            .collect();
        let diplomatic_requests = save
            .diplomacy
            .requests
            .iter()
            .map(|request| DiplomaticRequestSummary {
                id: request.id.to_string(),
                kind: format!("{:?}", request.kind).to_ascii_lowercase(),
                from: request.from.clone(),
                to: request.to.clone(),
                title: request.title.resolve().to_owned(),
                status: format!("{:?}", request.status).to_ascii_lowercase(),
                expires_tick: request.expires_tick,
            })
            .collect();
        let assault_zones = save
            .wars
            .iter()
            .flat_map(|war| war.assault_zones.iter())
            .map(|zone| AssaultZoneSummary {
                id: zone.id.to_string(),
                province: zone.province.clone(),
                status: format!("{:?}", zone.status).to_ascii_lowercase(),
                controller: zone.controller.clone(),
                attacker_progress: zone.attacker_progress,
                defender_progress: zone.defender_progress,
                score: zone.score,
            })
            .collect();
        let alliances = save
            .diplomacy
            .organizations
            .iter()
            .map(|organization| AllianceSummary {
                id: organization.id.to_string(),
                name: organization.name.resolve().to_owned(),
                kind: format!("{:?}", organization.kind).to_ascii_lowercase(),
                members: organization.members.clone(),
                leader: organization.leader.clone(),
                treasury: organization.treasury,
                cohesion: organization.cohesion,
            })
            .collect();
        let weekly_ranking = save
            .weekly_ranking
            .iter()
            .map(|entry| {
                (
                    entry.nation.clone(),
                    entry.rank,
                    entry.previous_rank,
                    entry.score,
                )
            })
            .collect();
        Self {
            date: save.date,
            tick: save.tick,
            player_nation: player,
            nations,
            map,
            diplomacy,
            wars,
            missiles,
            diplomatic_requests,
            assault_zones,
            alliances,
            weekly_ranking,
        }
    }
}

fn nation_summary(
    nation: &Nation,
    player: Option<&NationId>,
    ranking: &[crate::civ_core::save::WeeklyRankingEntry],
) -> NationSummary {
    let rank = ranking
        .iter()
        .find(|entry| entry.nation == nation.id)
        .map(|entry| (Some(entry.rank), entry.previous_rank))
        .unwrap_or((None, None));
    let mut research_domains: Vec<String> = nation
        .technology
        .domains
        .iter()
        .map(|domain| {
            format!(
                "{:?}: niv {} / {} pts",
                domain.domain, domain.level, domain.points as u64
            )
        })
        .collect();
    if research_domains.is_empty() {
        research_domains = [
            "Military: niv 0 / 0 pts",
            "Economy: niv 0 / 0 pts",
            "Diplomacy: niv 0 / 0 pts",
            "Infrastructure: niv 0 / 0 pts",
            "Technology: niv 0 / 0 pts",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
    }
    NationSummary {
        id: nation.id.clone(),
        name: nation.name.resolve().to_owned(),
        short_name: nation.display_short_name().to_owned(),
        color: nation.color.clone(),
        capital: nation.capital.clone(),
        provinces: nation.province_count(),
        population: nation.population.total,
        stability: nation.stability,
        legitimacy: nation.legitimacy,
        at_war: nation.is_at_war(),
        is_player: player == Some(&nation.id),
        treasury: nation.economy.treasury,
        purse: nation.economy.purse,
        gdp: nation.economy.gdp,
        weekly_rank: rank.0,
        previous_weekly_rank: rank.1,
        research_domains,
    }
}

fn war_summary(war: &War) -> WarSummary {
    let mut attackers = Vec::new();
    let mut defenders = Vec::new();
    for belligerent in &war.belligerents {
        let name = belligerent.nation.to_string();
        match belligerent.side {
            BelligerentSide::Attacker | BelligerentSide::CoBelligerent => attackers.push(name),
            BelligerentSide::Defender => defenders.push(name),
        }
    }
    WarSummary {
        id: war.id.to_string(),
        name: war.name.resolve().to_owned(),
        status: format!("{:?}", war.status).to_ascii_lowercase(),
        attackers,
        defenders,
        casualties: war.total_casualties(),
        fronts: war.fronts.len(),
        battles: war.battles.len(),
    }
}
