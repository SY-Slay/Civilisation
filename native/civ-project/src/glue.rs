// Civilisations — adaptateur Horizon.
//
// Ce module est injecté comme `crate::game::civ_glue` dans `horizon-client`.
// Il est la seule couche qui connaît les types Horizon : `civ-core` reste pur
// et `civ-horizon` ne dépend que de son trait `Host`.

use std::cell::RefCell;
use std::collections::{HashSet, VecDeque};
use std::path::PathBuf;
use std::time::Duration;

use civ_horizon::civ_core::math::Vec3;
use civ_horizon::civ_core::save::Save;
use civ_horizon::civ_core::scenario::Scenario;
use civ_horizon::civ_core::sim::SimEventKind;
use civ_horizon::cinematic::director::CinematicDirector;
use civ_horizon::cinematic::shots::CinematicShot;
use civ_horizon::commands::{execute_str, native_command_usages, Command};
use civ_horizon::host::Host;
use civ_horizon::net::protocol::{ClientIntent, CHANNEL_CLIENT, CHANNEL_SERVER, MAX_PACKET_SIZE};
use civ_horizon::net::{self, NetMessage, NoticeLevel};
use civ_horizon::save_file::SaveFile;
use civ_horizon::state::Session;
use civ_horizon::ui::diplomacy::DiplomacyView;
use civ_horizon::ui::map::MapView;
use civ_horizon::ui::missile::MissileView;
use civ_horizon::ui::nation::NationView;
use civ_horizon::ui::theme::{colors, ColorRgba};
use civ_horizon::ui::war::WarView;
use civ_horizon::ui::widgets::{DrawCommand, Rect};
use civ_horizon::ui::UiModel;

use super::native_hooks::{NativeScreenView, NativeShake};
use super::Game;
use crate::mod_api::{NativeLifecycleContext, ServerTickPhase, WorldPhase};
use horizon_client::extras::Corner;
use horizon_client::extras_hud::Corners;
use horizon_client::hud::HudBatch;
use horizon_client::particles::{ParticleOptions, ParticleType};
use horizon_client::text::Text;

pub(crate) const ID: &str = "civ";
const MOD_NAME: &str = "Civilisations";
const MOD_VERSION: &str = "0.1.0";
const OP_PERMISSION_LEVEL: u8 = 2;
const TICKS_PER_DAY: u32 = 2400;
const SNAPSHOT_INTERVAL_TICKS: u64 = 200;
const SAVE_DIR: &str = "civ/saves";

thread_local! {
    /// Session autoritaire, uniquement sur le fil du serveur.
    static SERVER_SESSION: RefCell<Option<Session>> = RefCell::new(None);
    /// Racine du monde ouverte, pour les sauvegardes.
    static SERVER_WORLD_DIR: RefCell<Option<PathBuf>> = RefCell::new(None);
    /// Joueurs dont le client Horizon peut recevoir les messages du mod.
    static SERVER_PLAYERS: RefCell<HashSet<u128>> = RefCell::new(HashSet::new());
    /// Miroir de l'interface envoyé par le serveur au client.
    static CLIENT_MODEL: RefCell<Option<UiModel>> = RefCell::new(None);
    /// Dernières notifications reçues par le client.
    static CLIENT_NOTICES: RefCell<VecDeque<String>> = RefCell::new(VecDeque::new());
    /// État local de l'écran (onglet, carte, nation sélectionnée).
    static CLIENT_UI: RefCell<ClientUi> = RefCell::new(ClientUi::new());
    /// Directeur cinématique côté client.
    static CLIENT_DIRECTOR: RefCell<CinematicDirector> = RefCell::new(CinematicDirector::new());
    /// Secousse de caméra en cours côté client.
    static SHAKE_STATE: RefCell<ShakeState> = RefCell::new(ShakeState::default());
}

#[derive(Clone, Copy, Debug)]
enum FeedbackTarget {
    Player(u128),
    Console,
}

#[derive(Clone, Copy, Debug, Default)]
struct ShakeState {
    intensity: f32,
    remaining_seconds: f32,
}

struct ClientUi {
    tab: usize,
    map: MapView,
    nation: NationView,
    width: f32,
    height: f32,
}

impl ClientUi {
    fn new() -> Self {
        Self {
            tab: 0,
            map: MapView::new(Rect::new(0.0, 0.0, 1.0, 1.0)),
            nation: NationView::new(Rect::new(0.0, 0.0, 1.0, 1.0)),
            width: 0.0,
            height: 0.0,
        }
    }
}

/// Pont implémentant `civ_horizon::Host` pour Horizon.
pub struct HorizonHost<'a, 'g> {
    game: &'a mut Game<'g>,
    world_dir: PathBuf,
    target: FeedbackTarget,
}

impl<'a, 'g> Host for HorizonHost<'a, 'g> {
    fn info(&mut self, message: &str) {
        match self.target {
            FeedbackTarget::Player(uuid) => {
                let _ = self.game.native_tell(uuid, Text::literal(message));
            }
            FeedbackTarget::Console => {
                self.game.native_feedback(Text::literal(message));
            }
        }
    }

    fn warn(&mut self, message: &str) {
        self.info(&format!("§6[CIV] {message}"));
    }

    fn error(&mut self, message: &str) {
        self.info(&format!("§c[CIV] {message}"));
    }

    fn spawn_particle(&mut self, particle: &str, position: Vec3) {
        let options = particle_options(particle);
        let count = if matches!(
            options.ty,
            ParticleType::Explosion | ParticleType::ExplosionEmitter
        ) {
            14
        } else {
            6
        };
        let at = [position.x(), position.y(), position.z()];
        for uuid in recipients(self.game, self.target) {
            let _ = self.game.native_send_particles_to(
                uuid,
                &options,
                None,
                at,
                [0.8, 0.8, 0.8],
                0.12,
                count,
                true,
            );
        }
    }

    fn play_sound(&mut self, sound: &str, position: Vec3) {
        let at = [position.x(), position.y(), position.z()];
        for uuid in recipients(self.game, self.target) {
            let _ = self.game.native_play_sound_to(
                uuid,
                sound,
                "master",
                Some(at),
                1.0,
                1.0,
            );
        }
    }

    fn camera_shake(&mut self, intensity: f32, duration_ticks: u32) {
        let message = NetMessage::camera_shake(intensity, duration_ticks);
        if let Ok(bytes) = net::encode(&message) {
            self.send_net(CHANNEL_CLIENT, &bytes);
        }
    }

    fn send_net(&mut self, channel: &str, payload: &[u8]) {
        if payload.len() > MAX_PACKET_SIZE {
            return;
        }
        for uuid in recipients(self.game, self.target) {
            let _ = self.game.native_net_send_to_player(uuid, channel, payload);
        }
    }

    fn request_save(&mut self, save_name: &str, json: &str) {
        let path = self
            .world_dir
            .join(SAVE_DIR)
            .join(format!("{}.json", safe_file_name(save_name)));
        if let Some(parent) = path.parent() {
            if let Err(error) = std::fs::create_dir_all(parent) {
                self.error(&format!("Impossible de créer {} : {error}", parent.display()));
                return;
            }
        }
        let result = Save::from_json_str(json)
            .and_then(SaveFile::new)
            .and_then(|file| file.seal_and_write(&path));
        match result {
            Ok(()) => self.info(&format!("Partie sauvegardée dans {}", path.display())),
            Err(error) => self.error(&format!("Échec de sauvegarde : {error}")),
        }
    }
}

fn recipients(game: &Game<'_>, target: FeedbackTarget) -> Vec<u128> {
    match target {
        FeedbackTarget::Player(uuid) => vec![uuid],
        FeedbackTarget::Console => server_player_ids(game),
    }
}

fn server_player_ids(game: &Game<'_>) -> Vec<u128> {
    let mut ids = HashSet::new();
    SERVER_PLAYERS.with(|players| ids.extend(players.borrow().iter().copied()));
    for player in game.native_server_players() {
        ids.insert(player.uuid);
    }
    ids.into_iter().collect()
}

fn particle_options(name: &str) -> ParticleOptions {
    let lower = name.to_ascii_lowercase();
    let ty = if let Some(ty) = ParticleType::from_name(&lower) {
        ty
    } else if lower.contains("nuclear") || lower.contains("mushroom") {
        ParticleType::ExplosionEmitter
    } else if lower.contains("smoke") {
        ParticleType::LargeSmoke
    } else if lower.contains("fire") || lower.contains("incend") {
        ParticleType::Flame
    } else {
        ParticleType::Explosion
    };
    ParticleOptions::simple(ty)
}

fn current_world_dir(game: &Game<'_>) -> PathBuf {
    if let Some(dir) = game.native_world_dir() {
        return dir;
    }
    if let Some(dir) = SERVER_WORLD_DIR.with(|dir| dir.borrow().clone()) {
        return dir;
    }
    game.native_data_dir(ID).join("world")
}

fn safe_file_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for character in name.chars() {
        if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
            out.push(character);
        } else if character.is_whitespace() {
            out.push('_');
        }
    }
    if out.is_empty() {
        "save".to_owned()
    } else {
        out
    }
}

fn load_or_create_session(game: &Game<'_>) -> Result<Session, String> {
    let world_dir = game
        .native_world_dir()
        .ok_or_else(|| "aucun monde ouvert sur le serveur".to_owned())?;
    SERVER_WORLD_DIR.with(|dir| *dir.borrow_mut() = Some(world_dir.clone()));
    let saves = world_dir.join(SAVE_DIR);
    for name in ["autosave", "civilisations"] {
        let path = saves.join(format!("{name}.json"));
        if !path.exists() {
            continue;
        }
        let file = SaveFile::load(&path)
            .map_err(|error| format!("sauvegarde {} illisible : {error}", path.display()))?;
        return Session::from_save(file.save().clone(), TICKS_PER_DAY)
            .map_err(|error| format!("sauvegarde {} invalide : {error}", path.display()));
    }
    let scenario: Scenario = serde_json::from_slice(horizon_client::native_data!(
        "civ",
        "scenario.json"
    ))
    .map_err(|error| format!("scénario embarqué invalide : {error}"))?;
    Session::new_with_generated_id(&scenario, "Partie Horizon", "2026-11-17", TICKS_PER_DAY)
    .map_err(|error| format!("création de session impossible : {error}"))
}

fn send_message(game: &mut Game<'_>, player: u128, message: &NetMessage) {
    if let Ok(bytes) = net::encode(message) {
        if bytes.len() <= MAX_PACKET_SIZE {
            let _ = game.native_net_send_to_player(player, CHANNEL_CLIENT, &bytes);
        }
    }
}

fn send_notice(game: &mut Game<'_>, player: u128, level: NoticeLevel, message: &str) {
    send_message(game, player, &NetMessage::Notice {
        level,
        message: message.to_owned(),
    });
}

fn send_light_snapshot(game: &mut Game<'_>, player: u128) {
    let data = SERVER_SESSION.with(|session| {
        session.borrow().as_ref().map(|session| {
            let save = session.save();
            (save.tick.as_u64(), save.date, save.checksum.clone())
        })
    });
    if let Some((tick, date, checksum)) = data {
        send_message(game, player, &NetMessage::snapshot(tick, date, checksum));
    }
}

fn send_full_snapshot(game: &mut Game<'_>, player: u128) {
    let model = SERVER_SESSION.with(|session| {
        session
            .borrow()
            .as_ref()
            .map(UiModel::from_session)
    });
    let Some(model) = model else {
        send_notice(
            game,
            player,
            NoticeLevel::Warn,
            "Civilisations : aucune session active.",
        );
        return;
    };
    let message = NetMessage::full_snapshot(model);
    match net::encode(&message) {
        Ok(bytes) if bytes.len() <= MAX_PACKET_SIZE => {
            let _ = game.native_net_send_to_player(player, CHANNEL_CLIENT, &bytes);
        }
        _ => {
            send_light_snapshot(game, player);
            send_notice(
                game,
                player,
                NoticeLevel::Warn,
                "Civilisations : instantané complet trop volumineux, résumé envoyé.",
            );
        }
    }
}

fn send_full_snapshot_all(game: &mut Game<'_>) {
    for player in server_player_ids(game) {
        send_full_snapshot(game, player);
    }
}

fn send_cinematic(
    game: &mut Game<'_>,
    player: u128,
    center: [f64; 3],
    title: &str,
    subtitle: &str,
) {
    let message = NetMessage::cinematic(center, 90.0, 45.0, 7.0, title, subtitle);
    send_message(game, player, &message);
}

fn send_cinematic_all(
    game: &mut Game<'_>,
    center: [f64; 3],
    title: &str,
    subtitle: &str,
) {
    for player in server_player_ids(game) {
        send_cinematic(game, player, center, title, subtitle);
        send_message(game, player, &NetMessage::camera_shake(0.7, 24));
    }
}

fn send_notice_all(game: &mut Game<'_>, level: NoticeLevel, message: &str) {
    for player in server_player_ids(game) {
        send_notice(game, player, level, message);
        let prefix = match level {
            NoticeLevel::Info => "§e",
            NoticeLevel::Warn => "§6",
            NoticeLevel::Error => "§c",
        };
        let _ = game.native_tell(
            player,
            Text::literal(format!("{prefix}[CIV] {message}")),
        );
    }
}

struct StrikeCinematic {
    center: [f64; 3],
    title: String,
    subtitle: String,
}

fn execute_line(
    game: &mut Game<'_>,
    target: FeedbackTarget,
    line: &str,
) -> (bool, Option<StrikeCinematic>) {
    let parsed = Command::parse(line).ok();
    let missile_target = match &parsed {
        Some(Command::Missile { province, .. }) => Some(province.clone()),
        _ => None,
    };
    let world_dir = current_world_dir(game);
    let mut host = HorizonHost {
        game,
        world_dir,
        target,
    };
    let mut success = false;
    let mut strike = None;
    SERVER_SESSION.with(|session| {
        let mut session = session.borrow_mut();
        let Some(session) = session.as_mut() else {
            host.warn("Aucune session active. Ouvrez un monde ou chargez un scénario.");
            return;
        };
        match execute_str(session, &mut host, line) {
            Ok(_) => {
                success = true;
                if let Some(province_id) = missile_target {
                    if let Some(province) = session.province(&province_id) {
                        let center = province
                            .center
                            .unwrap_or_else(|| province.polygon.centroid());
                        let surface_y = session.save().world.sea_level as f64 + 2.0;
                        strike = Some(StrikeCinematic {
                            center: [center.x(), surface_y, center.z()],
                            title: "Frappe stratégique".to_owned(),
                            subtitle: format!(
                                "{} — {}", province.name.resolve(), province_id
                            ),
                        });
                    }
                }
            }
            Err(error) => host.error(&error.to_string()),
        }
    });
    drop(host);
    (success, strike)
}

// ---- Commandes ----

pub(super) fn register_commands(
    game: &mut Game<'_>,
    commands: &mut Vec<crate::mod_api::NativeCommand>,
) {
    if !game.native_on(ID) {
        return;
    }
    for usage in native_command_usages() {
        commands.push(crate::mod_api::NativeCommand::op(usage, OP_PERMISSION_LEVEL));
    }
}

pub(super) fn perform_command(
    game: &mut Game<'_>,
    player: &crate::mod_api::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    if *ran || !game.native_on(ID) {
        return;
    }
    let line = command.trim();
    if !line.starts_with("civ") && !line.starts_with("/civ") {
        return;
    }
    let (success, strike) = execute_line(game, FeedbackTarget::Player(player.uuid), line);
    if success {
        send_full_snapshot(game, player.uuid);
        if let Some(strike) = strike {
            send_cinematic(
                game,
                player.uuid,
                strike.center,
                &strike.title,
                &strike.subtitle,
            );
            send_message(game, player.uuid, &NetMessage::camera_shake(0.8, 26));
        }
    }
    *ran = true;
}

pub(super) fn console_command(game: &mut Game<'_>, command: &str, ran: &mut bool) {
    if *ran || !game.native_on(ID) {
        return;
    }
    let line = command.trim();
    if !line.starts_with("civ") && !line.starts_with("/civ") {
        return;
    }
    let _ = execute_line(game, FeedbackTarget::Console, line);
    *ran = true;
}

// ---- Cycle de vie et ticks ----

pub(super) fn server_tick(
    game: &mut Game<'_>,
    _context: &NativeLifecycleContext,
    phase: ServerTickPhase,
) {
    if !game.native_on(ID) {
        return;
    }
    let running = match phase {
        ServerTickPhase::ServerTickStart { world_running } => world_running,
        ServerTickPhase::ServerTickEnd { .. } => return,
    };
    if !running {
        return;
    }

    let mut tick = None;
    let mut notices: Vec<(NoticeLevel, String)> = Vec::new();
    let mut strikes: Vec<StrikeCinematic> = Vec::new();
    SERVER_SESSION.with(|session| {
        let mut session = session.borrow_mut();
        let Some(session) = session.as_mut() else {
            return;
        };
        match session.advance_ticks(1) {
            Ok(events) => {
                tick = Some(session.save().tick.as_u64());
                let surface_y = session.save().world.sea_level as f64 + 2.0;
                for event in events.iter().rev().take(5).rev() {
                    let level = match event.kind {
                        SimEventKind::War
                        | SimEventKind::Battle
                        | SimEventKind::Missile
                        | SimEventKind::Revolution => NoticeLevel::Warn,
                        _ => NoticeLevel::Info,
                    };
                    notices.push((level, event.message.clone()));
                }
                for event in &events {
                    if !matches!(event.kind, SimEventKind::Missile | SimEventKind::War) {
                        continue;
                    }
                    let Some(province_id) = event.province.as_ref() else {
                        continue;
                    };
                    let Some(province) = session.province(province_id) else {
                        continue;
                    };
                    let center = province
                        .center
                        .unwrap_or_else(|| province.polygon.centroid());
                    strikes.push(StrikeCinematic {
                        center: [center.x(), surface_y, center.z()],
                        title: match event.kind {
                            SimEventKind::Missile => "Frappe stratégique".to_owned(),
                            _ => "Théâtre de guerre".to_owned(),
                        },
                        subtitle: event.message.clone(),
                    });
                }
            }
            Err(error) => notices.push((
                NoticeLevel::Error,
                format!("Simulation interrompue : {error}"),
            )),
        }
    });

    for (level, message) in notices {
        send_notice_all(game, level, &message);
    }
    for strike in strikes {
        send_cinematic_all(game, strike.center, &strike.title, &strike.subtitle);
    }
    if tick.is_some_and(|tick| tick > 0 && tick % SNAPSHOT_INTERVAL_TICKS == 0) {
        send_full_snapshot_all(game);
    }
}

pub(super) fn world_lifecycle(
    game: &mut Game<'_>,
    _context: &NativeLifecycleContext,
    phase: WorldPhase,
    error_text: &str,
) {
    if !game.native_on(ID) {
        return;
    }
    match phase {
        WorldPhase::WorldOpenReady => {
            let already_open = SERVER_SESSION.with(|session| session.borrow().is_some());
            if already_open {
                return;
            }
            match load_or_create_session(game) {
                Ok(session) => {
                    SERVER_SESSION.with(|slot| *slot.borrow_mut() = Some(session));
                    let _ = game.native_feedback(Text::literal(
                        "§a[Civilisations] Simulation géopolitique initialisée.",
                    ));
                    send_full_snapshot_all(game);
                    send_notice_all(game, NoticeLevel::Info, "Civilisations : session prête.");
                }
                Err(error) => {
                    let _ = game.native_feedback(Text::literal(format!(
                        "§c[Civilisations] {error}"
                    )));
                }
            }
        }
        WorldPhase::WorldCloseFinal => {
            let world_dir = current_world_dir(game);
            let mut host = HorizonHost {
                game,
                world_dir,
                target: FeedbackTarget::Console,
            };
            SERVER_SESSION.with(|session| {
                let mut session = session.borrow_mut();
                if let Some(session) = session.as_mut() {
                    if let Err(error) = session.save_to_host(&mut host, "autosave") {
                        host.error(&format!("Sauvegarde finale impossible : {error}"));
                    }
                }
            });
            drop(host);
            SERVER_SESSION.with(|session| *session.borrow_mut() = None);
            SERVER_WORLD_DIR.with(|dir| *dir.borrow_mut() = None);
            SERVER_PLAYERS.with(|players| players.borrow_mut().clear());
        }
        WorldPhase::WorldOpenFailed => {
            SERVER_SESSION.with(|session| *session.borrow_mut() = None);
            let _ = game.native_feedback(Text::literal(format!(
                "§c[Civilisations] Ouverture du monde impossible : {error_text}"
            )));
        }
        WorldPhase::WorldCloseFailed => {
            SERVER_SESSION.with(|session| *session.borrow_mut() = None);
            let _ = game.native_feedback(Text::literal(format!(
                "§c[Civilisations] Fermeture du monde incomplète : {error_text}"
            )));
        }
    }
}

pub(super) fn player_logged_in(game: &mut Game<'_>, player: u128, _name: &str) {
    if !game.native_on(ID) {
        return;
    }
    SERVER_PLAYERS.with(|players| {
        players.borrow_mut().insert(player);
    });
    let has_session = SERVER_SESSION.with(|session| session.borrow().is_some());
    if has_session {
        send_full_snapshot(game, player);
        send_notice(
            game,
            player,
            NoticeLevel::Info,
            "Civilisations : touche J ouverte par défaut pour l'écran géopolitique.",
        );
    } else {
        send_notice(
            game,
            player,
            NoticeLevel::Warn,
            "Civilisations : session en attente de l'ouverture du monde.",
        );
    }
}

pub(super) fn player_logged_out(game: &mut Game<'_>, player: u128, _name: &str) {
    if !game.native_on(ID) {
        return;
    }
    SERVER_PLAYERS.with(|players| {
        players.borrow_mut().remove(&player);
    });
}

// ---- Entrée clavier, frame, HUD ----

pub(super) fn mods(
    game: &mut Game<'_>,
    mods: &mut Vec<crate::game::native_hooks::NativeMod>,
) {
    if !game.native_on(ID) {
        return;
    }
    mods.push(crate::game::native_hooks::NativeMod {
        id: ID.to_owned(),
        title: format!("{MOD_NAME} {MOD_VERSION}"),
        description: "Géopolitique solo déterministe : nations, diplomatie, guerres et missiles."
            .to_owned(),
        keys: vec![[
            "screen".to_owned(),
            "Ouvrir Civilisations".to_owned(),
            "key.keyboard.j".to_owned(),
        ]],
    });
}

pub(super) fn key(game: &mut Game<'_>, key: &str, pressed: bool) {
    if !pressed || !game.native_on(ID) {
        return;
    }
    let chosen = game.native_mod_key(ID, "screen", "key.keyboard.j");
    if key != chosen {
        return;
    }
    if game.native_mod_screen() == Some(ID) {
        game.native_mod_screen_close(ID);
    } else if game.native_playing() {
        let _ = game.native_mod_screen_open(ID);
    }
}

pub(super) fn frame(game: &mut Game<'_>, elapsed: Duration) {
    if !game.native_on(ID) {
        return;
    }
    let dt = elapsed.as_secs_f32().clamp(0.0, 0.25);
    let anchor = game
        .native_player()
        .map(|player| Vec3::new(player.x, player.y, player.z));
    CLIENT_DIRECTOR.with(|director| {
        let mut director = director.borrow_mut();
        if let Some(anchor) = anchor {
            director.set_player_anchor(anchor);
        }
        director.advance(dt);
    });
    SHAKE_STATE.with(|shake| {
        let mut shake = shake.borrow_mut();
        shake.remaining_seconds = (shake.remaining_seconds - dt).max(0.0);
        if shake.remaining_seconds <= 0.0 {
            shake.intensity = 0.0;
        }
    });
}

pub(super) fn hud(game: &mut Game<'_>, batch: &mut HudBatch, corners: &mut Corners) {
    if !game.native_on(ID) {
        return;
    }
    let model = CLIENT_MODEL.with(|model| model.borrow().clone());
    let line = match &model {
        Some(model) => format!(
            "Civ : {} · tick {} · {} nation(s)",
            model.date,
            model.tick.as_u64(),
            model.nations.len()
        ),
        None => "Civ : attente du serveur".to_owned(),
    };
    let notice = CLIENT_NOTICES.with(|notices| notices.borrow().back().cloned());
    let width = game
        .native_text_width(&line)
        .max(notice.as_deref().map_or(0.0, |line| game.native_text_width(line)))
        + 10.0;
    let height = if notice.is_some() { 36.0 } else { 20.0 };
    let (x, y) = corners.place(Corner::TopLeft, width.max(140.0), height);
    game.native_fill(batch, [x, y, width.max(140.0), height], 0x7800_0000);
    game.native_text(batch, &line, [x + 5.0, y + 4.0], 0x00FF_FF, true);
    if let Some(notice) = notice {
        game.native_text(batch, &notice, [x + 5.0, y + 19.0], 0xFF_FF_80, false);
    }

    CLIENT_DIRECTOR.with(|director| {
        let director = director.borrow();
        if !director.is_active() {
            return;
        }
        let mut commands = Vec::new();
        director.render_overlay(batch.width, batch.height, &mut commands);
        for command in &commands {
            draw_command(game, batch, command);
        }
    });
}

fn draw_command(game: &Game<'_>, batch: &mut HudBatch, command: &DrawCommand) {
    match command {
        DrawCommand::FillRect { rect, color } => {
            game.native_fill(
                batch,
                [rect.x, rect.y, rect.width, rect.height],
                color.to_argb_u32(),
            );
        }
        DrawCommand::StrokeRect {
            rect,
            color,
            thickness,
        } => {
            let argb = color.to_argb_u32();
            let t = thickness.max(1.0);
            game.native_fill(batch, [rect.x, rect.y, rect.width, t], argb);
            game.native_fill(
                batch,
                [rect.x, rect.y + rect.height - t, rect.width, t],
                argb,
            );
            game.native_fill(batch, [rect.x, rect.y, t, rect.height], argb);
            game.native_fill(
                batch,
                [rect.x + rect.width - t, rect.y, t, rect.height],
                argb,
            );
        }
        DrawCommand::Line {
            start,
            end,
            color,
            thickness,
        } => {
            let dx = end.0 - start.0;
            let dy = end.1 - start.1;
            let length = (dx * dx + dy * dy).sqrt();
            if length <= 0.001 {
                return;
            }
            let half = thickness.max(1.0) * 0.5;
            let nx = -dy / length * half;
            let ny = dx / length * half;
            game.native_quad(
                batch,
                [
                    [start.0 + nx, start.1 + ny],
                    [end.0 + nx, end.1 + ny],
                    [end.0 - nx, end.1 - ny],
                    [start.0 - nx, start.1 - ny],
                ],
                color.to_argb_u32(),
            );
        }
        DrawCommand::Text {
            x,
            y,
            text,
            color,
            scale,
        } => {
            let _ = scale;
            let argb = color.to_argb_u32();
            game.native_text(batch, text, [*x, *y], argb & 0x00FF_FFFF, true);
        }
    }
}

// ---- Écran de mod ----

const SCREEN_MARGIN: f32 = 18.0;
const SCREEN_HEADER: f32 = 22.0;
const SCREEN_TABS: f32 = 26.0;
const SCREEN_GAP: f32 = 6.0;
const TAB_COUNT: usize = 5;

fn screen_bounds(width: f32, height: f32) -> Rect {
    Rect::new(
        SCREEN_MARGIN,
        SCREEN_MARGIN,
        (width - SCREEN_MARGIN * 2.0).max(320.0),
        (height - SCREEN_MARGIN * 2.0).max(200.0),
    )
}

fn tab_bar(bounds: Rect) -> Rect {
    Rect::new(
        bounds.x,
        bounds.y + SCREEN_HEADER,
        bounds.width,
        SCREEN_TABS,
    )
}

fn content_rect(bounds: Rect) -> Rect {
    Rect::new(
        bounds.x,
        bounds.y + SCREEN_HEADER + SCREEN_TABS + SCREEN_GAP,
        bounds.width,
        (bounds.height - SCREEN_HEADER - SCREEN_TABS - SCREEN_GAP).max(0.0),
    )
}

fn tab_at(bounds: Rect, mouse: [f32; 2]) -> Option<usize> {
    let bar = tab_bar(bounds);
    if !bar.contains(mouse[0], mouse[1]) || bar.width <= 0.0 {
        return None;
    }
    let width = bar.width / TAB_COUNT as f32;
    let index = ((mouse[0] - bar.x) / width).floor() as usize;
    (index < TAB_COUNT).then_some(index)
}

fn mod_screen_open(game: &Game<'_>, screen: &NativeScreenView<'_>) -> bool {
    screen.id == "mod" && game.native_mod_screen() == Some(ID)
}

pub(super) fn screen_render_post(
    game: &Game<'_>,
    batch: &mut HudBatch,
    screen: &NativeScreenView<'_>,
) {
    if !mod_screen_open(game, screen) {
        return;
    }
    let bounds = screen_bounds(batch.width, batch.height);
    let content = content_rect(bounds);
    let model = CLIENT_MODEL.with(|model| model.borrow().clone());
    let Some(model) = model else {
        game.native_fill(
            batch,
            [bounds.x, bounds.y, bounds.width, bounds.height],
            ColorRgba::new(0.08, 0.09, 0.11, 0.92).to_argb_u32(),
        );
        game.native_text(
            batch,
            "Civilisations : attente de l'état serveur…",
            [bounds.x + 16.0, bounds.y + 16.0],
            0xFF_FF_FF,
            true,
        );
        return;
    };

    CLIENT_UI.with(|ui| {
        let mut ui = ui.borrow_mut();
        ui.width = batch.width;
        ui.height = batch.height;
        ui.map.bounds = content;
    });

    let mut commands = Vec::new();
    commands.push(DrawCommand::FillRect {
        rect: bounds,
        color: colors::PANEL_BG,
    });
    commands.push(DrawCommand::StrokeRect {
        rect: bounds,
        color: colors::PANEL_BORDER,
        thickness: 1.0,
    });
    commands.push(DrawCommand::Text {
        x: bounds.x + 8.0,
        y: bounds.y + 4.0,
        text: format!(
            "Civilisations — {} · tick {}",
            model.date,
            model.tick.as_u64()
        ),
        color: colors::TEXT_HIGHLIGHT,
        scale: 1.0,
    });

    let bar = tab_bar(bounds);
    commands.push(DrawCommand::FillRect {
        rect: bar,
        color: colors::HEADER_BG,
    });
    let tab_width = bar.width / TAB_COUNT as f32;
    let labels = ["Carte", "Nations", "Diplomatie", "Guerres", "Missiles"];
    let selected = CLIENT_UI.with(|ui| ui.borrow().tab);
    for (index, label) in labels.iter().enumerate() {
        let rect = Rect::new(bar.x + index as f32 * tab_width, bar.y, tab_width, bar.height);
        if index == selected {
            commands.push(DrawCommand::FillRect {
                rect,
                color: colors::ROW_HOVER,
            });
            commands.push(DrawCommand::Line {
                start: (rect.x, rect.y + rect.height - 2.0),
                end: (rect.x + rect.width, rect.y + rect.height - 2.0),
                color: colors::ACCENT,
                thickness: 2.0,
            });
        }
        commands.push(DrawCommand::Text {
            x: rect.x + 10.0,
            y: rect.y + 6.0,
            text: (*label).to_owned(),
            color: if index == selected {
                colors::TEXT_HIGHLIGHT
            } else {
                colors::TEXT_MUTED
            },
            scale: 1.0,
        });
    }

    CLIENT_UI.with(|ui| {
        let mut ui = ui.borrow_mut();
        match ui.tab {
            0 => {
                ui.map.fit_to_markers(&model.map);
                ui.map.render(&model, &mut commands);
            }
            1 => {
                let list = Rect::new(content.x, content.y, 190.0, content.height);
                let details = Rect::new(
                    content.x + list.width + 8.0,
                    content.y,
                    (content.width - list.width - 8.0).max(0.0),
                    content.height,
                );
                ui.nation.bounds = details;
                draw_nation_list(&model, list, &ui.nation, &mut commands);
                ui.nation.render(&model, &mut commands);
            }
            2 => DiplomacyView::new(content).render(&model, &mut commands),
            3 => WarView::new(content).render(&model, &mut commands),
            4 => MissileView::new(content).render(&model, &mut commands),
            _ => {}
        }
    });

    for command in &commands {
        draw_command(game, batch, command);
    }
}

fn draw_nation_list(
    model: &UiModel,
    bounds: Rect,
    view: &NationView,
    out: &mut Vec<DrawCommand>,
) {
    out.push(DrawCommand::FillRect {
        rect: bounds,
        color: colors::PANEL_BG,
    });
    out.push(DrawCommand::StrokeRect {
        rect: bounds,
        color: colors::PANEL_BORDER,
        thickness: 1.0,
    });
    let selected = view
        .selected_nation
        .as_ref()
        .or(model.player_nation.as_ref());
    for (index, nation) in model.nations.iter().enumerate() {
        let row = Rect::new(
            bounds.x + 6.0,
            bounds.y + 8.0 + index as f32 * 22.0,
            (bounds.width - 12.0).max(0.0),
            20.0,
        );
        if row.y + row.height > bounds.y + bounds.height {
            break;
        }
        if selected == Some(&nation.id) {
            out.push(DrawCommand::FillRect {
                rect: row,
                color: colors::ROW_HOVER,
            });
        }
        let color = ColorRgba::from_hex(&nation.color).unwrap_or(colors::ACCENT);
        out.push(DrawCommand::FillRect {
            rect: Rect::new(row.x + 4.0, row.y + 4.0, 12.0, 12.0),
            color,
        });
        out.push(DrawCommand::Text {
            x: row.x + 22.0,
            y: row.y + 4.0,
            text: nation.short_name.clone(),
            color: if selected == Some(&nation.id) {
                colors::TEXT_HIGHLIGHT
            } else {
                colors::TEXT_PRIMARY
            },
            scale: 1.0,
        });
    }
}

pub(super) fn screen_click(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    if !mod_screen_open(game, screen) {
        return;
    }
    *canceled = true;
    if button != 0 {
        return;
    }
    let (width, height) = CLIENT_UI.with(|ui| {
        let ui = ui.borrow();
        (ui.width, ui.height)
    });
    if width <= 0.0 || height <= 0.0 {
        return;
    }
    let bounds = screen_bounds(width, height);
    let content = content_rect(bounds);
    let mouse = screen.mouse;
    if let Some(index) = tab_at(bounds, mouse) {
        CLIENT_UI.with(|ui| ui.borrow_mut().tab = index);
        return;
    }
    let Some(model) = CLIENT_MODEL.with(|model| model.borrow().clone()) else {
        return;
    };
    CLIENT_UI.with(|ui| {
        let mut ui = ui.borrow_mut();
        match ui.tab {
            0 => {
                if content.contains(mouse[0], mouse[1]) {
                    ui.map.update_cursor(mouse[0], mouse[1], &model.map);
                    let _ = ui.map.handle_click(mouse[0], mouse[1]);
                }
            }
            1 => {
                let list = Rect::new(content.x, content.y, 190.0, content.height);
                if list.contains(mouse[0], mouse[1]) {
                    let row = ((mouse[1] - content.y - 8.0) / 22.0).floor();
                    if row >= 0.0 {
                        if let Some(nation) = model.nations.get(row as usize) {
                            ui.nation.selected_nation = Some(nation.id.clone());
                        }
                    }
                }
            }
            _ => {}
        }
    });
}

pub(super) fn screen_key(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    key: &str,
    canceled: &mut bool,
) {
    if !mod_screen_open(game, screen) {
        return;
    }
    if key == "key.keyboard.escape" {
        return;
    }
    *canceled = true;
    let map_tab = CLIENT_UI.with(|ui| ui.borrow().tab == 0);
    match key {
        "key.keyboard.1" => CLIENT_UI.with(|ui| ui.borrow_mut().tab = 0),
        "key.keyboard.2" => CLIENT_UI.with(|ui| ui.borrow_mut().tab = 1),
        "key.keyboard.3" => CLIENT_UI.with(|ui| ui.borrow_mut().tab = 2),
        "key.keyboard.4" => CLIENT_UI.with(|ui| ui.borrow_mut().tab = 3),
        "key.keyboard.5" => CLIENT_UI.with(|ui| ui.borrow_mut().tab = 4),
        "key.keyboard.tab" if map_tab => CLIENT_MODEL.with(|model| {
            let model = model.borrow();
            if let Some(model) = model.as_ref() {
                CLIENT_UI.with(|ui| {
                    ui.borrow_mut().map.handle_key("tab", &model.map);
                });
            }
        }),
        "key.keyboard.enter" if map_tab => CLIENT_MODEL.with(|model| {
            let model = model.borrow();
            if let Some(model) = model.as_ref() {
                CLIENT_UI.with(|ui| {
                    ui.borrow_mut().map.handle_key("enter", &model.map);
                });
            }
        }),
        "key.keyboard.space" if map_tab => CLIENT_MODEL.with(|model| {
            let model = model.borrow();
            if let Some(model) = model.as_ref() {
                CLIENT_UI.with(|ui| {
                    ui.borrow_mut().map.handle_key(" ", &model.map);
                });
            }
        }),
        "key.keyboard.equal" if map_tab => CLIENT_UI.with(|ui| {
            let mut ui = ui.borrow_mut();
            ui.map.zoom = (ui.map.zoom * 1.15).clamp(0.02, 8.0);
            CLIENT_MODEL.with(|model| {
                if let Some(model) = model.borrow().as_ref() {
                    ui.map.keep_focused_visible(&model.map);
                }
            });
        }),
        "key.keyboard.minus" if map_tab => CLIENT_UI.with(|ui| {
            let mut ui = ui.borrow_mut();
            ui.map.zoom = (ui.map.zoom / 1.15).clamp(0.02, 8.0);
            CLIENT_MODEL.with(|model| {
                if let Some(model) = model.borrow().as_ref() {
                    ui.map.keep_focused_visible(&model.map);
                }
            });
        }),
        "key.keyboard.left" if map_tab => CLIENT_UI.with(|ui| {
            let mut ui = ui.borrow_mut();
            ui.map.offset_x -= 24.0 / ui.map.zoom;
            CLIENT_MODEL.with(|model| {
                if let Some(model) = model.borrow().as_ref() {
                    ui.map.keep_focused_visible(&model.map);
                }
            });
        }),
        "key.keyboard.right" if map_tab => CLIENT_UI.with(|ui| {
            let mut ui = ui.borrow_mut();
            ui.map.offset_x += 24.0 / ui.map.zoom;
            CLIENT_MODEL.with(|model| {
                if let Some(model) = model.borrow().as_ref() {
                    ui.map.keep_focused_visible(&model.map);
                }
            });
        }),
        "key.keyboard.up" if map_tab => CLIENT_UI.with(|ui| {
            let mut ui = ui.borrow_mut();
            ui.map.offset_z -= 24.0 / ui.map.zoom;
            CLIENT_MODEL.with(|model| {
                if let Some(model) = model.borrow().as_ref() {
                    ui.map.keep_focused_visible(&model.map);
                }
            });
        }),
        "key.keyboard.down" if map_tab => CLIENT_UI.with(|ui| {
            let mut ui = ui.borrow_mut();
            ui.map.offset_z += 24.0 / ui.map.zoom;
            CLIENT_MODEL.with(|model| {
                if let Some(model) = model.borrow().as_ref() {
                    ui.map.keep_focused_visible(&model.map);
                }
            });
        }),
        _ => {}
    }
}

pub(super) fn screen_char(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    _c: char,
    canceled: &mut bool,
) {
    if mod_screen_open(game, screen) {
        *canceled = true;
    }
}

// ---- Caméra et main ----

pub(super) fn camera_pose(
    _game: &Game<'_>,
    _partial: f32,
    position: &mut [f64; 3],
    yaw: &mut f32,
    pitch: &mut f32,
) {
    CLIENT_DIRECTOR.with(|director| {
        let director = director.borrow();
        if let Some(pose) = director.current_camera_pose() {
            position[0] = pose.position.x();
            position[1] = pose.position.y();
            position[2] = pose.position.z();
            *yaw = pose.yaw_deg;
            *pitch = pose.pitch_deg;
        }
    });
}

pub(super) fn camera_shake(_game: &Game<'_>, partial: f32, shake: &mut NativeShake) {
    let state = SHAKE_STATE.with(|state| *state.borrow());
    if state.intensity <= 0.0 || state.remaining_seconds <= 0.0 {
        return;
    }
    let phase = (state.remaining_seconds * 40.0 + partial * 6.283_185_5).sin();
    let amplitude = state.intensity * (state.remaining_seconds * 20.0).min(1.0);
    shake.pitch += amplitude * 2.0 * phase;
    shake.yaw += amplitude * 1.1 * (phase * 0.7).cos();
    shake.roll += amplitude * 1.4 * phase;
    shake.offset[1] += amplitude * 0.12 * phase;
}

pub(super) fn render_hand(
    _game: &Game<'_>,
    _hand: horizon_client::first_person::Hand,
    _stack: Option<&horizon_client::items::ItemStack>,
    _partial: f32,
    _swing: f32,
    _equip: f32,
    canceled: &mut bool,
) {
    CLIENT_DIRECTOR.with(|director| {
        if director.borrow().should_hide_hand() {
            *canceled = true;
        }
    });
}

// ---- Réseau ----

pub(super) fn net_from_client(
    game: &mut Game<'_>,
    player: u128,
    channel: &str,
    data: &[u8],
) {
    if !game.native_on(ID) || channel != CHANNEL_SERVER || data.len() > MAX_PACKET_SIZE {
        return;
    }
    let intent = match ClientIntent::decode(data) {
        Ok(intent) => intent,
        Err(error) => {
            send_notice(
                game,
                player,
                NoticeLevel::Error,
                &format!("Message client invalide : {error}"),
            );
            return;
        }
    };
    match intent {
        ClientIntent::Command { line } => {
            let (success, strike) = execute_line(game, FeedbackTarget::Player(player), &line);
            if success {
                send_full_snapshot(game, player);
                if let Some(strike) = strike {
                    send_cinematic(
                        game,
                        player,
                        strike.center,
                        &strike.title,
                        &strike.subtitle,
                    );
                }
            }
        }
        ClientIntent::SelectNation { nation } => {
            let result = SERVER_SESSION.with(|session| {
                session
                    .borrow_mut()
                    .as_mut()
                    .ok_or_else(|| "aucune session active".to_owned())
                    .and_then(|session| {
                        session
                            .set_player_nation(nation.clone())
                            .map_err(|error| error.to_string())
                    })
            });
            match result {
                Ok(()) => {
                    send_full_snapshot(game, player);
                    send_notice(
                        game,
                        player,
                        NoticeLevel::Info,
                        &format!("Nation du joueur : {nation}"),
                    );
                }
                Err(error) => send_notice(game, player, NoticeLevel::Error, &error),
            }
        }
        ClientIntent::InspectProvince { province } => {
            let text = SERVER_SESSION.with(|session| {
                session.borrow().as_ref().and_then(|session| {
                    session.province(&province).map(|province| {
                        format!(
                            "{} ({}) · propriétaire {:?} · contrôleur {:?} · troubles {:.1}",
                            province.name.resolve(),
                            province.id,
                            province.owner,
                            province.controller,
                            province.unrest
                        )
                    })
                })
            });
            match text {
                Some(text) => send_notice(game, player, NoticeLevel::Info, &text),
                None => send_notice(
                    game,
                    player,
                    NoticeLevel::Warn,
                    "Province inconnue.",
                ),
            }
        }
        ClientIntent::ProposeDiplomaticRequest { kind, from, to } => {
            let result = SERVER_SESSION.with(|session| {
                session
                    .borrow_mut()
                    .as_mut()
                    .ok_or_else(|| "aucune session active".to_owned())
                    .and_then(|session| {
                        if session.player_nation() != Some(&from) {
                            return Err(
                                "seule la nation du joueur peut proposer une requête".to_owned(),
                            );
                        }
                        session
                            .propose_diplomatic_request(kind, &from, &to)
                            .map_err(|error| error.to_string())
                    })
            });
            match result {
                Ok(request) => {
                    send_full_snapshot(game, player);
                    send_notice(
                        game,
                        player,
                        NoticeLevel::Info,
                        &format!("Requête diplomatique créée : {request}"),
                    );
                }
                Err(error) => send_notice(game, player, NoticeLevel::Error, &error),
            }
        }
        ClientIntent::RespondDiplomaticRequest {
            request,
            recipient,
            accept,
        } => {
            let result = SERVER_SESSION.with(|session| {
                session
                    .borrow_mut()
                    .as_mut()
                    .ok_or_else(|| "aucune session active".to_owned())
                    .and_then(|session| {
                        if session.player_nation() != Some(&recipient) {
                            return Err(
                                "seule la nation destinataire peut répondre".to_owned(),
                            );
                        }
                        session
                            .respond_diplomatic_request(&request, &recipient, accept)
                            .map_err(|error| error.to_string())
                    })
            });
            match result {
                Ok(()) => {
                    send_full_snapshot(game, player);
                    send_notice(
                        game,
                        player,
                        NoticeLevel::Info,
                        if accept {
                            "Requête diplomatique acceptée."
                        } else {
                            "Requête diplomatique refusée."
                        },
                    );
                }
                Err(error) => send_notice(game, player, NoticeLevel::Error, &error),
            }
        }
        ClientIntent::CancelDiplomaticRequest { request, actor } => {
            let result = SERVER_SESSION.with(|session| {
                session
                    .borrow_mut()
                    .as_mut()
                    .ok_or_else(|| "aucune session active".to_owned())
                    .and_then(|session| {
                        let player_nation = session.player_nation().cloned();
                        if player_nation.as_ref() != Some(&actor) {
                            return Err(
                                "seule une partie de la requête peut l'annuler".to_owned(),
                            );
                        }
                        session
                            .cancel_diplomatic_request(&request, &actor)
                            .map_err(|error| error.to_string())
                    })
            });
            match result {
                Ok(()) => {
                    send_full_snapshot(game, player);
                    send_notice(
                        game,
                        player,
                        NoticeLevel::Info,
                        "Requête diplomatique annulée.",
                    );
                }
                Err(error) => send_notice(game, player, NoticeLevel::Error, &error),
            }
        }
        ClientIntent::InspectAlliance { alliance } => {
            let text = SERVER_SESSION.with(|session| {
                session.borrow().as_ref().and_then(|session| {
                    session
                        .save()
                        .diplomacy
                        .organizations
                        .iter()
                        .find(|organization| organization.id == alliance)
                        .map(|organization| {
                            format!(
                                "{} ({}) · {} membre(s) · dirigeant {:?} · cohésion {:.1} · trésorerie {:.0}",
                                organization.name.resolve(),
                                organization.id,
                                organization.members.len(),
                                organization.leader,
                                organization.cohesion,
                                organization.treasury
                            )
                        })
                })
            });
            match text {
                Some(text) => send_notice(game, player, NoticeLevel::Info, &text),
                None => send_notice(
                    game,
                    player,
                    NoticeLevel::Warn,
                    "Alliance inconnue.",
                ),
            }
        }
        ClientIntent::RequestFullSnapshot => send_full_snapshot(game, player),
    }
}

pub(super) fn net_from_server(_game: &mut Game<'_>, channel: &str, data: &[u8]) {
    if channel != CHANNEL_CLIENT || data.len() > MAX_PACKET_SIZE {
        return;
    }
    let message = match net::decode(data) {
        Ok(message) => message,
        Err(_) => return,
    };
    match message {
        NetMessage::Hello {
            protocol,
            mod_version,
        } => push_client_notice(format!(
            "Civilisations {mod_version} (protocole {protocol}) connecté."
        )),
        NetMessage::Snapshot { tick, date, .. } => {
            push_client_notice(format!("Synchronisation : {date} · tick {tick}"));
        }
        NetMessage::FullSnapshot { model, .. } => {
            CLIENT_MODEL.with(|slot| *slot.borrow_mut() = Some(model));
        }
        NetMessage::Notice { message, .. } => push_client_notice(message),
        NetMessage::CameraShake {
            intensity,
            duration_ticks,
        } => {
            SHAKE_STATE.with(|state| {
                *state.borrow_mut() = ShakeState {
                    intensity,
                    remaining_seconds: duration_ticks as f32 / 20.0,
                };
            });
        }
        NetMessage::Cinematic {
            center,
            radius,
            height,
            duration_seconds,
            title,
            subtitle,
            ..
        } => {
            let shot = CinematicShot::orbit(
                Vec3::new(center[0], center[1], center[2]),
                radius.max(16.0),
                height.max(8.0),
                duration_seconds.max(1.0),
            )
            .with_caption(title, subtitle);
            CLIENT_DIRECTOR.with(|director| director.borrow_mut().play(vec![shot]));
        }
        NetMessage::Command { .. } => {}
    }
}

fn push_client_notice(message: String) {
    CLIENT_NOTICES.with(|notices| {
        let mut notices = notices.borrow_mut();
        if notices.len() >= 6 {
            notices.pop_front();
        }
        notices.push_back(message);
    });
}
