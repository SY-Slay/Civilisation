//! Session de jeu testable, sans dépendance Horizon.

use crate::civ_core::data::GameData;
use crate::civ_core::error::{CoreError, CoreResult};
use crate::civ_core::ids::NationId;
use crate::civ_core::nation::Nation;
use crate::civ_core::rng::DeterministicRng;
use crate::civ_core::save::Save;
use crate::civ_core::scenario::Scenario;
use crate::civ_core::sim::{SimEvent, Simulation};
use crate::civ_core::time::{GameDate, SimClock, Tick};
use crate::civ_core::world::{Province, World};

use crate::host::Host;
use uuid::Uuid;

/// Résumé léger pour l'interface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionStatus {
    pub date: GameDate,
    pub tick: Tick,
    pub provinces: usize,
    pub nations: usize,
    pub wars: usize,
    pub revolutions: usize,
    pub player_nation: Option<NationId>,
}

/// État runtime d'une partie.
#[derive(Clone, Debug)]
pub struct Session {
    save: Save,
    clock: SimClock,
    rng: DeterministicRng,
    data: GameData,
    dirty: bool,
}

impl Session {
    /// Crée une session avec un nouvel identifiant de sauvegarde UUID v4.
    pub fn new_with_generated_id(
        scenario: &Scenario,
        save_name: impl Into<String>,
        created_at: impl Into<String>,
        ticks_per_day: u32,
    ) -> CoreResult<Self> {
        Self::new(
            scenario,
            Uuid::new_v4().to_string(),
            save_name,
            created_at,
            ticks_per_day,
        )
    }

    /// Crée une session à partir d'un scénario validé.
    pub fn new(
        scenario: &Scenario,
        save_id: impl Into<String>,
        save_name: impl Into<String>,
        created_at: impl Into<String>,
        ticks_per_day: u32,
    ) -> CoreResult<Self> {
        Self::new_with_data(
            scenario,
            GameData::default(),
            save_id,
            save_name,
            created_at,
            ticks_per_day,
        )
    }

    /// Crée une session avec un jeu de définitions explicite.
    pub fn new_with_data(
        scenario: &Scenario,
        data: GameData,
        save_id: impl Into<String>,
        save_name: impl Into<String>,
        created_at: impl Into<String>,
        ticks_per_day: u32,
    ) -> CoreResult<Self> {
        scenario.validate()?;
        data.validate()?;
        let created_at = created_at.into();
        let mut save =
            Save::from_scenario(scenario, save_id, save_name, created_at.clone(), created_at)?;
        save.ticks_per_day = Some(ticks_per_day);
        save.seal()?;
        let clock = SimClock::new(scenario.start_date, ticks_per_day)?;
        let rng = DeterministicRng::new(scenario.seed, 0);
        Ok(Self {
            save,
            clock,
            rng,
            data,
            dirty: false,
        })
    }

    /// Reprend une sauvegarde vérifiée.
    pub fn from_save(save: Save, ticks_per_day: u32) -> CoreResult<Self> {
        Self::from_save_with_data(save, GameData::default(), ticks_per_day)
    }

    /// Reprend une sauvegarde avec un jeu de définitions explicite.
    pub fn from_save_with_data(save: Save, data: GameData, ticks_per_day: u32) -> CoreResult<Self> {
        save.check_compatibility()?;
        save.validate_identity()?;
        save.verify()?;
        save.validate_payload()?;
        data.validate()?;
        let ticks_per_day = save.ticks_per_day.unwrap_or(ticks_per_day);
        if ticks_per_day == 0 {
            return Err(CoreError::InvalidDate(
                "ticks_per_day doit être strictement positif".to_owned(),
            ));
        }
        save.date.validate()?;
        let clock = SimClock {
            tick: save.tick,
            date: save.date,
            ticks_per_day,
        };
        let rng = DeterministicRng::from_state(&save.rng)?;
        Ok(Self {
            save,
            clock,
            rng,
            data,
            dirty: false,
        })
    }

    /// Avance la simulation et retourne les événements produits.
    pub fn advance_ticks(&mut self, ticks: u64) -> CoreResult<Vec<SimEvent>> {
        if ticks == 0 {
            return Ok(Vec::new());
        }
        let mut simulation = Simulation::from_save(
            &self.save,
            self.rng.clone(),
            &self.data,
            self.clock.ticks_per_day,
        )?;
        let events = simulation.advance_ticks(ticks)?;
        simulation.write_to(&mut self.save);
        self.clock.tick = self.save.tick;
        self.clock.date = self.save.date;
        self.rng = DeterministicRng::from_state(&self.save.rng)?;
        self.dirty = true;
        Ok(events)
    }

    /// Applique une action immédiate via une simulation temporaire.
    fn with_simulation<T>(
        &mut self,
        action: impl FnOnce(&mut Simulation<'_>) -> CoreResult<T>,
    ) -> CoreResult<T> {
        let mut simulation = Simulation::from_save(
            &self.save,
            self.rng.clone(),
            &self.data,
            self.clock.ticks_per_day,
        )?;
        let result = action(&mut simulation)?;
        simulation.write_to(&mut self.save);
        self.clock.tick = self.save.tick;
        self.clock.date = self.save.date;
        self.rng = DeterministicRng::from_state(&self.save.rng)?;
        self.dirty = true;
        Ok(result)
    }

    /// Déclare une guerre.
    pub fn declare_war(
        &mut self,
        attacker: &NationId,
        defender: &NationId,
        province: Option<&crate::civ_core::ids::ProvinceId>,
    ) -> CoreResult<crate::civ_core::ids::WarId> {
        self.with_simulation(|simulation| simulation.declare_war(attacker, defender, province))
    }

    /// Signe la paix d'une guerre.
    pub fn make_peace(
        &mut self,
        war: &crate::civ_core::ids::WarId,
    ) -> CoreResult<crate::civ_core::ids::TreatyId> {
        self.with_simulation(|simulation| simulation.make_peace(war))
    }

    /// Signe un traité.
    pub fn sign_treaty(
        &mut self,
        kind: crate::civ_core::diplomacy::TreatyKind,
        parties: &[NationId],
        duration_days: Option<u32>,
    ) -> CoreResult<crate::civ_core::ids::TreatyId> {
        self.with_simulation(|simulation| simulation.sign_treaty(kind, parties, duration_days))
    }

    /// Lance un missile.
    pub fn launch_missile(
        &mut self,
        owner: &NationId,
        missile_type: &crate::civ_core::ids::MissileTypeId,
        target: &crate::civ_core::ids::ProvinceId,
    ) -> CoreResult<crate::civ_core::ids::MissileInstanceId> {
        self.with_simulation(|simulation| simulation.launch_missile(owner, missile_type, target))
    }

    /// Applique le choix d'un événement actif.
    pub fn resolve_event(
        &mut self,
        nation: &NationId,
        choice: &crate::civ_core::ids::EventChoiceId,
    ) -> CoreResult<()> {
        self.with_simulation(|simulation| simulation.resolve_event(nation, choice))
    }

    /// Sélectionne une technologie.
    pub fn set_research(
        &mut self,
        nation: &NationId,
        technology: &crate::civ_core::ids::TechnologyId,
    ) -> CoreResult<()> {
        self.with_simulation(|simulation| simulation.set_research(nation, technology))
    }

    /// Force une révolution (commande de test).
    pub fn force_revolution(&mut self, nation: &NationId) -> CoreResult<()> {
        self.with_simulation(|simulation| simulation.start_revolution(nation))
    }

    /// Ajoute une sanction.
    pub fn add_sanction(
        &mut self,
        by: &[NationId],
        target: &NationId,
        kind: crate::civ_core::diplomacy::SanctionKind,
        severity: f64,
    ) -> CoreResult<()> {
        self.with_simulation(|simulation| simulation.add_sanction(by, target, kind, severity))
    }

    /// Propose une requête diplomatique visible dans la boîte de réception.
    pub fn propose_diplomatic_request(
        &mut self,
        kind: crate::civ_core::diplomacy::DiplomaticRequestKind,
        from: &NationId,
        to: &NationId,
    ) -> CoreResult<crate::civ_core::ids::DiplomaticRequestId> {
        self.with_simulation(|simulation| simulation.propose_diplomatic_request(kind, from, to))
    }

    /// Accepte ou refuse une requête pour la nation destinataire.
    pub fn respond_diplomatic_request(
        &mut self,
        request: &crate::civ_core::ids::DiplomaticRequestId,
        recipient: &NationId,
        accept: bool,
    ) -> CoreResult<()> {
        self.with_simulation(|simulation| {
            simulation.respond_diplomatic_request(request, recipient, accept)
        })
    }

    /// Annule une requête proposée par son émetteur ou son destinataire.
    pub fn cancel_diplomatic_request(
        &mut self,
        request: &crate::civ_core::ids::DiplomaticRequestId,
        actor: &NationId,
    ) -> CoreResult<()> {
        self.with_simulation(|simulation| simulation.cancel_diplomatic_request(request, actor))
    }

    /// Crée une alliance gouvernée par ses membres.
    pub fn create_alliance(
        &mut self,
        name: impl Into<String>,
        members: &[NationId],
        leader: Option<NationId>,
    ) -> CoreResult<crate::civ_core::ids::OrganizationId> {
        self.with_simulation(|simulation| simulation.create_alliance(name, members, leader))
    }

    /// Enregistre un vote pondéré dans une alliance.
    pub fn cast_alliance_vote(
        &mut self,
        alliance: &crate::civ_core::ids::OrganizationId,
        nation: &NationId,
        choice: crate::civ_core::diplomacy::VoteChoice,
    ) -> CoreResult<()> {
        self.with_simulation(|simulation| simulation.cast_alliance_vote(alliance, nation, choice))
    }

    /// Ajoute un membre à une alliance.
    pub fn add_alliance_member(
        &mut self,
        alliance: &crate::civ_core::ids::OrganizationId,
        nation: &NationId,
    ) -> CoreResult<()> {
        self.with_simulation(|simulation| simulation.add_alliance_member(alliance, nation))
    }

    /// Retire un membre d'une alliance.
    pub fn remove_alliance_member(
        &mut self,
        alliance: &crate::civ_core::ids::OrganizationId,
        nation: &NationId,
    ) -> CoreResult<()> {
        self.with_simulation(|simulation| simulation.remove_alliance_member(alliance, nation))
    }

    /// Ajuste la purse nationale d'un montant signé.
    pub fn adjust_purse(&mut self, nation: &NationId, delta: f64) -> CoreResult<()> {
        self.with_simulation(|simulation| simulation.adjust_purse(nation, delta))
    }

    /// Synchronise l'horloge et le RNG dans la sauvegarde.
    pub fn sync(&mut self) {
        self.save.tick = self.clock.tick;
        self.save.date = self.clock.date;
        self.save.rng = self.rng.state();
        self.dirty = true;
    }

    /// Scelle et retourne une copie de la sauvegarde.
    pub fn snapshot(&mut self) -> CoreResult<Save> {
        self.sync();
        self.save.seal()?;
        self.dirty = false;
        Ok(self.save.clone())
    }

    /// Envoie la sauvegarde à l'hôte.
    pub fn save_to_host(&mut self, host: &mut dyn Host, save_name: &str) -> CoreResult<()> {
        self.sync();
        self.save.metadata.save_name = save_name.to_owned();
        self.save.seal()?;
        self.dirty = false;
        let json = self.save.to_json_pretty()?;
        host.request_save(save_name, &json);
        Ok(())
    }

    /// Change la nation du joueur.
    pub fn set_player_nation(&mut self, nation: NationId) -> CoreResult<()> {
        if !self.save.nations.iter().any(|value| value.id == nation) {
            return Err(CoreError::data(format!("nation inconnue : {nation}")));
        }
        self.save.metadata.player_nation = Some(nation);
        self.dirty = true;
        Ok(())
    }

    /// Nation du joueur, si définie.
    pub fn player_nation(&self) -> Option<&NationId> {
        self.save.metadata.player_nation.as_ref()
    }

    /// Accès à la sauvegarde courante.
    pub fn save(&self) -> &Save {
        &self.save
    }

    /// Accès au monde.
    pub fn world(&self) -> &World {
        &self.save.world
    }

    /// Recherche une nation.
    pub fn nation(&self, id: &NationId) -> Option<&Nation> {
        self.save.nations.iter().find(|nation| &nation.id == id)
    }

    /// Recherche une province.
    pub fn province(&self, id: &crate::civ_core::ids::ProvinceId) -> Option<&Province> {
        self.save.world.province(id)
    }

    /// Horloge courante.
    pub fn clock(&self) -> &SimClock {
        &self.clock
    }

    /// RNG courant.
    pub fn rng(&self) -> &DeterministicRng {
        &self.rng
    }

    /// RNG mutable, réservé aux systèmes déterministes.
    pub fn rng_mut(&mut self) -> &mut DeterministicRng {
        self.dirty = true;
        &mut self.rng
    }

    /// Définitions de données actives, utilisées par les résumés d'interface.
    pub fn data(&self) -> &GameData {
        &self.data
    }

    /// Vrai si des changements ne sont pas encore scellés.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Résumé pour l'interface et les commandes.
    pub fn status(&self) -> SessionStatus {
        SessionStatus {
            date: self.clock.date,
            tick: self.clock.tick,
            provinces: self.save.world.provinces.len(),
            nations: self.save.nations.len(),
            wars: self.save.wars.len(),
            revolutions: self.save.revolutions.len(),
            player_nation: self.save.metadata.player_nation.clone(),
        }
    }
}
