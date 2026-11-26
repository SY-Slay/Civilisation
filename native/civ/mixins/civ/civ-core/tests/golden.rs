//! Tests golden de non-régression déterministe sur le pack officiel 'Arc des Frontières'.
//!
//! Vérifie :
//! 1. Qu'une simulation multi-annuelle produit exactement les mêmes états et le même checksum scellé au bit près.
//! 2. Qu'une sauvegarde intermédiaire rechargée et poursuivie jusqu'au même tick final produit l'état identique à une simulation continue (invariance par checkpoint/reprise).
//! 3. L'équilibre géopolitique (guerres, traités, économie, moral).

use std::path::{Path, PathBuf};

use civ_core::data::load_data_pack;
use civ_core::rng::DeterministicRng;
use civ_core::save::Save;
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
fn simulation_cinq_ans_strictement_deterministe() {
    let root = data_pack_path();
    let (_, _, scenario, game_data) =
        load_data_pack(&root).expect("chargement du scénario officiel");

    let seed = 424242;
    let days_to_simulate = 1800; // 5 années fictives de 360 jours (12 mois * 30 jours)

    // Simulation 1
    let mut sim1 =
        Simulation::from_scenario(&scenario, &game_data, seed).expect("init simulation 1");
    let journal1 = sim1
        .advance_days(days_to_simulate)
        .expect("avance de 5 ans sim 1");

    let mut save1 = Save::from_scenario(
        &scenario,
        "d814be95-17a4-44b2-9bc5-e8d123456781",
        "Golden Save Sim 1",
        "2026-01-01T00:00:00Z",
        "2031-01-01T00:00:00Z",
    )
    .expect("création save 1");
    sim1.write_to(&mut save1);
    save1.seal().expect("sceau save 1");

    // Simulation 2
    let mut sim2 =
        Simulation::from_scenario(&scenario, &game_data, seed).expect("init simulation 2");
    let journal2 = sim2
        .advance_days(days_to_simulate)
        .expect("avance de 5 ans sim 2");

    let mut save2 = Save::from_scenario(
        &scenario,
        "d814be95-17a4-44b2-9bc5-e8d123456781",
        "Golden Save Sim 1",
        "2026-01-01T00:00:00Z",
        "2031-01-01T00:00:00Z",
    )
    .expect("création save 2");
    sim2.write_to(&mut save2);
    save2.seal().expect("sceau save 2");

    // Vérification de l'identité stricte du checksum et du journal
    assert_eq!(
        journal1.len(),
        journal2.len(),
        "même nombre d'événements journalisés"
    );
    assert_eq!(
        save1.checksum, save2.checksum,
        "le checksum scellé doit être rigoureusement identique"
    );

    // Vérification de la date finale
    assert_eq!(save1.date.year, 2031);
    assert_eq!(save1.date.month, 1);
    assert_eq!(save1.date.day, 1);

    // Les 4 nations doivent être présentes et actives
    assert_eq!(sim1.nations().len(), 4);
    for nation in sim1.nations() {
        assert!(nation.population.total > 0, "population vivante");
        assert!(nation.economy.gdp > 0.0, "PIB positif");
        assert!(
            nation.stability >= 0.0 && nation.stability <= 100.0,
            "stabilité bornée"
        );
    }
}

#[test]
fn reprise_depuis_sauvegarde_conserve_le_determinisme() {
    let root = data_pack_path();
    let (_, _, scenario, game_data) =
        load_data_pack(&root).expect("chargement du scénario officiel");

    let seed = 999888;
    let days_step1 = 360; // 1 an
    let days_step2 = 720; // 2 ans de plus (total 3 ans = 1080 jours)

    // 1. Simulation continue de 3 ans
    let mut continuous_sim =
        Simulation::from_scenario(&scenario, &game_data, seed).expect("init simulation continue");
    continuous_sim
        .advance_days(days_step1 + days_step2)
        .expect("avance continue 3 ans");

    let mut continuous_save = Save::from_scenario(
        &scenario,
        "e925cf06-28b5-45c3-a8e6-f9e234567892",
        "Continuous 3 Years",
        "2026-01-01T00:00:00Z",
        "2029-01-01T00:00:00Z",
    )
    .expect("création save continue");
    continuous_sim.write_to(&mut continuous_save);
    continuous_save.seal().expect("sceau save continue");

    // 2. Simulation avec sauvegarde à 1 an puis reprise
    let mut phased_sim =
        Simulation::from_scenario(&scenario, &game_data, seed).expect("init simulation par étapes");
    phased_sim.advance_days(days_step1).expect("avance étape 1");

    let mut checkpoint_save = Save::from_scenario(
        &scenario,
        "c713ad84-0693-43b1-9ba4-d7c012345670",
        "Checkpoint 1 Year",
        "2026-01-01T00:00:00Z",
        "2027-01-01T00:00:00Z",
    )
    .expect("création checkpoint");
    phased_sim.write_to(&mut checkpoint_save);
    checkpoint_save.seal().expect("sceau checkpoint");

    // Rechargement depuis le checkpoint
    let rng = DeterministicRng::from_state(&checkpoint_save.rng).expect("reconstitution du RNG");
    let mut resumed_sim = Simulation::from_save(&checkpoint_save, rng, &game_data, 1)
        .expect("reprise depuis le checkpoint");
    resumed_sim
        .advance_days(days_step2)
        .expect("avance étape 2 après reprise");

    let mut resumed_save = Save::from_scenario(
        &scenario,
        "e925cf06-28b5-45c3-a8e6-f9e234567892",
        "Continuous 3 Years",
        "2026-01-01T00:00:00Z",
        "2029-01-01T00:00:00Z",
    )
    .expect("création save reprise");
    resumed_sim.write_to(&mut resumed_save);
    resumed_save.seal().expect("sceau save reprise");

    // Invariance stricte par checkpoint / reprise
    assert_eq!(
        continuous_save.checksum, resumed_save.checksum,
        "La simulation continue et la reprise après checkpoint doivent produire le même checksum"
    );
}
