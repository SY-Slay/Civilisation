//! Boucle géopolitique déterministe.
//!
//! Un tick de simulation est un tick de l'horloge. Les systèmes lourds
//! (économie, IA, guerre, révolutions, missiles, événements) s'exécutent une
//! fois par jour fictif, dans un ordre fixe. Toutes les itérations sont triées
//! par identifiant : aucun `HashMap` ne participe aux décisions.

pub mod ai;
pub mod diplomacy;
pub mod economy;
pub mod events;
pub mod missile;
pub mod revolution;
pub mod warfare;

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::data::GameData;
use crate::diplomacy::{
    Diplomacy, DiplomaticRequest, DiplomaticRequestKind, DiplomaticRequestStatus, Organization,
    OrganizationKind, OrganizationVote, Relation, RelationStatus, VoteChoice,
};
use crate::error::{CoreError, CoreResult};
use crate::event::EventState;
use crate::ids::{DiplomaticRequestId, NationId, OrganizationId, ProvinceId, WarId};
use crate::missile::MissileInstance;
use crate::nation::Nation;
use crate::revolution::Revolution;
use crate::rng::DeterministicRng;
use crate::save::{Save, WeeklyRankingEntry};
use crate::scenario::{Difficulty, Scenario};
use crate::time::{GameDate, Tick};
use crate::war::{Army, War};
use crate::world::World;

/// Catégorie d'un événement de simulation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimEventKind {
    Economy,
    Technology,
    Diplomacy,
    Treaty,
    War,
    Battle,
    Occupation,
    Peace,
    Revolution,
    Missile,
    Event,
    System,
}

/// Événement lisible produit par la simulation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SimEvent {
    pub tick: Tick,
    pub date: GameDate,
    pub kind: SimEventKind,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nation: Option<NationId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub other: Option<NationId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub war: Option<WarId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub province: Option<ProvinceId>,
}

impl SimEvent {
    /// Construit un événement simple rattaché à une nation.
    pub fn nation(
        tick: Tick,
        date: GameDate,
        kind: SimEventKind,
        nation: &NationId,
        message: impl Into<String>,
    ) -> Self {
        Self {
            tick,
            date,
            kind,
            message: message.into(),
            nation: Some(nation.clone()),
            other: None,
            war: None,
            province: None,
        }
    }
}

/// État runtime d'une simulation, indépendant du fichier de sauvegarde.
pub struct Simulation<'a> {
    pub(crate) data: &'a GameData,
    pub(crate) tick: Tick,
    pub(crate) date: GameDate,
    pub(crate) ticks_per_day: u32,
    pub(crate) start_date: GameDate,
    pub(crate) player_nation: Option<NationId>,
    pub(crate) world: World,
    pub(crate) nations: Vec<Nation>,
    pub(crate) diplomacy: Diplomacy,
    pub(crate) wars: Vec<War>,
    pub(crate) revolutions: Vec<Revolution>,
    pub(crate) armies: Vec<Army>,
    pub(crate) missiles: Vec<MissileInstance>,
    pub(crate) event_state: EventState,
    pub(crate) rng: DeterministicRng,
    pub(crate) difficulty: Difficulty,
    pub(crate) weekly_ranking: Vec<WeeklyRankingEntry>,
    pub(crate) last_ranking_week: u32,
}

impl<'a> Simulation<'a> {
    /// Crée une simulation depuis un scénario validé.
    pub fn from_scenario(
        scenario: &Scenario,
        data: &'a GameData,
        ticks_per_day: u32,
    ) -> CoreResult<Self> {
        scenario.validate()?;
        if ticks_per_day == 0 {
            return Err(CoreError::InvalidDate(
                "ticks_per_day doit être strictement positif".to_owned(),
            ));
        }
        Ok(Self {
            data,
            tick: Tick::ZERO,
            date: scenario.start_date,
            ticks_per_day,
            start_date: scenario.start_date,
            player_nation: scenario.player_nation.clone(),
            world: scenario.world.clone(),
            nations: scenario.nations.clone(),
            diplomacy: scenario.diplomacy.clone(),
            wars: scenario.wars.clone(),
            revolutions: scenario.revolutions.clone(),
            armies: scenario.armies.clone(),
            missiles: scenario.missiles_in_flight.clone(),
            event_state: EventState::default(),
            rng: DeterministicRng::new(scenario.seed, 0),
            difficulty: scenario.difficulty,
            weekly_ranking: Vec::new(),
            last_ranking_week: 0,
        })
    }

    /// Reprend une simulation depuis une sauvegarde vérifiée.
    pub fn from_save(
        save: &Save,
        rng: DeterministicRng,
        data: &'a GameData,
        ticks_per_day: u32,
    ) -> CoreResult<Self> {
        let ticks_per_day = save.ticks_per_day.unwrap_or(ticks_per_day);
        if ticks_per_day == 0 {
            return Err(CoreError::InvalidDate(
                "ticks_per_day doit être strictement positif".to_owned(),
            ));
        }
        save.date.validate()?;
        Ok(Self {
            data,
            tick: save.tick,
            date: save.date,
            ticks_per_day,
            start_date: save.start_date.unwrap_or(save.date),
            player_nation: save.metadata.player_nation.clone(),
            world: save.world.clone(),
            nations: save.nations.clone(),
            diplomacy: save.diplomacy.clone(),
            wars: save.wars.clone(),
            revolutions: save.revolutions.clone(),
            armies: save.armies.clone(),
            missiles: save.missiles_in_flight.clone(),
            event_state: save.event_state.clone(),
            rng,
            difficulty: save.difficulty.unwrap_or(Difficulty::Normal),
            weekly_ranking: save.weekly_ranking.clone(),
            last_ranking_week: save.last_ranking_week,
        })
    }

    /// Copie l'état courant dans une sauvegarde, sans recalculer le sceau.
    pub fn write_to(&self, save: &mut Save) {
        save.tick = self.tick;
        save.date = self.date;
        save.start_date = Some(self.start_date);
        save.difficulty = Some(self.difficulty);
        save.ticks_per_day = Some(self.ticks_per_day);
        save.world = self.world.clone();
        save.nations = self.nations.clone();
        save.diplomacy = self.diplomacy.clone();
        save.wars = self.wars.clone();
        save.revolutions = self.revolutions.clone();
        save.armies = self.armies.clone();
        save.missiles_in_flight = self.missiles.clone();
        save.event_state = self.event_state.clone();
        save.rng = self.rng.state();
        save.weekly_ranking = self.weekly_ranking.clone();
        save.last_ranking_week = self.last_ranking_week;
    }

    /// Avance de `ticks` ticks et exécute les systèmes à chaque jour fictif.
    pub fn advance_ticks(&mut self, ticks: u64) -> CoreResult<Vec<SimEvent>> {
        let mut events = Vec::new();
        let ticks_per_day = u64::from(self.ticks_per_day);
        for _ in 0..ticks {
            self.tick = self.tick.saturating_add(1);
            if self.tick.as_u64() % ticks_per_day == 0 {
                self.date = self.date.next_day();
                events.extend(self.tick_day()?);
            }
        }
        Ok(events)
    }

    /// Avance d'un nombre de jours fictifs complets.
    pub fn advance_days(&mut self, days: u64) -> CoreResult<Vec<SimEvent>> {
        self.advance_ticks(days.saturating_mul(u64::from(self.ticks_per_day)))
    }

    /// Exécute tous les systèmes pour le jour courant.
    pub fn tick_day(&mut self) -> CoreResult<Vec<SimEvent>> {
        let mut events = Vec::new();
        self.ensure_relations();
        self.expire_diplomatic_requests();
        self.update_weekly_ranking();
        economy::tick(self, &mut events);
        events::tick(self, &mut events)?;
        ai::tick(self, &mut events);
        warfare::tick(self, &mut events);
        revolution::tick(self, &mut events);
        missile::tick(self, &mut events);
        self.recompute_indexes();
        Ok(events)
    }

    /// Create a server-validated diplomatic request in the inbox.
    pub fn propose_diplomatic_request(
        &mut self,
        kind: DiplomaticRequestKind,
        from: &NationId,
        to: &NationId,
    ) -> CoreResult<DiplomaticRequestId> {
        if from == to {
            return Err(CoreError::data(
                "une nation ne peut pas envoyer une requête à elle-même",
            ));
        }
        if !self.nations.iter().any(|nation| &nation.id == from) {
            return Err(CoreError::data(format!("nation inconnue : {from}")));
        }
        if !self.nations.iter().any(|nation| &nation.id == to) {
            return Err(CoreError::data(format!("nation inconnue : {to}")));
        }
        if self.diplomacy.requests.iter().any(|request| {
            request.from == *from
                && request.to == *to
                && request.kind == kind
                && request.status == DiplomaticRequestStatus::Proposed
        }) {
            return Err(CoreError::data("une requête identique est déjà en attente"));
        }
        let id = DiplomaticRequestId::new(next_id(
            "diplo_request",
            self.diplomacy
                .requests
                .iter()
                .map(|request| request.id.to_string()),
        ))?;
        let title = match kind {
            DiplomaticRequestKind::Alliance => "Proposition d'alliance",
            DiplomaticRequestKind::Peace => "Proposition de paix",
            DiplomaticRequestKind::NonAggression => "Pacte de non-agression",
            DiplomaticRequestKind::Trade => "Accord commercial",
            DiplomaticRequestKind::DefensivePact => "Pacte défensif",
            DiplomaticRequestKind::Ceasefire => "Proposition de cessez-le-feu",
            DiplomaticRequestKind::JoinAlliance => "Demande d'admission",
        };
        let expires_tick = Some(
            self.tick
                .saturating_add(u64::from(self.ticks_per_day).saturating_mul(30)),
        );
        self.diplomacy.requests.push(DiplomaticRequest {
            id: id.clone(),
            kind,
            from: from.clone(),
            to: to.clone(),
            title: crate::text::LocalizedText::plain(title),
            terms: Vec::new(),
            created_date: self.date,
            created_tick: self.tick,
            expires_tick,
            status: DiplomaticRequestStatus::Proposed,
            resolved_tick: None,
        });
        Ok(id)
    }

    /// Respond to a diplomatic request. Only the recipient may accept or reject.
    pub fn respond_diplomatic_request(
        &mut self,
        request_id: &DiplomaticRequestId,
        recipient: &NationId,
        accept: bool,
    ) -> CoreResult<()> {
        let index = self
            .diplomacy
            .requests
            .iter()
            .position(|request| &request.id == request_id)
            .ok_or_else(|| CoreError::data(format!("requête inconnue : {request_id}")))?;
        let request = self.diplomacy.requests[index].clone();
        if request.to != *recipient {
            return Err(CoreError::data(
                "seule la nation destinataire peut répondre",
            ));
        }
        if request.status != DiplomaticRequestStatus::Proposed {
            return Err(CoreError::data("cette requête a déjà été résolue"));
        }
        if request
            .expires_tick
            .is_some_and(|expires| expires <= self.tick)
        {
            self.diplomacy.requests[index].status = DiplomaticRequestStatus::Expired;
            self.diplomacy.requests[index].resolved_tick = Some(self.tick);
            return Err(CoreError::data("cette requête a expiré"));
        }
        self.diplomacy.requests[index].status = if accept {
            DiplomaticRequestStatus::Accepted
        } else {
            DiplomaticRequestStatus::Rejected
        };
        self.diplomacy.requests[index].resolved_tick = Some(self.tick);
        if accept {
            let treaty_kind = match request.kind {
                DiplomaticRequestKind::Alliance | DiplomaticRequestKind::JoinAlliance => {
                    crate::diplomacy::TreatyKind::Alliance
                }
                DiplomaticRequestKind::Peace => crate::diplomacy::TreatyKind::Peace,
                DiplomaticRequestKind::NonAggression => crate::diplomacy::TreatyKind::NonAggression,
                DiplomaticRequestKind::Trade => crate::diplomacy::TreatyKind::Trade,
                DiplomaticRequestKind::DefensivePact => crate::diplomacy::TreatyKind::Defensive,
                DiplomaticRequestKind::Ceasefire => crate::diplomacy::TreatyKind::Ceasefire,
            };
            let mut events = Vec::new();
            diplomacy::sign_treaty(
                self,
                treaty_kind,
                &[request.from.clone(), request.to.clone()],
                Some(720),
                &mut events,
            )?;
        }
        Ok(())
    }

    /// Cancels a proposed diplomatic request by its sender or recipient.
    pub fn cancel_diplomatic_request(
        &mut self,
        request_id: &DiplomaticRequestId,
        actor: &NationId,
    ) -> CoreResult<()> {
        let request = self
            .diplomacy
            .requests
            .iter_mut()
            .find(|request| &request.id == request_id)
            .ok_or_else(|| CoreError::data(format!("requête inconnue : {request_id}")))?;
        if request.status != DiplomaticRequestStatus::Proposed {
            return Err(CoreError::data("cette requête a déjà été résolue"));
        }
        if request.from != *actor && request.to != *actor {
            return Err(CoreError::data(
                "seuls les parties de la requête peuvent l'annuler",
            ));
        }
        request.status = DiplomaticRequestStatus::Cancelled;
        request.resolved_tick = Some(self.tick);
        Ok(())
    }

    fn expire_diplomatic_requests(&mut self) {
        for request in &mut self.diplomacy.requests {
            if request.status == DiplomaticRequestStatus::Proposed
                && request
                    .expires_tick
                    .is_some_and(|expires| expires <= self.tick)
            {
                request.status = DiplomaticRequestStatus::Expired;
                request.resolved_tick = Some(self.tick);
            }
        }
    }

    /// Creates an alliance organization with governance and a shared treasury.
    pub fn create_alliance(
        &mut self,
        name: impl Into<String>,
        members: &[NationId],
        leader: Option<NationId>,
    ) -> CoreResult<OrganizationId> {
        if members.len() < 2 {
            return Err(CoreError::data("une alliance exige au moins deux membres"));
        }
        let mut unique = BTreeSet::new();
        for member in members {
            if !self.nations.iter().any(|nation| &nation.id == member) {
                return Err(CoreError::data(format!("nation inconnue : {member}")));
            }
            unique.insert(member.clone());
        }
        if let Some(leader) = leader.as_ref() {
            if !unique.contains(leader) {
                return Err(CoreError::data(
                    "le dirigeant doit être membre de l'alliance",
                ));
            }
        }
        let id = OrganizationId::new(next_id(
            "alliance",
            self.diplomacy
                .organizations
                .iter()
                .map(|organization| organization.id.to_string()),
        ))?;
        let organization = Organization {
            id: id.clone(),
            name: crate::text::LocalizedText::plain(name),
            kind: OrganizationKind::Alliance,
            members: unique.into_iter().collect(),
            founding_date: self.date,
            rules: Vec::new(),
            leader,
            cohesion: 50.0,
            votes: Vec::new(),
            active: true,
            treasury: 0.0,
            economy_terms: Vec::new(),
            diplomacy_terms: Vec::new(),
        };
        self.diplomacy.organizations.push(organization);
        Ok(id)
    }

    /// Records a weighted alliance vote and keeps it persistent in the save.
    pub fn cast_alliance_vote(
        &mut self,
        alliance: &OrganizationId,
        nation: &NationId,
        choice: VoteChoice,
    ) -> CoreResult<()> {
        let organization = self
            .diplomacy
            .organizations
            .iter_mut()
            .find(|organization| &organization.id == alliance)
            .ok_or_else(|| CoreError::data(format!("alliance inconnue : {alliance}")))?;
        if !organization.members.contains(nation) {
            return Err(CoreError::data(
                "seuls les membres peuvent voter dans l'alliance",
            ));
        }
        let weight = if organization.leader.as_ref() == Some(nation) {
            2.0
        } else {
            1.0
        };
        if let Some(vote) = organization
            .votes
            .iter_mut()
            .find(|vote| &vote.nation == nation)
        {
            vote.weight = weight;
            vote.vote = choice;
        } else {
            organization.votes.push(OrganizationVote {
                nation: nation.clone(),
                weight,
                vote: choice,
            });
        }
        Ok(())
    }

    /// Add a nation to an alliance organization.
    pub fn add_alliance_member(
        &mut self,
        alliance: &OrganizationId,
        nation: &NationId,
    ) -> CoreResult<()> {
        if !self.nations.iter().any(|value| &value.id == nation) {
            return Err(CoreError::data(format!("nation inconnue : {nation}")));
        }
        let organization = self
            .diplomacy
            .organizations
            .iter_mut()
            .find(|organization| &organization.id == alliance)
            .ok_or_else(|| CoreError::data(format!("alliance inconnue : {alliance}")))?;
        if !organization.members.contains(nation) {
            organization.members.push(nation.clone());
            organization.members.sort();
        }
        Ok(())
    }

    /// Remove a nation from an alliance, clearing it if it was the leader.
    pub fn remove_alliance_member(
        &mut self,
        alliance: &OrganizationId,
        nation: &NationId,
    ) -> CoreResult<()> {
        let organization = self
            .diplomacy
            .organizations
            .iter_mut()
            .find(|organization| &organization.id == alliance)
            .ok_or_else(|| CoreError::data(format!("alliance inconnue : {alliance}")))?;
        if !organization.members.contains(nation) {
            return Err(CoreError::data(
                "la nation n'est pas membre de cette alliance",
            ));
        }
        organization.members.retain(|member| member != nation);
        organization.votes.retain(|vote| &vote.nation != nation);
        if organization.leader.as_ref() == Some(nation) {
            organization.leader = organization.members.first().cloned();
        }
        Ok(())
    }

    /// Adjust a national purse by a signed amount.
    pub fn adjust_purse(&mut self, nation: &NationId, delta: f64) -> CoreResult<()> {
        let nation = self
            .nations
            .iter_mut()
            .find(|value| &value.id == nation)
            .ok_or_else(|| CoreError::data(format!("nation inconnue : {nation}")))?;
        nation.economy.purse = (nation.economy.purse + delta).max(0.0);
        Ok(())
    }

    fn update_weekly_ranking(&mut self) {
        let week = (self.date.ordinal() / 7).min(u64::from(u32::MAX)) as u32;
        if week == 0 || week == self.last_ranking_week {
            return;
        }
        let previous: std::collections::BTreeMap<NationId, u32> = self
            .weekly_ranking
            .iter()
            .map(|entry| (entry.nation.clone(), entry.rank))
            .collect();
        let mut scored: Vec<(NationId, f64, f64, f64, u64)> = self
            .nations
            .iter()
            .map(|nation| {
                let score = nation.economy.gdp * 0.0000004
                    + nation.economy.treasury * 0.000001
                    + nation.economy.purse * 0.000001
                    + nation.population.total as f64 * 0.00001
                    + nation.technology.researched.len() as f64 * 50.0
                    + nation.prestige
                    + nation.stability * 0.5;
                (
                    nation.id.clone(),
                    score,
                    nation.economy.treasury,
                    nation.economy.gdp,
                    nation.population.total,
                )
            })
            .collect();
        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });
        let ranking = scored
            .into_iter()
            .enumerate()
            .map(|(index, (nation, score, treasury, gdp, population))| {
                let previous_rank = previous.get(&nation).copied();
                WeeklyRankingEntry {
                    nation: nation.clone(),
                    rank: index as u32 + 1,
                    previous_rank,
                    score,
                    treasury,
                    gdp,
                    population,
                    week,
                }
            })
            .collect();
        self.weekly_ranking = ranking;
        self.last_ranking_week = week;
    }

    /// Date courante.
    pub fn date(&self) -> GameDate {
        self.date
    }

    /// Tick courant.
    pub fn tick(&self) -> Tick {
        self.tick
    }

    /// Accès aux nations.
    pub fn nations(&self) -> &[Nation] {
        &self.nations
    }

    /// Accès à la diplomatie.
    pub fn diplomacy(&self) -> &Diplomacy {
        &self.diplomacy
    }

    /// Accès aux guerres.
    pub fn wars(&self) -> &[War] {
        &self.wars
    }

    /// Accès aux révolutions.
    pub fn revolutions(&self) -> &[Revolution] {
        &self.revolutions
    }

    /// Accès aux missiles.
    pub fn missiles(&self) -> &[MissileInstance] {
        &self.missiles
    }

    /// Déclare une guerre et retourne son identifiant.
    pub fn declare_war(
        &mut self,
        attacker: &NationId,
        defender: &NationId,
        province: Option<&ProvinceId>,
    ) -> CoreResult<WarId> {
        let mut events = Vec::new();
        diplomacy::declare_war(self, attacker, defender, province, &mut events)
    }

    /// Termine une guerre avec le vainqueur désigné.
    pub fn make_peace(&mut self, war_id: &WarId) -> CoreResult<crate::ids::TreatyId> {
        let war = self
            .wars
            .iter()
            .find(|war| &war.id == war_id)
            .cloned()
            .ok_or_else(|| CoreError::data(format!("guerre inconnue : {war_id}")))?;
        let winner = if war.score.attacker >= war.score.defender {
            war.belligerents
                .iter()
                .find(|belligerent| belligerent.side == crate::war::BelligerentSide::Attacker)
                .map(|belligerent| belligerent.nation.clone())
        } else {
            war.belligerents
                .iter()
                .find(|belligerent| belligerent.side == crate::war::BelligerentSide::Defender)
                .map(|belligerent| belligerent.nation.clone())
        }
        .ok_or_else(|| CoreError::data("guerre sans belligérant"))?;
        let mut events = Vec::new();
        diplomacy::end_war(self, war_id, &winner, &mut events)
    }

    /// Signe un traité actif.
    pub fn sign_treaty(
        &mut self,
        kind: crate::diplomacy::TreatyKind,
        parties: &[NationId],
        duration_days: Option<u32>,
    ) -> CoreResult<crate::ids::TreatyId> {
        let mut events = Vec::new();
        diplomacy::sign_treaty(self, kind, parties, duration_days, &mut events)
    }

    /// Lance un missile.
    pub fn launch_missile(
        &mut self,
        owner: &NationId,
        missile_type: &crate::ids::MissileTypeId,
        target: &ProvinceId,
    ) -> CoreResult<crate::ids::MissileInstanceId> {
        let mut events = Vec::new();
        missile::launch(self, owner, missile_type, target, &mut events)
    }

    /// Applique un choix d'événement actif.
    pub fn resolve_event(
        &mut self,
        nation: &NationId,
        choice: &crate::ids::EventChoiceId,
    ) -> CoreResult<()> {
        let mut events = Vec::new();
        events::resolve(self, nation, choice, &mut events)
    }

    /// Sélectionne une technologie à rechercher.
    pub fn set_research(
        &mut self,
        nation: &NationId,
        technology: &crate::ids::TechnologyId,
    ) -> CoreResult<()> {
        let definition = self
            .data
            .technology(technology)
            .ok_or_else(|| CoreError::data(format!("technologie inconnue : {technology}")))?;
        let Some(nation) = self.nations.iter_mut().find(|value| &value.id == nation) else {
            return Err(CoreError::data(format!("nation inconnue : {nation}")));
        };
        if nation.technology.researched.contains(technology) {
            return Err(CoreError::data("technologie déjà recherchée"));
        }
        for prerequisite in &definition.prerequisites {
            if !nation.technology.researched.contains(prerequisite) {
                return Err(CoreError::data(format!(
                    "prérequis manquant : {prerequisite}"
                )));
            }
        }
        nation.technology.researching = Some(technology.clone());
        Ok(())
    }

    /// Force le démarrage d'une révolution (outil de test et commande MJ).
    pub fn start_revolution(&mut self, nation: &NationId) -> CoreResult<()> {
        let mut events = Vec::new();
        if !self.nations.iter().any(|value| &value.id == nation) {
            return Err(CoreError::data(format!("nation inconnue : {nation}")));
        }
        revolution::start_revolution(self, nation, &mut events);
        Ok(())
    }

    /// Ajoute une sanction active.
    pub fn add_sanction(
        &mut self,
        by: &[NationId],
        target: &NationId,
        kind: crate::diplomacy::SanctionKind,
        severity: f64,
    ) -> CoreResult<()> {
        let mut events = Vec::new();
        diplomacy::add_sanction(self, by, target, kind, severity, &mut events)
    }

    /// Crée les relations manquantes pour toutes les paires de nations.
    fn ensure_relations(&mut self) {
        let mut ids: Vec<NationId> = self
            .nations
            .iter()
            .map(|nation| nation.id.clone())
            .collect();
        ids.sort();
        for (index, first) in ids.iter().enumerate() {
            for second in ids.iter().skip(index + 1) {
                if first == second {
                    continue;
                }
                let exists = self
                    .diplomacy
                    .relations
                    .iter()
                    .any(|relation| relation.involves(first) && relation.involves(second));
                if !exists {
                    let (tension, opinion) = initial_tension(self, first, second);
                    self.diplomacy.relations.push(Relation {
                        a: first.clone(),
                        b: second.clone(),
                        opinion,
                        trust: 50.0,
                        fear: 0.0,
                        trade_dependence: 0.0,
                        border_tension: tension,
                        status: RelationStatus::Neutral,
                        embassy: false,
                        contact: true,
                        last_change_tick: None,
                        modifiers: Vec::new(),
                    });
                }
            }
        }
        self.diplomacy
            .relations
            .sort_by(|left, right| left.a.cmp(&right.a).then(left.b.cmp(&right.b)));
    }

    /// Recalcule les index dérivés et retire les références mortes.
    fn recompute_indexes(&mut self) {
        let nation_ids: BTreeSet<NationId> = self
            .nations
            .iter()
            .map(|nation| nation.id.clone())
            .collect();

        for province in &mut self.world.provinces {
            if let Some(controller) = &province.controller {
                if !nation_ids.contains(controller) {
                    province.controller = province.owner.clone();
                }
            }
            if province.controller.is_none() {
                province.controller = province.owner.clone();
            }
            if let Some(owner) = &province.owner {
                if !nation_ids.contains(owner) {
                    province.owner = None;
                    province.controller = None;
                }
            }
        }

        let active_war_ids: BTreeSet<WarId> = self
            .wars
            .iter()
            .filter(|war| war.is_active())
            .map(|war| war.id.clone())
            .collect();
        let active_revolution_ids: BTreeSet<String> = self
            .revolutions
            .iter()
            .filter(|revolution| revolution.is_active())
            .map(|revolution| revolution.id.to_string())
            .collect();
        let active_missile_ids: BTreeSet<String> = self
            .missiles
            .iter()
            .filter(|missile| missile.is_in_flight())
            .map(|missile| missile.id.to_string())
            .collect();

        for nation in &mut self.nations {
            nation.provinces = self
                .world
                .provinces
                .iter()
                .filter(|province| province.owner.as_ref() == Some(&nation.id))
                .map(|province| province.id.clone())
                .collect();
            nation.provinces.sort();
            nation.provinces.dedup();
            if nation.provinces.is_empty() {
                // Une nation sans territoire est conservée comme entité
                // historique, mais ne participe plus aux systèmes actifs.
                nation.wars.clear();
                nation.revolutions.clear();
                nation.armies.clear();
                nation.missiles.clear();
                continue;
            }
            if !nation.provinces.contains(&nation.capital) {
                nation.capital = nation.provinces[0].clone();
            }
            nation.wars.retain(|war| active_war_ids.contains(war));
            nation
                .revolutions
                .retain(|revolution| active_revolution_ids.contains(&revolution.to_string()));
            nation
                .missiles
                .retain(|missile| active_missile_ids.contains(&missile.to_string()));
            nation.allies.retain(|ally| nation_ids.contains(ally));
            nation.allies.sort();
            nation.allies.dedup();
            nation.economy.sanctions = self
                .diplomacy
                .sanctions
                .iter()
                .filter(|sanction| sanction.is_active() && sanction.targets(&nation.id))
                .map(|sanction| crate::nation::economy::SanctionRef {
                    by: sanction
                        .by
                        .first()
                        .cloned()
                        .unwrap_or_else(|| nation.id.clone()),
                    severity: sanction.severity,
                    since: Some(sanction.start_date),
                })
                .collect();
        }

        for province in &mut self.world.provinces {
            province.capital_of = None;
        }
        for nation in &self.nations {
            if let Some(province) = self.world.province_mut(&nation.capital) {
                province.capital_of = Some(nation.id.clone());
            }
        }

        for relation in &mut self.diplomacy.relations {
            let at_war = self.wars.iter().any(|war| {
                war.is_active()
                    && war.has_belligerent(&relation.a)
                    && war.has_belligerent(&relation.b)
            });
            if at_war {
                relation.status = RelationStatus::AtWar;
            } else if relation.status == RelationStatus::AtWar {
                relation.status = RelationStatus::Truce;
            }
        }
    }
}

fn initial_tension(sim: &Simulation<'_>, first: &NationId, second: &NationId) -> (f64, f64) {
    let mut tension: f64 = 0.0;
    let mut opinion: f64 = 0.0;
    let first_nation = sim.nations.iter().find(|nation| &nation.id == first);
    let second_nation = sim.nations.iter().find(|nation| &nation.id == second);
    if let Some(nation) = first_nation {
        for claim in &nation.claims {
            if sim
                .world
                .province(&claim.province)
                .is_some_and(|province| province.owner.as_ref() == Some(second))
            {
                tension += 25.0;
                opinion -= 15.0;
            }
        }
    }
    if let Some(nation) = second_nation {
        for claim in &nation.claims {
            if sim
                .world
                .province(&claim.province)
                .is_some_and(|province| province.owner.as_ref() == Some(first))
            {
                tension += 25.0;
                opinion -= 15.0;
            }
        }
    }
    let adjacent = sim.world.provinces.iter().any(|province| {
        province.owner.as_ref() == Some(first)
            && province.neighbors.iter().any(|neighbor| {
                sim.world
                    .province(neighbor)
                    .is_some_and(|value| value.owner.as_ref() == Some(second))
            })
    });
    if adjacent {
        tension += 10.0;
    }
    (tension.clamp(0.0, 100.0), opinion.clamp(-100.0, 100.0))
}

/// Borne une valeur.
pub(crate) fn clamp(value: f64, min: f64, max: f64) -> f64 {
    value.max(min).min(max)
}

/// Convertit un taux annuel en taux journalier.
pub(crate) fn daily_rate(annual: f64) -> f64 {
    (1.0 + annual).max(0.0).powf(1.0 / 360.0) - 1.0
}

/// Génère un identifiant préfixé unique.
pub(crate) fn next_id(prefix: &str, existing: impl Iterator<Item = String>) -> String {
    let mut highest = 0u64;
    for value in existing {
        if let Some(suffix) = value.strip_prefix(prefix) {
            if let Some(suffix) = suffix.strip_prefix('_') {
                if let Ok(number) = suffix.parse::<u64>() {
                    highest = highest.max(number);
                }
            }
        }
    }
    format!("{prefix}_{:03}", highest + 1)
}

/// Indice d'une nation.
pub(crate) fn nation_index(sim: &Simulation<'_>, id: &NationId) -> Option<usize> {
    sim.nations.iter().position(|nation| &nation.id == id)
}

/// Puissance militaire agrégée, utilisée par l'IA et les combats.
pub(crate) fn nation_power(nation: &Nation) -> f64 {
    let manpower = nation.military.manpower as f64;
    let army = nation.military.army_size as f64;
    let readiness = nation.military.readiness / 100.0;
    let morale = nation.military.morale / 100.0;
    let tech = 1.0 + nation.technology.researched.len() as f64 * 0.01;
    (army * 0.7 + manpower * 0.2 + nation.military.navy_size as f64 * 0.05)
        * (0.5 + readiness * 0.5)
        * (0.5 + morale * 0.5)
        * tech
}

/// Vrai si une nation participe à une guerre active.
pub(crate) fn is_at_war(sim: &Simulation<'_>, nation: &NationId) -> bool {
    sim.wars
        .iter()
        .any(|war| war.is_active() && war.has_belligerent(nation))
}

/// Centre 2D d'une province.
pub(crate) fn province_center(
    sim: &Simulation<'_>,
    province: &ProvinceId,
) -> Option<crate::math::Vec2> {
    sim.world
        .province(province)
        .map(|value| value.center.unwrap_or_else(|| value.polygon.centroid()))
}
