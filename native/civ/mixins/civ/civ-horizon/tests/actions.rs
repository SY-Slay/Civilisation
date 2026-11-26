//! Tests des commandes Step 3 et des actions de session.

use std::path::Path;

use civ_horizon::civ_core::data::GameData;
use civ_horizon::civ_core::diplomacy::{
    DiplomaticRequestKind, DiplomaticRequestStatus, TreatyKind, VoteChoice,
};
use civ_horizon::civ_core::ids::{MissileTypeId, NationId, ProvinceId, WarId};
use civ_horizon::civ_core::nation::Nation;
use civ_horizon::civ_core::scenario::Scenario;
use civ_horizon::civ_core::text::LocalizedText;
use civ_horizon::commands::{execute_str, Command};
use civ_horizon::host::RecordingHost;
use civ_horizon::state::Session;

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("civ-core")
        .join("tests")
        .join("fixtures")
        .join(name)
}

fn nation_id(value: &str) -> NationId {
    NationId::new(value).expect("nation valide")
}

fn province_id(value: &str) -> ProvinceId {
    ProvinceId::new(value).expect("province valide")
}

fn make_nation(base: &Nation, id: &str, name: &str, color: &str, provinces: &[&str]) -> Nation {
    let mut nation = base.clone();
    nation.id = nation_id(id);
    nation.name = LocalizedText::plain(name);
    nation.adjective = LocalizedText::plain(format!("{name}ais"));
    nation.short_name = Some(LocalizedText::plain(name));
    nation.color = color.to_owned();
    nation.flag.palette = vec![color.to_owned(), "#f5f5f5".to_owned()];
    nation.provinces = provinces
        .iter()
        .map(|province| province_id(province))
        .collect();
    nation.capital = nation.provinces[0].clone();
    nation.population.total = 480_000;
    nation.population.workforce = 240_000;
    nation.population.conscription_pool = 40_000;
    nation.economy.gdp = 900_000_000.0;
    nation.economy.treasury = 25_000_000.0;
    nation.economy.debt = 0.0;
    nation.economy.tax_rate = 0.22;
    nation.economy.unemployment = 0.06;
    nation.economy.sanctions.clear();
    nation.military.army_size = 12_000;
    nation.military.manpower = 18_000;
    nation.military.navy_size = 0;
    nation.military.airforce_size = 0;
    nation.military.readiness = 60.0;
    nation.military.morale = 60.0;
    nation.military.missile_stockpile.clear();
    nation.stability = 60.0;
    nation.legitimacy = 55.0;
    nation.prestige = 40.0;
    nation.unrest = 20.0;
    nation.war_exhaustion = 0.0;
    nation.technology.era = 1;
    nation.technology.researched.clear();
    nation.technology.researching = None;
    nation.technology.research_points = 0.0;
    nation.claims.clear();
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
    let mut ardenne = make_nation(
        &base,
        "ardenne",
        "Ardenne",
        "#4c6ef5",
        &["prov_0_0", "prov_1_0", "prov_0_1", "prov_1_1"],
    );
    ardenne
        .military
        .missile_stockpile
        .insert(MissileTypeId::new("missile_short").unwrap(), 1);
    let mut kesh = make_nation(
        &base,
        "kesh",
        "Kesh",
        "#f76707",
        &["prov_0_2", "prov_1_2", "prov_0_3", "prov_1_3"],
    );
    kesh.military
        .missile_stockpile
        .insert(MissileTypeId::new("missile_short").unwrap(), 3);
    scenario.nations = vec![ardenne, kesh];
    scenario.player_nation = Some(nation_id("ardenne"));
    scenario.wars.clear();
    scenario.revolutions.clear();
    scenario.armies.clear();
    scenario.missiles_in_flight.clear();
    scenario.diplomacy.relations.clear();
    scenario.diplomacy.treaties.clear();
    scenario.diplomacy.organizations.clear();
    scenario.diplomacy.sanctions.clear();
    scenario.diplomacy.guarantees.clear();
    scenario.diplomacy.espionage_operations.clear();
    scenario.diplomacy.trade_routes.clear();

    for province in &mut scenario.world.provinces {
        province.capital_of = None;
        province.controller = province.owner.clone();
        province.supply = 85.0;
        province.infrastructure = 65.0;
        province.unrest = 15.0;
    }
    let assignments = [
        ("ardenne", ["prov_0_0", "prov_1_0", "prov_0_1", "prov_1_1"]),
        ("kesh", ["prov_0_2", "prov_1_2", "prov_0_3", "prov_1_3"]),
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
    scenario.validate().expect("scénario de commandes valide");
    scenario
}

fn session() -> Session {
    let data = GameData::from_path(&fixture("game_data.json")).expect("données de jeu");
    Session::new_with_data(
        &scenario(),
        data,
        "00000000-0000-4000-8000-000000000020",
        "Actions",
        "2026-01-01T00:00:00Z",
        1,
    )
    .expect("session")
}

#[test]
fn analyse_les_nouvelles_commandes() {
    assert_eq!(Command::parse("/civ nations").unwrap(), Command::Nations);
    assert_eq!(
        Command::parse("/civ nation ardenne").unwrap(),
        Command::Nation {
            nation: nation_id("ardenne")
        }
    );
    assert_eq!(
        Command::parse("/civ diplo ardenne kesh").unwrap(),
        Command::Diplomacy {
            first: nation_id("ardenne"),
            second: nation_id("kesh")
        }
    );
    assert_eq!(
        Command::parse("/civ war ardenne kesh prov_0_2").unwrap(),
        Command::War {
            attacker: nation_id("ardenne"),
            defender: nation_id("kesh"),
            province: Some(province_id("prov_0_2"))
        }
    );
    assert_eq!(
        Command::parse("/civ peace war_001").unwrap(),
        Command::Peace {
            war: WarId::new("war_001").unwrap()
        }
    );
    assert_eq!(
        Command::parse("/civ treaty commerce ardenne kesh").unwrap(),
        Command::Treaty {
            kind: TreatyKind::Trade,
            first: nation_id("ardenne"),
            second: nation_id("kesh")
        }
    );
    assert_eq!(
        Command::parse("/civ missile kesh missile_short prov_0_0").unwrap(),
        Command::Missile {
            nation: nation_id("kesh"),
            missile_type: MissileTypeId::new("missile_short").unwrap(),
            province: province_id("prov_0_0")
        }
    );
    assert_eq!(
        Command::parse("/civ research ardenne tech_administration").unwrap(),
        Command::Research {
            nation: nation_id("ardenne"),
            technology: civ_horizon::civ_core::ids::TechnologyId::new("tech_administration")
                .unwrap()
        }
    );
    assert_eq!(
        Command::parse("/civ event accept").unwrap(),
        Command::Event {
            choice: civ_horizon::civ_core::ids::EventChoiceId::new("accept").unwrap()
        }
    );
    assert_eq!(
        Command::parse("/civ revolution ardenne").unwrap(),
        Command::Revolution {
            nation: nation_id("ardenne")
        }
    );
    assert!(Command::parse("/civ war ardenne").is_err());
    assert!(Command::parse("/civ treaty inconnu ardenne kesh").is_err());
}

#[test]
fn execute_les_actions_step_trois() {
    let mut session = session();
    let mut host = RecordingHost::default();

    execute_str(&mut session, &mut host, "/civ tick 1").unwrap();
    execute_str(&mut session, &mut host, "/civ event refuse").unwrap();
    assert_eq!(session.save().event_state.pending.len(), 1);
    execute_str(&mut session, &mut host, "/civ tick 2").unwrap();
    assert!(session.save().event_state.pending.is_empty());
    assert_eq!(session.status().nations, 2);

    let lines = execute_str(&mut session, &mut host, "/civ nations").unwrap();
    assert!(lines.iter().any(|line| line.contains("ardenne")));
    let lines = execute_str(&mut session, &mut host, "/civ nation ardenne").unwrap();
    assert!(lines.iter().any(|line| line.contains("Ardenne")));
    let lines = execute_str(&mut session, &mut host, "/civ diplo ardenne kesh").unwrap();
    assert!(lines.iter().any(|line| line.contains("Relation")));

    execute_str(&mut session, &mut host, "/civ war ardenne kesh prov_0_2").unwrap();
    assert_eq!(session.save().wars.len(), 1);
    assert!(session.save().wars[0].is_active());
    execute_str(&mut session, &mut host, "/civ peace war_001").unwrap();
    assert!(!session.save().wars[0].is_active());
    execute_str(&mut session, &mut host, "/civ treaty trade ardenne kesh").unwrap();
    assert!(session
        .save()
        .diplomacy
        .treaties
        .iter()
        .any(|treaty| treaty.kind == TreatyKind::Trade));

    execute_str(
        &mut session,
        &mut host,
        "/civ missile kesh missile_short prov_0_0",
    )
    .unwrap();
    assert_eq!(session.save().missiles_in_flight.len(), 1);

    execute_str(
        &mut session,
        &mut host,
        "/civ research ardenne tech_administration",
    )
    .unwrap();
    assert_eq!(
        session
            .nation(&nation_id("ardenne"))
            .unwrap()
            .technology
            .researching
            .as_ref()
            .map(ToString::to_string),
        Some("tech_administration".to_owned())
    );

    execute_str(&mut session, &mut host, "/civ revolution ardenne").unwrap();
    assert_eq!(session.status().revolutions, 1);

    execute_str(&mut session, &mut host, "/civ save actions").unwrap();
    assert_eq!(host.saves.len(), 1);
    assert_eq!(host.saves[0].0, "actions");
    assert!(host.saves[0].1.contains("\"start_date\""));
}

#[test]
fn gere_les_requetes_diplomatiques_et_les_alliances() {
    let mut session = session();
    let ardenne = nation_id("ardenne");
    let kesh = nation_id("kesh");

    let request = session
        .propose_diplomatic_request(DiplomaticRequestKind::Alliance, &ardenne, &kesh)
        .expect("proposition créée");
    assert_eq!(
        session.save().diplomacy.requests[0].status,
        DiplomaticRequestStatus::Proposed
    );
    assert!(session
        .respond_diplomatic_request(&request, &ardenne, true)
        .is_err());
    session
        .respond_diplomatic_request(&request, &kesh, true)
        .expect("acceptation valide");
    assert_eq!(
        session.save().diplomacy.requests[0].status,
        DiplomaticRequestStatus::Accepted
    );

    let alliance = session
        .create_alliance(
            "Entente",
            &[ardenne.clone(), kesh.clone()],
            Some(ardenne.clone()),
        )
        .expect("alliance créée");
    session
        .cast_alliance_vote(&alliance, &ardenne, VoteChoice::Yes)
        .expect("vote valide");
    session
        .remove_alliance_member(&alliance, &kesh)
        .expect("retrait valide");
    session
        .add_alliance_member(&alliance, &kesh)
        .expect("réadmission valide");
    session
        .adjust_purse(&ardenne, 1234.0)
        .expect("purse ajustée");
    assert!(session.nation(&ardenne).unwrap().economy.purse >= 1234.0);
    assert!(session
        .save()
        .diplomacy
        .organizations
        .iter()
        .any(|organization| organization.id == alliance));

    let cancelled = session
        .propose_diplomatic_request(DiplomaticRequestKind::Trade, &ardenne, &kesh)
        .expect("demande commerciale créée");
    session
        .cancel_diplomatic_request(&cancelled, &ardenne)
        .expect("annulation valide");
    assert!(session.save().diplomacy.requests.iter().any(|request| {
        request.id == cancelled && request.status == DiplomaticRequestStatus::Cancelled
    }));
}
