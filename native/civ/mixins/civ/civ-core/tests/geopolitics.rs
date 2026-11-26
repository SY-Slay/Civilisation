//! Test de bout en bout : dix années fictives, sans intervention du joueur.
//!
//! Le scénario est construit à partir de la fixture minimale pour rester
//! reproductible et indépendant d'un contenu de production.

use std::collections::BTreeMap;
use std::path::Path;

use civ_core::data::GameData;
use civ_core::diplomacy::{Diplomacy, Relation, RelationStatus};
use civ_core::ids::{MissileTypeId, NationId, ProvinceId};
use civ_core::nation::model::{AiArchetype, Claim, ClaimKind};
use civ_core::nation::Nation;
use civ_core::rng::DeterministicRng;
use civ_core::save::Save;
use civ_core::scenario::{Difficulty, Scenario};
use civ_core::sim::Simulation;
use civ_core::text::LocalizedText;
use civ_core::war::WarStatus;

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn nation_id(value: &str) -> NationId {
    NationId::new(value).expect("identifiant de nation valide")
}

fn province_id(value: &str) -> ProvinceId {
    ProvinceId::new(value).expect("identifiant de province valide")
}

#[allow(clippy::too_many_arguments)]
fn make_nation(
    base: &Nation,
    id: &str,
    name: &str,
    color: &str,
    provinces: &[&str],
    gdp: f64,
    treasury: f64,
    army: u64,
    manpower: u64,
    stability: f64,
    legitimacy: f64,
    unrest: f64,
    aggression: f64,
    caution: f64,
    honor: f64,
    greed: f64,
    revanchism: f64,
    isolationism: f64,
    opportunism: f64,
    war_threshold: f64,
    archetype: AiArchetype,
    claim: Option<&str>,
) -> Nation {
    let mut nation = base.clone();
    nation.id = nation_id(id);
    nation.name = LocalizedText::plain(name);
    nation.adjective = LocalizedText::plain(format!("{name}ais"));
    nation.short_name = Some(LocalizedText::plain(name));
    nation.color = color.to_owned();
    nation.flag.palette = vec![color.to_owned(), "#f2f2f2".to_owned()];
    nation.provinces = provinces.iter().map(|value| province_id(value)).collect();
    nation.capital = nation.provinces[0].clone();
    nation.population.total = 240_000;
    nation.population.workforce = 120_000;
    nation.population.conscription_pool = 20_000;
    nation.economy.gdp = gdp;
    nation.economy.treasury = treasury;
    nation.economy.debt = 0.0;
    nation.economy.tax_rate = 0.22;
    nation.economy.inflation = 0.02;
    nation.economy.unemployment = 0.06;
    nation.economy.sanctions.clear();
    nation.military.army_size = army;
    nation.military.manpower = manpower;
    nation.military.navy_size = 0;
    nation.military.airforce_size = 0;
    nation.military.readiness = if aggression > 0.6 { 78.0 } else { 52.0 };
    nation.military.morale = if aggression > 0.6 { 74.0 } else { 58.0 };
    nation.military.missile_stockpile.clear();
    if id == "kesh" {
        nation
            .military
            .missile_stockpile
            .insert(MissileTypeId::new("missile_short").expect("type valide"), 4);
        nation.military.missile_stockpile.insert(
            MissileTypeId::new("missile_cruise").expect("type valide"),
            2,
        );
    }
    nation.stability = stability;
    nation.legitimacy = legitimacy;
    nation.prestige = 45.0;
    nation.war_exhaustion = 0.0;
    nation.unrest = unrest;
    nation.technology.era = 1;
    nation.technology.researched.clear();
    nation.technology.researching = None;
    nation.technology.research_points = 0.0;
    nation.technology.research_speed = 1.0;
    nation.ai_profile.archetype = archetype;
    nation.ai_profile.aggression = aggression;
    nation.ai_profile.caution = caution;
    nation.ai_profile.honor = honor;
    nation.ai_profile.greed = greed;
    nation.ai_profile.revanchism = revanchism;
    nation.ai_profile.isolationism = isolationism;
    nation.ai_profile.opportunism = opportunism;
    nation.ai_profile.loyalty = 0.5;
    nation.ai_profile.risk_tolerance = aggression;
    nation.ai_profile.patience = 0.5;
    nation.ai_profile.war_threshold = war_threshold;
    nation.ai_profile.peace_threshold = 0.4;
    nation.ai_profile.red_lines.clear();
    nation.claims.clear();
    if let Some(province) = claim {
        nation.claims.push(Claim {
            province: province_id(province),
            kind: ClaimKind::Revanchist,
            priority: 0.9,
            since: Some(
                base.founded
                    .unwrap_or(civ_core::time::GameDate::new(2020, 1, 1).expect("date valide")),
            ),
        });
    }
    nation.subjects.clear();
    nation.overlord = None;
    nation.allies.clear();
    nation.wars.clear();
    nation.revolutions.clear();
    nation.armies.clear();
    nation.missiles.clear();
    nation.is_playable = true;
    nation
}

fn scenario() -> Scenario {
    let mut scenario = Scenario::from_path(&fixture("scenario.json")).expect("scénario minimal");
    let base = scenario.nations[0].clone();
    let ardenne = make_nation(
        &base,
        "ardenne",
        "Ardenne",
        "#5c7cfa",
        &["prov_0_0", "prov_1_0"],
        620_000_000.0,
        8_000_000.0,
        8_000,
        10_000,
        22.0,
        18.0,
        78.0,
        0.15,
        0.80,
        0.70,
        0.20,
        0.10,
        0.60,
        0.10,
        0.70,
        AiArchetype::Caretaker,
        None,
    );
    let kesh = make_nation(
        &base,
        "kesh",
        "Kesh",
        "#e8590c",
        &["prov_0_1", "prov_1_1"],
        910_000_000.0,
        55_000_000.0,
        28_000,
        30_000,
        64.0,
        61.0,
        18.0,
        0.95,
        0.15,
        0.20,
        0.85,
        0.80,
        0.10,
        0.90,
        0.10,
        AiArchetype::Warlord,
        Some("prov_1_0"),
    );
    let varenne = make_nation(
        &base,
        "varenne",
        "Varenne",
        "#12b886",
        &["prov_0_2", "prov_1_2"],
        1_180_000_000.0,
        80_000_000.0,
        10_000,
        12_000,
        70.0,
        66.0,
        12.0,
        0.10,
        0.60,
        0.60,
        0.90,
        0.10,
        0.40,
        0.40,
        0.80,
        AiArchetype::Merchant,
        None,
    );
    let orlune = make_nation(
        &base,
        "orlune",
        "Orlune",
        "#9c36b5",
        &["prov_0_3", "prov_1_3"],
        720_000_000.0,
        30_000_000.0,
        18_000,
        22_000,
        55.0,
        45.0,
        30.0,
        0.40,
        0.40,
        0.50,
        0.50,
        0.30,
        0.30,
        0.50,
        0.50,
        AiArchetype::Ideologue,
        None,
    );

    scenario.nations = vec![ardenne, kesh, varenne, orlune];
    scenario.wars.clear();
    scenario.revolutions.clear();
    scenario.armies.clear();
    scenario.missiles_in_flight.clear();
    scenario.player_nation = None;
    scenario.difficulty = Difficulty::Aggressive;
    scenario.diplomacy = Diplomacy {
        schema_version: 1,
        relations: vec![
            Relation {
                a: nation_id("ardenne"),
                b: nation_id("kesh"),
                opinion: -65.0,
                trust: 20.0,
                fear: 50.0,
                trade_dependence: 0.05,
                border_tension: 85.0,
                status: RelationStatus::Hostile,
                embassy: false,
                contact: true,
                last_change_tick: None,
                modifiers: Vec::new(),
            },
            Relation {
                a: nation_id("orlune"),
                b: nation_id("varenne"),
                opinion: 35.0,
                trust: 65.0,
                fear: 0.0,
                trade_dependence: 0.2,
                border_tension: 10.0,
                status: RelationStatus::Neutral,
                embassy: true,
                contact: true,
                last_change_tick: None,
                modifiers: Vec::new(),
            },
        ],
        treaties: Vec::new(),
        organizations: Vec::new(),
        sanctions: Vec::new(),
        guarantees: Vec::new(),
        espionage_operations: Vec::new(),
        trade_routes: Vec::new(),
        requests: Vec::new(),
    };

    for province in &mut scenario.world.provinces {
        province.capital_of = None;
        province.controller = province.owner.clone();
        province.supply = 82.0;
        province.infrastructure = 62.0;
        province.population.total = 120_000;
        province.population.urban = 48_000;
        province.population.rural = 72_000;
        province.unrest = match province.owner.as_ref().map(NationId::as_str) {
            Some("ardenne") => 82.0,
            Some("orlune") => 34.0,
            _ => 12.0,
        };
    }

    let assignments = [
        ("ardenne", ["prov_0_0", "prov_1_0"]),
        ("kesh", ["prov_0_1", "prov_1_1"]),
        ("varenne", ["prov_0_2", "prov_1_2"]),
        ("orlune", ["prov_0_3", "prov_1_3"]),
    ];
    for (nation, provinces) in assignments {
        for province in provinces {
            if let Some(value) = scenario.world.province_mut(&province_id(province)) {
                value.owner = Some(nation_id(nation));
                value.controller = Some(nation_id(nation));
            }
        }
        if let Some(value) = scenario.world.province_mut(&province_id(provinces[0])) {
            value.capital_of = Some(nation_id(nation));
        }
    }

    scenario.validate().expect("scénario géopolitique valide");
    scenario
}

#[test]
fn dix_ans_produisent_guerre_paix_et_revolution() {
    let scenario = scenario();
    let data = GameData::from_path(&fixture("game_data.json")).expect("données de jeu");
    let mut simulation = Simulation::from_scenario(&scenario, &data, 1).expect("simulation");
    let events = simulation.advance_days(3_600).expect("dix années fictives");
    assert!(
        !events.is_empty(),
        "le journal doit contenir des événements"
    );
    assert!(
        simulation
            .wars()
            .iter()
            .any(|war| war.status == WarStatus::Ended || war.is_active()),
        "au moins une guerre doit éclater"
    );
    assert!(
        !simulation.diplomacy().treaties.is_empty(),
        "au moins un traité doit être signé"
    );
    assert!(
        !simulation.revolutions().is_empty(),
        "au moins une révolution doit démarrer"
    );
    assert!(
        simulation
            .wars()
            .iter()
            .any(|war| war.status == WarStatus::Ended),
        "la guerre doit se terminer par une paix"
    );
}

#[test]
fn deux_simulations_identiques_ont_le_meme_etat() {
    let scenario = scenario();
    let data = GameData::from_path(&fixture("game_data.json")).expect("données de jeu");
    let mut first = Simulation::from_scenario(&scenario, &data, 1).expect("simulation");
    let mut second = Simulation::from_scenario(&scenario, &data, 1).expect("simulation");
    first.advance_days(1_080).expect("trois ans");
    second.advance_days(1_080).expect("trois ans");

    let mut first_save = Save::from_scenario(
        &scenario,
        "00000000-0000-4000-8000-000000000010",
        "Déterminisme géopolitique",
        "2026-01-01T00:00:00Z",
        "2026-01-01T00:00:00Z",
    )
    .expect("sauvegarde");
    let mut second_save = Save::from_scenario(
        &scenario,
        "00000000-0000-4000-8000-000000000010",
        "Déterminisme géopolitique",
        "2026-01-01T00:00:00Z",
        "2026-01-01T00:00:00Z",
    )
    .expect("sauvegarde");
    first.write_to(&mut first_save);
    second.write_to(&mut second_save);
    first_save.seal().expect("sceau 1");
    second_save.seal().expect("sceau 2");
    assert_eq!(first_save.checksum, second_save.checksum);
}

#[test]
fn recharger_conserve_horloge_et_difficulte() {
    let scenario = scenario();
    let data = GameData::from_path(&fixture("game_data.json")).expect("données de jeu");
    let mut continuous = Simulation::from_scenario(&scenario, &data, 1).expect("simulation");
    continuous.advance_days(120).expect("cent vingt jours");

    let mut snapshot = Save::from_scenario(
        &scenario,
        "00000000-0000-4000-8000-000000000012",
        "Rechargement géopolitique",
        "2026-01-01T00:00:00Z",
        "2026-01-01T00:00:00Z",
    )
    .expect("sauvegarde");
    continuous.write_to(&mut snapshot);
    snapshot.seal().expect("sceau");
    let rng = DeterministicRng::from_state(&snapshot.rng).expect("rng");
    let mut resumed =
        Simulation::from_save(&snapshot, rng, &data, 1).expect("reprise de simulation");

    continuous.advance_days(120).expect("suite continue");
    resumed.advance_days(120).expect("suite rechargée");
    let mut expected = Save::from_scenario(
        &scenario,
        "00000000-0000-4000-8000-000000000012",
        "Rechargement géopolitique",
        "2026-01-01T00:00:00Z",
        "2026-01-01T00:00:00Z",
    )
    .expect("sauvegarde");
    let mut actual = expected.clone();
    continuous.write_to(&mut expected);
    resumed.write_to(&mut actual);
    expected.seal().expect("sceau attendu");
    actual.seal().expect("sceau réel");
    assert_eq!(expected.checksum, actual.checksum);
}

#[test]
fn les_donnees_de_jeu_se_chargent() {
    let data = GameData::from_path(&fixture("game_data.json")).expect("données de jeu");
    assert_eq!(data.technologies.len(), 3);
    assert_eq!(data.missiles.len(), 2);
    assert_eq!(data.events.len(), 4);
    let _: BTreeMap<MissileTypeId, u64> = BTreeMap::new();
}
