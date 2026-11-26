use std::path::PathBuf;

use civ_core::error::CoreError;
use civ_core::rng::{DeterministicRng, RNG_ALGORITHM};
use civ_core::save::migrate::{migrate_value, CURRENT_SAVE_VERSION};
use civ_core::save::Save;
use civ_core::scenario::Scenario;
use serde_json::json;

fn scenario_fixture() -> Scenario {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("scenario.json");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("impossible de lire {}", path.display()));
    serde_json::from_str(&content).expect("scénario invalide")
}

#[test]
fn refuse_une_sauvegarde_v0() {
    let raw = json!({
        "schema_version": 0,
        "save_id": "e925cf06-28b5-45c3-a8e6-123456789abc"
    });
    let err = migrate_value(raw).unwrap_err();
    match err {
        CoreError::UnsupportedSaveVersion(0) => {}
        other => panic!("attendu UnsupportedSaveVersion(0), obtenu: {:?}", other),
    }
}

#[test]
fn refuse_une_sauvegarde_version_superieure() {
    let raw = json!({
        "schema_version": 99,
        "save_id": "e925cf06-28b5-45c3-a8e6-123456789abc"
    });
    let err = migrate_value(raw).unwrap_err();
    match err {
        CoreError::UnsupportedSaveVersion(99) => {}
        other => panic!("attendu UnsupportedSaveVersion(99), obtenu: {:?}", other),
    }
}

#[test]
fn migre_algorithme_rng_legacy_dans_json_direct() {
    let raw = json!({
        "schema_version": 1,
        "rng": {
            "algorithm": "splitmix64_pcg_v1",
            "seed": 12345,
            "counter": 42,
            "stream": 0
        }
    });
    let migrated = migrate_value(raw).expect("la migration doit réussir");
    assert_eq!(
        migrated["rng"]["algorithm"].as_str(),
        Some(RNG_ALGORITHM),
        "l'algorithme doit être migré vers la version canonique"
    );
    assert_eq!(migrated["rng"]["seed"].as_u64(), Some(12345));
    assert_eq!(migrated["rng"]["counter"].as_u64(), Some(42));
}

#[test]
fn charge_et_migre_sauvegarde_complete_avec_ancien_rng() {
    let scenario = scenario_fixture();
    let mut save = Save::from_scenario(
        &scenario,
        "e925cf06-28b5-45c3-a8e6-123456789abc",
        "Partie Test Migration",
        "2026-01-01T00:00:00Z",
        "2026-01-01T00:00:00Z",
    )
    .expect("création de sauvegarde valide");

    // Simuler un brouillon v1 ayant enregistré le nom legacy
    save.rng.algorithm = "splitmix64_pcg_v1".to_string();
    save.seal().expect("scellement de la sauvegarde d'origine");
    assert!(
        save.verify().is_ok(),
        "le sceau initial sur les données legacy doit être valide"
    );

    let json_str = save.to_json_pretty().expect("sérialisation JSON");

    // Le chargeur Save::from_json_str doit vérifier l'original, migrer le RNG,
    // re-sceller et valider l'intégrité
    let loaded =
        Save::from_json_str(&json_str).expect("chargement et migration automatiques réussis");

    assert_eq!(loaded.schema_version, CURRENT_SAVE_VERSION);
    assert_eq!(
        loaded.rng.algorithm, RNG_ALGORITHM,
        "le champ algorithme doit être normalisé"
    );
    assert!(
        loaded.verify().is_ok(),
        "la sauvegarde re-scellée doit être vérifiée avec succès"
    );

    // Vérifier que le RNG reconstruit fonctionne
    let rng = DeterministicRng::from_state(&loaded.rng)
        .expect("reconstitution du RNG depuis l'état migré");
    assert_eq!(rng.seed(), scenario.seed);
}

#[test]
fn idempotence_sur_sauvegarde_a_jour() {
    let scenario = scenario_fixture();
    let mut save = Save::from_scenario(
        &scenario,
        "e925cf06-28b5-45c3-a8e6-123456789abc",
        "Partie Idempotente",
        "2026-01-01T00:00:00Z",
        "2026-01-01T00:00:00Z",
    )
    .expect("création de sauvegarde valide");
    save.seal().expect("scellement");

    let original_value = serde_json::to_value(&save).expect("to_value");
    let migrated_value = migrate_value(original_value.clone()).expect("migration");
    assert_eq!(
        original_value, migrated_value,
        "une sauvegarde déjà à jour ne doit subir aucune mutation"
    );
}
