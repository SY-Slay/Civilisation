//! Tests d'intégration de l'adaptateur, sans Horizon.

use std::path::Path;

use civ_horizon::civ_core::missile::DamageReport;
use civ_horizon::civ_core::save::Save;
use civ_horizon::civ_core::scenario::Scenario;
use civ_horizon::civ_core::time::Tick;
use civ_horizon::commands::execute_str;
use civ_horizon::host::RecordingHost;
use civ_horizon::render_effects::{
    DefaultExplosionEffects, ExplosionContext, ExplosionEffects, ExplosionKind,
};
use civ_horizon::save_file::SaveFile;
use civ_horizon::state::Session;
use civ_horizon::ui::UiModel;

fn scenario() -> Scenario {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("civ-core")
        .join("tests")
        .join("fixtures")
        .join("scenario.json");
    Scenario::from_path(&path).expect("scénario de test")
}

#[test]
fn deux_sessions_identiques_produisent_le_meme_checksum() {
    let scenario = scenario();
    let mut first = Session::new(
        &scenario,
        "00000000-0000-4000-8000-000000000002",
        "Déterminisme",
        "2026-01-01T00:00:00Z",
        2400,
    )
    .unwrap();
    let mut second = Session::new(
        &scenario,
        "00000000-0000-4000-8000-000000000002",
        "Déterminisme",
        "2026-01-01T00:00:00Z",
        2400,
    )
    .unwrap();
    first.advance_ticks(2400).unwrap();
    second.advance_ticks(2400).unwrap();
    let first_save = first.snapshot().unwrap();
    let second_save = second.snapshot().unwrap();
    assert_eq!(first_save.checksum, second_save.checksum);
    assert_eq!(first_save.tick, Tick::new(2400));
    assert_eq!(first_save.date.day, 2);
}

#[test]
fn recharger_une_sauvegarde_conserve_le_checksum() {
    let scenario = scenario();
    let mut session = Session::new(
        &scenario,
        "00000000-0000-4000-8000-000000000003",
        "Rechargement",
        "2026-01-01T00:00:00Z",
        2400,
    )
    .unwrap();
    session.advance_ticks(4800).unwrap();
    let sealed = session.snapshot().unwrap();
    let mut resumed = Session::from_save(sealed.clone(), 2400).unwrap();
    assert_eq!(resumed.status().date, sealed.date);
    let resealed = resumed.snapshot().unwrap();
    assert_eq!(sealed.checksum, resealed.checksum);
}

#[test]
fn commandes_et_sauvegarde_hote() {
    let scenario = scenario();
    let mut session = Session::new(
        &scenario,
        "00000000-0000-4000-8000-000000000004",
        "Commandes",
        "2026-01-01T00:00:00Z",
        2400,
    )
    .unwrap();
    let mut host = RecordingHost::default();
    assert!(execute_str(&mut session, &mut host, "/civ ping")
        .unwrap()
        .iter()
        .any(|line| line.contains("pong")));
    execute_str(&mut session, &mut host, "/civ tick 10").unwrap();
    execute_str(&mut session, &mut host, "/civ save intégration").unwrap();
    assert_eq!(host.saves.len(), 1);
    assert_eq!(host.saves[0].0, "intégration");
    let loaded = Save::from_json_str(&host.saves[0].1).unwrap();
    loaded.verify().unwrap();
    assert_eq!(loaded.tick, Tick::new(10));
}

#[test]
fn modele_ui_contient_la_carte_et_les_nations() {
    let scenario = scenario();
    let session = Session::new(
        &scenario,
        "00000000-0000-4000-8000-000000000005",
        "Interface",
        "2026-01-01T00:00:00Z",
        2400,
    )
    .unwrap();
    let model = UiModel::from_session(&session);
    assert_eq!(model.nations.len(), 1);
    assert_eq!(model.map.len(), 8);
    assert_eq!(model.map[0].polygon.len(), 4);
    assert_eq!(model.nations[0].provinces, 8);
    assert!(model.nations[0].is_player);
}

#[test]
fn sauvegarde_disque_atomique() {
    let scenario = scenario();
    let mut session = Session::new(
        &scenario,
        "00000000-0000-4000-8000-000000000006",
        "Disque",
        "2026-01-01T00:00:00Z",
        2400,
    )
    .unwrap();
    let save = session.snapshot().unwrap();
    let path = std::env::temp_dir().join(format!("civ_save_test_{}.json", std::process::id()));
    SaveFile::new(save.clone())
        .unwrap()
        .seal_and_write(&path)
        .unwrap();
    SaveFile::new(save).unwrap().seal_and_write(&path).unwrap();
    let backup = path.with_extension("json.bak");
    assert!(backup.exists());
    let loaded = SaveFile::load(&path).unwrap();
    loaded.save().verify().unwrap();
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&backup);
}

#[test]
fn effets_appellent_l_hote() {
    let mut host = RecordingHost::default();
    let mut effects = DefaultExplosionEffects::default();
    let context = ExplosionContext::new(
        civ_horizon::civ_core::math::Vec3::new(10.0, 64.0, 20.0),
        8.0,
        ExplosionKind::Conventional,
    );
    effects.impact(&mut host, &context, &DamageReport::default());
    assert_eq!(host.particles.len(), 1);
    assert_eq!(host.sounds.len(), 1);
    assert_eq!(host.shakes.len(), 1);
}

#[test]
fn nouvelle_session_genere_un_save_id_uuid_v4() {
    let scenario = scenario();
    let session =
        Session::new_with_generated_id(&scenario, "Session Horizon", "2026-11-18T00:00:00Z", 2400)
            .expect("création de session");

    session
        .save()
        .validate_identity()
        .expect("identité UUID v4 valide");
}
