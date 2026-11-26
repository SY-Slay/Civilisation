//! Test d'intégration du pack de données officiel 'Arc des Frontières'.
//!
//! Vérifie le chargement complet depuis le disque (`data/civ/`),
//! la cohérence des registres, la validation croisée et le déroulement de la simulation.

use std::path::{Path, PathBuf};

use civ_core::data::load_data_pack;
use civ_core::sim::Simulation;

fn data_pack_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(5)
        .expect("racine du projet Horizon")
        .join("data")
        .join("civ")
}

#[test]
fn charge_le_pack_de_donnees_officiel() {
    let root = data_pack_path();
    assert!(root.exists(), "le dossier data/civ doit exister");

    let (manifest, registries, scenario, game_data) =
        load_data_pack(&root).expect("chargement et validation du data-pack");

    // 1. Vérification du manifeste
    assert_eq!(manifest.id, "civ");
    assert_eq!(manifest.version, "0.1.0");

    // 2. Vérification des registres
    assert_eq!(registries.cultures.len(), 4);
    assert_eq!(registries.religions.len(), 4);
    assert_eq!(registries.ideologies.len(), 4);
    assert_eq!(registries.governments.len(), 4);
    assert_eq!(registries.resources.len(), 5);
    assert_eq!(registries.units.len(), 5);

    // 3. Vérification des données de jeu
    assert_eq!(game_data.technologies.len(), 6);
    assert_eq!(game_data.missiles.len(), 3);
    assert_eq!(game_data.events.len(), 5);

    // 4. Vérification du scénario
    assert_eq!(scenario.id.as_str(), "arc_des_frontieres");
    assert_eq!(scenario.nations.len(), 4);
    assert_eq!(scenario.world.provinces.len(), 16);
    assert_eq!(scenario.world.borders.len(), 24);
    assert_eq!(scenario.diplomacy.relations.len(), 6);

    // Vérification des 4 capitales et nations
    for nation in &scenario.nations {
        assert!(nation.provinces.contains(&nation.capital));
        assert_eq!(nation.provinces.len(), 4);
    }
}

#[test]
fn execute_simulation_sur_scenario_officiel() {
    let root = data_pack_path();
    let (_, _, scenario, game_data) =
        load_data_pack(&root).expect("chargement du scénario officiel");

    let mut sim = Simulation::from_scenario(&scenario, &game_data, 42)
        .expect("initialisation de la simulation sur le pack officiel");

    // Avance de 60 jours
    let journal = sim.advance_days(60).expect("simulation de 60 jours");
    assert!(
        !journal.is_empty(),
        "le journal de simulation doit contenir des événements"
    );

    // L'horloge doit avoir progressé exactement de 60 jours
    assert_eq!(sim.date().month, 3);
    assert_eq!(sim.date().day, 1);
}
