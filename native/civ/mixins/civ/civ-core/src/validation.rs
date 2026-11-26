//! Validation des données chargées.
//!
//! La validation est volontairement indépendante de JSON Schema : elle
//! vérifie les invariants du moteur (réciprocité, références, bornes) et
//! produit un message explicite au premier lot d'erreurs.

use std::collections::{BTreeMap, BTreeSet};

use crate::diplomacy::Diplomacy;
use crate::error::{CoreError, CoreResult};
use crate::ids::{BattleId, NationId, ProvinceId, TreatyId, WarId};
use crate::missile::MissileInstance;
use crate::nation::Nation;
use crate::revolution::Revolution;
use crate::scenario::Scenario;
use crate::war::{Army, War};
use crate::world::{Border, Province, World};

pub(crate) fn finish(errors: Vec<String>) -> CoreResult<()> {
    if errors.is_empty() {
        Ok(())
    } else {
        Err(CoreError::InvalidData(errors.join(" ; ")))
    }
}

fn check_unique<'a, I, T>(ids: I, label: &str, errors: &mut Vec<String>)
where
    I: IntoIterator<Item = &'a T>,
    T: std::fmt::Display + 'a,
{
    let mut seen = BTreeSet::new();
    for id in ids {
        let value = id.to_string();
        if !seen.insert(value.clone()) {
            errors.push(format!("{label} dupliqué : {value}"));
        }
    }
}

fn check_unique_debug<T: std::fmt::Debug>(values: &[T], label: &str, errors: &mut Vec<String>) {
    let mut seen = BTreeSet::new();
    for value in values {
        let key = format!("{value:?}");
        if !seen.insert(key.clone()) {
            errors.push(format!("{label} dupliqué : {key}"));
        }
    }
}

fn is_unit(value: f64) -> bool {
    (0.0..=1.0).contains(&value)
}

fn is_percentage(value: f64) -> bool {
    (0.0..=100.0).contains(&value)
}

fn is_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn check_unit(value: f64, label: &str, errors: &mut Vec<String>) {
    if !is_unit(value) {
        errors.push(format!("{label} doit être dans [0,1] : {value}"));
    }
}

fn check_percentage(value: f64, label: &str, errors: &mut Vec<String>) {
    if !is_percentage(value) {
        errors.push(format!("{label} doit être dans [0,100] : {value}"));
    }
}

fn check_date(date: crate::time::GameDate, label: &str, errors: &mut Vec<String>) {
    if let Err(error) = date.validate() {
        errors.push(format!("{label} : {error}"));
    }
}

fn check_optional_date(date: Option<crate::time::GameDate>, label: &str, errors: &mut Vec<String>) {
    if let Some(date) = date {
        check_date(date, label, errors);
    }
}

/// Valide un monde sans contexte de nations.
pub fn validate_world(world: &World) -> CoreResult<()> {
    let mut errors = Vec::new();
    validate_world_into(world, None, &mut errors);
    finish(errors)
}

/// Valide un scénario complet.
pub fn validate_scenario(scenario: &Scenario) -> CoreResult<()> {
    let mut errors = Vec::new();
    if scenario.schema_version == 0 {
        errors.push("scénario : schema_version doit être >= 1".to_owned());
    }
    if scenario.seed > i64::MAX as u64 {
        errors.push("scénario : seed hors bornes".to_owned());
    }
    check_date(scenario.start_date, "scénario.start_date", &mut errors);

    let nation_ids: BTreeSet<NationId> = scenario
        .nations
        .iter()
        .map(|nation| nation.id.clone())
        .collect();
    check_unique(
        scenario.nations.iter().map(|nation| &nation.id),
        "nation",
        &mut errors,
    );

    validate_world_into(&scenario.world, Some(&nation_ids), &mut errors);

    for nation in &scenario.nations {
        validate_nation(nation, &scenario.world, &nation_ids, &mut errors);
    }

    validate_diplomacy(
        &scenario.diplomacy,
        &scenario.world,
        &nation_ids,
        &mut errors,
    );
    validate_wars(
        &scenario.wars,
        &scenario.world,
        &nation_ids,
        &scenario.diplomacy,
        &mut errors,
    );
    validate_revolutions(
        &scenario.revolutions,
        &scenario.world,
        &nation_ids,
        &mut errors,
    );
    validate_armies(
        &scenario.armies,
        &scenario.world,
        &nation_ids,
        &scenario.wars,
        &mut errors,
    );
    validate_missiles(
        &scenario.missiles_in_flight,
        &scenario.world,
        &nation_ids,
        &mut errors,
    );

    if let Some(player) = &scenario.player_nation {
        if !nation_ids.contains(player) {
            errors.push(format!("player_nation inconnue : {player}"));
        }
    }

    // Les listes d'identifiants portées par chaque nation doivent référencer
    // des objets présents dans le scénario.
    let war_ids: BTreeSet<WarId> = scenario.wars.iter().map(|war| war.id.clone()).collect();
    let revolution_ids: BTreeSet<String> = scenario
        .revolutions
        .iter()
        .map(|revolution| revolution.id.to_string())
        .collect();
    let army_ids: BTreeSet<String> = scenario
        .armies
        .iter()
        .map(|army| army.id.to_string())
        .collect();
    let missile_ids: BTreeSet<String> = scenario
        .missiles_in_flight
        .iter()
        .map(|missile| missile.id.to_string())
        .collect();
    for nation in &scenario.nations {
        for war in &nation.wars {
            if !war_ids.contains(war) {
                errors.push(format!(
                    "nation {} référence la guerre inconnue {}",
                    nation.id, war
                ));
            }
        }
        for revolution in &nation.revolutions {
            if !revolution_ids.contains(&revolution.to_string()) {
                errors.push(format!(
                    "nation {} référence la révolution inconnue {}",
                    nation.id, revolution
                ));
            }
        }
        for army in &nation.armies {
            if !army_ids.contains(&army.to_string()) {
                errors.push(format!(
                    "nation {} référence l'armée inconnue {}",
                    nation.id, army
                ));
            }
        }
        for missile in &nation.missiles {
            if !missile_ids.contains(&missile.to_string()) {
                errors.push(format!(
                    "nation {} référence le missile inconnu {}",
                    nation.id, missile
                ));
            }
        }
    }

    if !scenario.diplomacy.relations.is_empty() {
        for relation in &scenario.diplomacy.relations {
            if !nation_ids.contains(&relation.a) || !nation_ids.contains(&relation.b) {
                errors.push(format!(
                    "relation {} / {} référence une nation absente",
                    relation.a, relation.b
                ));
            }
        }
    }

    finish(errors)
}

pub(crate) fn validate_world_into(
    world: &World,
    nation_ids: Option<&BTreeSet<NationId>>,
    errors: &mut Vec<String>,
) {
    if world.schema_version == 0 {
        errors.push("monde : schema_version doit être >= 1".to_owned());
    }
    if world.seed > i64::MAX as u64 {
        errors.push("monde : seed hors bornes".to_owned());
    }
    if !(512..=32_768).contains(&world.size.width) {
        errors.push(format!("monde : largeur invalide {}", world.size.width));
    }
    if !(512..=32_768).contains(&world.size.height) {
        errors.push(format!("monde : hauteur invalide {}", world.size.height));
    }
    if !(-64..=320).contains(&world.sea_level) {
        errors.push(format!("monde : sea_level invalide {}", world.sea_level));
    }
    if world.provinces.len() < 8 {
        errors.push("monde : au moins 8 provinces sont requises".to_owned());
    }

    check_unique(
        world.provinces.iter().map(|province| &province.id),
        "province",
        errors,
    );
    check_unique(
        world.borders.iter().map(|border| &border.id),
        "frontière",
        errors,
    );
    check_unique(
        world.regions.iter().map(|region| &region.id),
        "région",
        errors,
    );
    check_unique(
        world.straits.iter().map(|strait| &strait.id),
        "détroit",
        errors,
    );
    check_unique(
        world
            .climate_zones
            .iter()
            .map(|climate_zone| &climate_zone.id),
        "zone climatique",
        errors,
    );

    let province_ids: BTreeSet<ProvinceId> = world
        .provinces
        .iter()
        .map(|province| province.id.clone())
        .collect();

    for province in &world.provinces {
        validate_province(province, &province_ids, nation_ids, errors);
    }

    let mut border_references: BTreeMap<String, usize> = BTreeMap::new();
    for province in &world.provinces {
        for border_id in &province.border_ids {
            *border_references.entry(border_id.to_string()).or_insert(0) += 1;
        }
    }

    for border in &world.borders {
        validate_border(border, &province_ids, errors);
        let references = border_references
            .get(&border.id.to_string())
            .copied()
            .unwrap_or(0);
        if references != 2 {
            errors.push(format!(
                "frontière {} référencée {} fois au lieu de 2",
                border.id, references
            ));
        }
        if let (Some(first), Some(second)) = (world.province(&border.a), world.province(&border.b))
        {
            if !first.neighbors.contains(&border.b) || !second.neighbors.contains(&border.a) {
                errors.push(format!(
                    "frontière {} n'est pas réciproque entre {} et {}",
                    border.id, border.a, border.b
                ));
            }
        }
    }

    for province in &world.provinces {
        for border_id in &province.border_ids {
            if !world.borders.iter().any(|border| &border.id == border_id) {
                errors.push(format!(
                    "province {} référence la frontière inconnue {}",
                    province.id, border_id
                ));
            }
        }
        for neighbor in &province.neighbors {
            if !province_ids.contains(neighbor) {
                errors.push(format!(
                    "province {} référence le voisin inconnu {}",
                    province.id, neighbor
                ));
            } else if let Some(other) = world.province(neighbor) {
                if !other.neighbors.contains(&province.id) {
                    errors.push(format!(
                        "voisinage non réciproque entre {} et {}",
                        province.id, neighbor
                    ));
                }
            }
        }
    }

    for region in &world.regions {
        for province in &region.provinces {
            if !province_ids.contains(province) {
                errors.push(format!(
                    "région {} référence la province inconnue {}",
                    region.id, province
                ));
            }
        }
    }
    for strait in &world.straits {
        if !(1..=32_768).contains(&strait.width_blocks) {
            errors.push(format!(
                "détroit {} : largeur invalide {}",
                strait.id, strait.width_blocks
            ));
        }
        for province in &strait.provinces {
            if !province_ids.contains(province) {
                errors.push(format!(
                    "détroit {} référence la province inconnue {}",
                    strait.id, province
                ));
            }
        }
        if let Some(nation) = &strait.blocked_by {
            if let Some(known) = nation_ids {
                if !known.contains(nation) {
                    errors.push(format!(
                        "détroit {} bloqué par une nation inconnue {}",
                        strait.id, nation
                    ));
                }
            }
        }
    }
}

fn validate_province(
    province: &Province,
    province_ids: &BTreeSet<ProvinceId>,
    nation_ids: Option<&BTreeSet<NationId>>,
    errors: &mut Vec<String>,
) {
    if let Err(error) = province.polygon.validate() {
        errors.push(format!(
            "province {} : polygone invalide ({error})",
            province.id
        ));
    }
    if let Some(area) = province.area_blocks {
        if area < 64 {
            errors.push(format!("province {} : area_blocks < 64", province.id));
        }
    }
    if !(-80.0..=60.0).contains(&province.climate.temperature_c) {
        errors.push(format!("province {} : température invalide", province.id));
    }
    if !(0.0..=20_000.0).contains(&province.climate.rainfall_mm) {
        errors.push(format!("province {} : pluviométrie invalide", province.id));
    }
    check_unit(
        province.climate.seasonality,
        &format!("province {}.climate.seasonality", province.id),
        errors,
    );
    if province.population.total != province.population.urban + province.population.rural {
        errors.push(format!(
            "province {} : population totale différente de urbaine + rurale",
            province.id
        ));
    }
    if !(-0.2..=0.2).contains(&province.population.growth_rate) {
        errors.push(format!("province {} : growth_rate invalide", province.id));
    }
    if let Some(density) = province.population.density_per_km2 {
        if density < 0.0 {
            errors.push(format!("province {} : densité négative", province.id));
        }
    }
    for minority in &province.culture.minorities {
        check_unit(
            minority.share,
            &format!("province {}.culture.minority.share", province.id),
            errors,
        );
    }
    for minority in &province.religion.minorities {
        check_unit(
            minority.share,
            &format!("province {}.religion.minority.share", province.id),
            errors,
        );
    }
    for resource in &province.resources {
        if !(0.0..=100.0).contains(&resource.abundance) {
            errors.push(format!(
                "province {} : abondance de ressource invalide",
                province.id
            ));
        }
        if resource.reserve.is_some_and(|value| value < 0.0)
            || resource.extraction_rate.is_some_and(|value| value < 0.0)
        {
            errors.push(format!(
                "province {} : réserve ou extraction négative",
                province.id
            ));
        }
    }
    check_percentage(
        province.unrest,
        &format!("province {}.unrest", province.id),
        errors,
    );
    check_percentage(
        province.supply,
        &format!("province {}.supply", province.id),
        errors,
    );
    check_percentage(
        province.infrastructure,
        &format!("province {}.infrastructure", province.id),
        errors,
    );
    check_percentage(
        province.strategic_value,
        &format!("province {}.strategic_value", province.id),
        errors,
    );
    if let Some(points) = province.victory_points {
        check_percentage(
            points,
            &format!("province {}.victory_points", province.id),
            errors,
        );
    }
    if province.fortification > 10 {
        errors.push(format!("province {} : fortification > 10", province.id));
    }

    for neighbor in &province.neighbors {
        if !province_ids.contains(neighbor) {
            errors.push(format!(
                "province {} : voisin inconnu {}",
                province.id, neighbor
            ));
        }
    }
    if let Some(known) = nation_ids {
        if let Some(owner) = &province.owner {
            if !known.contains(owner) {
                errors.push(format!(
                    "province {} : propriétaire inconnu {}",
                    province.id, owner
                ));
            }
        }
        if let Some(controller) = &province.controller {
            if !known.contains(controller) {
                errors.push(format!(
                    "province {} : contrôleur inconnu {}",
                    province.id, controller
                ));
            }
        }
        for claim in &province.claims {
            if !known.contains(claim) {
                errors.push(format!(
                    "province {} : revendication inconnue {}",
                    province.id, claim
                ));
            }
        }
        if let Some(capital_of) = &province.capital_of {
            if !known.contains(capital_of) {
                errors.push(format!(
                    "province {} : capital_of inconnue {}",
                    province.id, capital_of
                ));
            }
        }
    }
}

fn validate_border(border: &Border, province_ids: &BTreeSet<ProvinceId>, errors: &mut Vec<String>) {
    if border.a == border.b {
        errors.push(format!(
            "frontière {} relie une province à elle-même",
            border.id
        ));
    }
    if !province_ids.contains(&border.a) || !province_ids.contains(&border.b) {
        errors.push(format!(
            "frontière {} référence une province inconnue",
            border.id
        ));
    }
    if border.points.0.len() < 2 {
        errors.push(format!("frontière {} : moins de deux points", border.id));
    }
    if let Some(length) = border.length_blocks {
        if length < 0.0 {
            errors.push(format!("frontière {} : longueur négative", border.id));
        }
    }
    if border.crossing_cost < 0.0 {
        errors.push(format!(
            "frontière {} : coût de traversée négatif",
            border.id
        ));
    }
}

fn validate_nation(
    nation: &Nation,
    world: &World,
    nation_ids: &BTreeSet<NationId>,
    errors: &mut Vec<String>,
) {
    if let Some(version) = nation.schema_version {
        if version == 0 {
            errors.push(format!("nation {} : schema_version invalide", nation.id));
        }
    }
    if !is_color(&nation.color) {
        errors.push(format!(
            "nation {} : couleur invalide {}",
            nation.id, nation.color
        ));
    }
    if nation.flag.palette.is_empty() || nation.flag.palette.len() > 5 {
        errors.push(format!(
            "nation {} : palette de drapeau invalide",
            nation.id
        ));
    }
    for color in &nation.flag.palette {
        if !is_color(color) {
            errors.push(format!(
                "nation {} : couleur de drapeau invalide",
                nation.id
            ));
        }
    }
    if !nation.provinces.contains(&nation.capital) {
        errors.push(format!(
            "nation {} : capitale absente de ses provinces",
            nation.id
        ));
    }
    check_unique(
        nation.provinces.iter(),
        &format!("nation {}.provinces", nation.id),
        errors,
    );
    if nation.provinces.is_empty() {
        errors.push(format!("nation {} : aucune province", nation.id));
    }
    for province in &nation.provinces {
        match world.province(province) {
            None => errors.push(format!(
                "nation {} référence la province inconnue {}",
                nation.id, province
            )),
            Some(value) => {
                if value.owner.as_ref() != Some(&nation.id) {
                    errors.push(format!(
                        "nation {} liste la province {} qui ne lui appartient pas",
                        nation.id, province
                    ));
                }
            }
        }
    }
    if let Some(language) = &nation.language {
        if !valid_language(language) {
            errors.push(format!(
                "nation {} : langue invalide {}",
                nation.id, language
            ));
        }
    }
    if let Some(age) = nation.leader.age {
        if !(18..=120).contains(&age) {
            errors.push(format!("nation {} : âge du dirigeant invalide", nation.id));
        }
    }
    check_optional_date(
        nation.founded,
        &format!("nation {}.founded", nation.id),
        errors,
    );

    if nation.population.workforce > nation.population.total
        || nation.population.conscription_pool > nation.population.total
    {
        errors.push(format!("nation {} : population incohérente", nation.id));
    }
    if !(-0.2..=0.2).contains(&nation.population.growth_rate) {
        errors.push(format!("nation {} : growth_rate invalide", nation.id));
    }
    if let Some(urbanization) = nation.population.urbanization {
        check_unit(
            urbanization,
            &format!("nation {}.population.urbanization", nation.id),
            errors,
        );
    }

    if nation.economy.gdp < 0.0 || nation.economy.treasury < 0.0 || nation.economy.debt < 0.0 {
        errors.push(format!("nation {} : économie négative", nation.id));
    }
    if !(0.0..=0.9).contains(&nation.economy.tax_rate) {
        errors.push(format!("nation {} : tax_rate invalide", nation.id));
    }
    if !(-0.5..=5.0).contains(&nation.economy.inflation) {
        errors.push(format!("nation {} : inflation invalide", nation.id));
    }
    if !(0.0..=1.0).contains(&nation.economy.unemployment) {
        errors.push(format!("nation {} : chômage invalide", nation.id));
    }
    let sectors = &nation.economy.sectors;
    if [
        sectors.agriculture,
        sectors.industry,
        sectors.services,
        sectors.technology,
    ]
    .iter()
    .any(|value| *value < 0.0)
    {
        errors.push(format!("nation {} : secteur économique négatif", nation.id));
    }
    for (resource, value) in &nation.economy.resources {
        if *value < 0.0 {
            errors.push(format!(
                "nation {} : stock négatif pour {}",
                nation.id, resource
            ));
        }
    }
    for sanction in &nation.economy.sanctions {
        if !nation_ids.contains(&sanction.by) {
            errors.push(format!(
                "nation {} : sanction d'une nation inconnue",
                nation.id
            ));
        }
        check_unit(
            sanction.severity,
            &format!("nation {}.economy.sanction.severity", nation.id),
            errors,
        );
        check_optional_date(
            sanction.since,
            &format!("nation {}.economy.sanction.since", nation.id),
            errors,
        );
    }

    check_percentage(
        nation.military.readiness,
        &format!("nation {}.military.readiness", nation.id),
        errors,
    );
    check_percentage(
        nation.military.morale,
        &format!("nation {}.military.morale", nation.id),
        errors,
    );

    check_percentage(
        nation.stability,
        &format!("nation {}.stability", nation.id),
        errors,
    );
    check_percentage(
        nation.legitimacy,
        &format!("nation {}.legitimacy", nation.id),
        errors,
    );
    check_percentage(
        nation.prestige,
        &format!("nation {}.prestige", nation.id),
        errors,
    );
    check_percentage(
        nation.war_exhaustion,
        &format!("nation {}.war_exhaustion", nation.id),
        errors,
    );
    check_percentage(
        nation.unrest,
        &format!("nation {}.unrest", nation.id),
        errors,
    );

    if !(1..=12).contains(&nation.technology.era) {
        errors.push(format!("nation {} : ère technologique invalide", nation.id));
    }
    check_unique(
        nation.technology.researched.iter(),
        &format!("nation {}.technology.researched", nation.id),
        errors,
    );
    if nation.technology.research_points < 0.0 || nation.technology.research_speed < 0.0 {
        errors.push(format!("nation {} : recherche négative", nation.id));
    }

    let profile = &nation.ai_profile;
    for (label, value) in [
        ("aggression", profile.aggression),
        ("caution", profile.caution),
        ("honor", profile.honor),
        ("greed", profile.greed),
        ("revanchism", profile.revanchism),
        ("isolationism", profile.isolationism),
        ("opportunism", profile.opportunism),
        ("loyalty", profile.loyalty),
        ("risk_tolerance", profile.risk_tolerance),
        ("patience", profile.patience),
        ("war_threshold", profile.war_threshold),
        ("peace_threshold", profile.peace_threshold),
    ] {
        check_unit(
            value,
            &format!("nation {}.ai_profile.{label}", nation.id),
            errors,
        );
    }
    for (priority, value) in &profile.priorities {
        check_unit(
            *value,
            &format!("nation {}.ai_profile.priorities.{priority}", nation.id),
            errors,
        );
    }
    check_unique_debug(
        &profile.red_lines,
        &format!("nation {}.ai_profile.red_lines", nation.id),
        errors,
    );

    for claim in &nation.claims {
        if world.province(&claim.province).is_none() {
            errors.push(format!(
                "nation {} revendique la province inconnue {}",
                nation.id, claim.province
            ));
        }
        check_unit(
            claim.priority,
            &format!("nation {}.claim.priority", nation.id),
            errors,
        );
        check_optional_date(
            claim.since,
            &format!("nation {}.claim.since", nation.id),
            errors,
        );
    }
    for subject in &nation.subjects {
        if !nation_ids.contains(&subject.nation) || subject.nation == nation.id {
            errors.push(format!(
                "nation {} : sujet invalide {}",
                nation.id, subject.nation
            ));
        }
        check_unit(
            subject.autonomy,
            &format!("nation {}.subject.autonomy", nation.id),
            errors,
        );
        check_optional_date(
            subject.since,
            &format!("nation {}.subject.since", nation.id),
            errors,
        );
    }
    if let Some(overlord) = &nation.overlord {
        if !nation_ids.contains(overlord) || overlord == &nation.id {
            errors.push(format!(
                "nation {} : suzerain invalide {}",
                nation.id, overlord
            ));
        }
    }
    for ally in &nation.allies {
        if !nation_ids.contains(ally) || ally == &nation.id {
            errors.push(format!("nation {} : allié invalide {}", nation.id, ally));
        }
    }
    check_unique(
        nation.allies.iter(),
        &format!("nation {}.allies", nation.id),
        errors,
    );
}

fn valid_language(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() == 2 {
        return bytes.iter().all(u8::is_ascii_lowercase);
    }
    bytes.len() == 5
        && bytes[0].is_ascii_lowercase()
        && bytes[1].is_ascii_lowercase()
        && bytes[2] == b'_'
        && bytes[3].is_ascii_uppercase()
        && bytes[4].is_ascii_uppercase()
}

fn validate_diplomacy(
    diplomacy: &Diplomacy,
    _world: &World,
    nation_ids: &BTreeSet<NationId>,
    errors: &mut Vec<String>,
) {
    if diplomacy.schema_version == 0 {
        errors.push("diplomatie : schema_version invalide".to_owned());
    }
    let mut pairs = BTreeSet::new();
    for relation in &diplomacy.relations {
        if relation.a >= relation.b {
            errors.push(format!(
                "relation {} / {} : identifiants non triés",
                relation.a, relation.b
            ));
        }
        let key = format!("{}|{}", relation.a, relation.b);
        if !pairs.insert(key) {
            errors.push(format!(
                "relation dupliquée {} / {}",
                relation.a, relation.b
            ));
        }
        if !nation_ids.contains(&relation.a) || !nation_ids.contains(&relation.b) {
            errors.push(format!(
                "relation {} / {} référence une nation inconnue",
                relation.a, relation.b
            ));
        }
        if !(-100.0..=100.0).contains(&relation.opinion) {
            errors.push(format!(
                "relation {} / {} : opinion hors bornes",
                relation.a, relation.b
            ));
        }
        check_percentage(
            relation.trust,
            &format!("relation {}/{}.trust", relation.a, relation.b),
            errors,
        );
        check_percentage(
            relation.fear,
            &format!("relation {}/{}.fear", relation.a, relation.b),
            errors,
        );
        check_unit(
            relation.trade_dependence,
            &format!("relation {}/{}.trade_dependence", relation.a, relation.b),
            errors,
        );
        check_percentage(
            relation.border_tension,
            &format!("relation {}/{}.border_tension", relation.a, relation.b),
            errors,
        );
    }

    check_unique(
        diplomacy.treaties.iter().map(|treaty| &treaty.id),
        "traité",
        errors,
    );
    for treaty in &diplomacy.treaties {
        if treaty.parties.len() < 2 {
            errors.push(format!("traité {} : moins de deux parties", treaty.id));
        }
        check_unique(
            treaty.parties.iter(),
            &format!("traité {}.parties", treaty.id),
            errors,
        );
        for party in treaty
            .parties
            .iter()
            .chain(treaty.signatories.iter())
            .chain(treaty.guarantors.iter())
        {
            if !nation_ids.contains(party) {
                errors.push(format!(
                    "traité {} référence une nation inconnue",
                    treaty.id
                ));
            }
        }
        check_date(
            treaty.start_date,
            &format!("traité {}.start_date", treaty.id),
            errors,
        );
        check_optional_date(
            treaty.end_date,
            &format!("traité {}.end_date", treaty.id),
            errors,
        );
        for violation in &treaty.violations {
            if !nation_ids.contains(&violation.by) {
                errors.push(format!(
                    "traité {} : violation par une nation inconnue",
                    treaty.id
                ));
            }
        }
    }

    check_unique(
        diplomacy
            .organizations
            .iter()
            .map(|organization| &organization.id),
        "organisation",
        errors,
    );
    for organization in &diplomacy.organizations {
        if organization.members.len() < 2 {
            errors.push(format!(
                "organisation {} : moins de deux membres",
                organization.id
            ));
        }
        check_unique(
            organization.members.iter(),
            &format!("organisation {}.members", organization.id),
            errors,
        );
        for member in &organization.members {
            if !nation_ids.contains(member) {
                errors.push(format!(
                    "organisation {} : membre inconnu {}",
                    organization.id, member
                ));
            }
        }
        if let Some(leader) = &organization.leader {
            if !nation_ids.contains(leader) {
                errors.push(format!("organisation {} : leader inconnu", organization.id));
            }
        }
        check_percentage(
            organization.cohesion,
            &format!("organisation {}.cohesion", organization.id),
            errors,
        );
        check_date(
            organization.founding_date,
            &format!("organisation {}.founding_date", organization.id),
            errors,
        );
        for vote in &organization.votes {
            if !nation_ids.contains(&vote.nation) {
                errors.push(format!(
                    "organisation {} : vote d'une nation inconnue",
                    organization.id
                ));
            }
            if vote.weight < 0.0 {
                errors.push(format!(
                    "organisation {} : poids de vote négatif",
                    organization.id
                ));
            }
        }
    }

    check_unique(
        diplomacy.sanctions.iter().map(|sanction| &sanction.id),
        "sanction",
        errors,
    );
    for sanction in &diplomacy.sanctions {
        if sanction.by.is_empty() {
            errors.push(format!(
                "sanction {} : aucune nation émettrice",
                sanction.id
            ));
        }
        check_unique(
            sanction.by.iter(),
            &format!("sanction {}.by", sanction.id),
            errors,
        );
        for nation in sanction.by.iter().chain(std::iter::once(&sanction.target)) {
            if !nation_ids.contains(nation) {
                errors.push(format!(
                    "sanction {} référence une nation inconnue",
                    sanction.id
                ));
            }
        }
        check_unit(
            sanction.severity,
            &format!("sanction {}.severity", sanction.id),
            errors,
        );
        check_date(
            sanction.start_date,
            &format!("sanction {}.start_date", sanction.id),
            errors,
        );
        check_optional_date(
            sanction.end_date,
            &format!("sanction {}.end_date", sanction.id),
            errors,
        );
    }

    check_unique(
        diplomacy.guarantees.iter().map(|guarantee| &guarantee.id),
        "garantie",
        errors,
    );
    for guarantee in &diplomacy.guarantees {
        if guarantee.guarantor == guarantee.protected {
            errors.push(format!("garantie {} : auto-garantie", guarantee.id));
        }
        if !nation_ids.contains(&guarantee.guarantor) || !nation_ids.contains(&guarantee.protected)
        {
            errors.push(format!(
                "garantie {} référence une nation inconnue",
                guarantee.id
            ));
        }
        check_date(
            guarantee.start_date,
            &format!("garantie {}.start_date", guarantee.id),
            errors,
        );
        check_optional_date(
            guarantee.expires_date,
            &format!("garantie {}.expires_date", guarantee.id),
            errors,
        );
    }

    check_unique(
        diplomacy
            .espionage_operations
            .iter()
            .map(|operation| &operation.id),
        "opération d'espionnage",
        errors,
    );
    for operation in &diplomacy.espionage_operations {
        if !nation_ids.contains(&operation.actor) || !nation_ids.contains(&operation.target) {
            errors.push(format!(
                "opération {} référence une nation inconnue",
                operation.id
            ));
        }
        check_percentage(
            operation.progress,
            &format!("opération {}.progress", operation.id),
            errors,
        );
        check_percentage(
            operation.exposure,
            &format!("opération {}.exposure", operation.id),
            errors,
        );
    }

    check_unique(
        diplomacy.trade_routes.iter().map(|route| &route.id),
        "route commerciale",
        errors,
    );
    for route in &diplomacy.trade_routes {
        if !nation_ids.contains(&route.a) || !nation_ids.contains(&route.b) {
            errors.push(format!("route {} référence une nation inconnue", route.id));
        }
        if route.volume < 0.0 {
            errors.push(format!("route {} : volume négatif", route.id));
        }
        if let Some(blocker) = &route.blockaded_by {
            if !nation_ids.contains(blocker) {
                errors.push(format!("route {} : bloqueur inconnu", route.id));
            }
        }
    }
}

fn validate_wars(
    wars: &[War],
    world: &World,
    nation_ids: &BTreeSet<NationId>,
    diplomacy: &Diplomacy,
    errors: &mut Vec<String>,
) {
    check_unique(wars.iter().map(|war| &war.id), "guerre", errors);
    let treaty_ids: BTreeSet<TreatyId> = diplomacy
        .treaties
        .iter()
        .map(|treaty| treaty.id.clone())
        .collect();
    for war in wars {
        check_date(
            war.start_date,
            &format!("guerre {}.start_date", war.id),
            errors,
        );
        check_optional_date(war.end_date, &format!("guerre {}.end_date", war.id), errors);
        if war.belligerents.len() < 2 {
            errors.push(format!("guerre {} : moins de deux belligérants", war.id));
        }
        for belligerent in &war.belligerents {
            if !nation_ids.contains(&belligerent.nation) {
                errors.push(format!("guerre {} : belligérant inconnu", war.id));
            }
            check_percentage(
                belligerent.war_support,
                &format!("guerre {}.war_support", war.id),
                errors,
            );
            for province in &belligerent.occupied_provinces {
                if world.province(province).is_none() {
                    errors.push(format!("guerre {} : province occupée inconnue", war.id));
                }
            }
        }
        check_unique(
            war.war_goals.iter().map(|goal| &goal.id),
            &format!("guerre {}.war_goals", war.id),
            errors,
        );
        for goal in &war.war_goals {
            if !nation_ids.contains(&goal.target) {
                errors.push(format!("guerre {} : cible d'objectif inconnue", war.id));
            }
            for province in &goal.provinces {
                if world.province(province).is_none() {
                    errors.push(format!("guerre {} : province d'objectif inconnue", war.id));
                }
            }
            check_unit(
                goal.priority,
                &format!("guerre {}.goal.priority", war.id),
                errors,
            );
        }
        check_unique(
            war.fronts.iter().map(|front| &front.id),
            &format!("guerre {}.fronts", war.id),
            errors,
        );
        for front in &war.fronts {
            if front.provinces.is_empty() {
                errors.push(format!("front {} : aucune province", front.id));
            }
            for province in &front.provinces {
                if world.province(province).is_none() {
                    errors.push(format!("front {} : province inconnue", front.id));
                }
            }
            if !nation_ids.contains(&front.attacker) || !nation_ids.contains(&front.defender) {
                errors.push(format!("front {} : nation inconnue", front.id));
            }
            check_percentage(front.supply, &format!("front {}.supply", front.id), errors);
            if !(-100.0..=100.0).contains(&front.momentum) {
                errors.push(format!("front {} : momentum hors bornes", front.id));
            }
        }
        check_unique(
            war.battles.iter().map(|battle| &battle.id),
            &format!("guerre {}.battles", war.id),
            errors,
        );
        let front_ids: BTreeSet<String> = war
            .fronts
            .iter()
            .map(|front| front.id.to_string())
            .collect();
        for battle in &war.battles {
            if let Some(front) = &battle.front {
                if !front_ids.contains(&front.to_string()) {
                    errors.push(format!("bataille {} : front inconnu", battle.id));
                }
            }
            if world.province(&battle.province).is_none() {
                errors.push(format!("bataille {} : province inconnue", battle.id));
            }
            if !nation_ids.contains(&battle.attacker) || !nation_ids.contains(&battle.defender) {
                errors.push(format!("bataille {} : nation inconnue", battle.id));
            }
            check_date(
                battle.start_date,
                &format!("bataille {}.start_date", battle.id),
                errors,
            );
            check_optional_date(
                battle.end_date,
                &format!("bataille {}.end_date", battle.id),
                errors,
            );
        }
        for army in &war.armies {
            validate_army(
                army,
                world,
                nation_ids,
                &war.battles,
                errors,
                war.id.as_ref(),
            );
        }
        check_unique(
            war.peace_offers.iter().map(|offer| &offer.id),
            &format!("guerre {}.peace_offers", war.id),
            errors,
        );
        for offer in &war.peace_offers {
            if !nation_ids.contains(&offer.by) || !nation_ids.contains(&offer.to) {
                errors.push(format!("offre {} : nation inconnue", offer.id));
            }
            check_date(offer.date, &format!("offre {}.date", offer.id), errors);
            if let Some(acceptance) = offer.ai_acceptance {
                check_unit(
                    acceptance,
                    &format!("offre {}.ai_acceptance", offer.id),
                    errors,
                );
            }
        }
        for intervention in &war.interventions {
            if !nation_ids.contains(&intervention.nation) {
                errors.push(format!("guerre {} : intervention inconnue", war.id));
            }
            check_date(
                intervention.date,
                &format!("guerre {}.intervention.date", war.id),
                errors,
            );
        }
        if let Some(treaty) = &war.treaty_id {
            if !treaty_ids.contains(treaty) {
                errors.push(format!("guerre {} : traité inconnu {}", war.id, treaty));
            }
        }
    }
}

fn validate_armies(
    armies: &[Army],
    world: &World,
    nation_ids: &BTreeSet<NationId>,
    wars: &[War],
    errors: &mut Vec<String>,
) {
    check_unique(armies.iter().map(|army| &army.id), "armée", errors);
    let all_battles: Vec<crate::war::Battle> = wars
        .iter()
        .flat_map(|war| war.battles.iter().cloned())
        .collect();
    for army in armies {
        validate_army(army, world, nation_ids, &all_battles, errors, "scénario");
    }
}

fn validate_army(
    army: &Army,
    world: &World,
    nation_ids: &BTreeSet<NationId>,
    battles: &[crate::war::Battle],
    errors: &mut Vec<String>,
    context: &str,
) {
    if !nation_ids.contains(&army.nation) {
        errors.push(format!("armée {} : nation inconnue", army.id));
    }
    if world.province(&army.location).is_none() {
        errors.push(format!(
            "armée {} : province de localisation inconnue",
            army.id
        ));
    }
    if army.units.is_empty() {
        errors.push(format!("armée {} : aucune unité", army.id));
    }
    for unit in &army.units {
        if unit.count == 0 {
            errors.push(format!("armée {} : unité à zéro", army.id));
        }
        check_percentage(
            unit.experience,
            &format!("armée {}.expérience", army.id),
            errors,
        );
        check_percentage(unit.morale, &format!("armée {}.moral", army.id), errors);
        if let Some(equipment) = unit.equipment {
            check_percentage(equipment, &format!("armée {}.équipement", army.id), errors);
        }
    }
    check_percentage(army.supply, &format!("armée {}.supply", army.id), errors);
    check_percentage(army.morale, &format!("armée {}.morale", army.id), errors);
    if let Some(movement) = &army.movement {
        if world.province(&movement.from).is_none() || world.province(&movement.to).is_none() {
            errors.push(format!(
                "armée {} : mouvement vers une province inconnue",
                army.id
            ));
        }
        check_unit(
            movement.progress,
            &format!("armée {}.movement.progress", army.id),
            errors,
        );
    }
    if let Some(general) = &army.general {
        check_percentage(
            general.skill,
            &format!("armée {}.general.skill", army.id),
            errors,
        );
    }
    if let Some(engaged) = &army.engaged_in {
        if !battles.iter().any(|battle| &battle.id == engaged) {
            errors.push(format!(
                "armée {} : bataille inconnue {} (guerre {})",
                army.id, engaged, context
            ));
        }
    }
}

fn validate_revolutions(
    revolutions: &[Revolution],
    world: &World,
    nation_ids: &BTreeSet<NationId>,
    errors: &mut Vec<String>,
) {
    check_unique(
        revolutions.iter().map(|revolution| &revolution.id),
        "révolution",
        errors,
    );
    for revolution in revolutions {
        if !nation_ids.contains(&revolution.nation) {
            errors.push(format!("révolution {} : nation inconnue", revolution.id));
        }
        check_date(
            revolution.start_date,
            &format!("révolution {}.start_date", revolution.id),
            errors,
        );
        check_optional_date(
            revolution.end_date,
            &format!("révolution {}.end_date", revolution.id),
            errors,
        );
        check_unique_debug(
            &revolution.causes,
            &format!("révolution {}.causes", revolution.id),
            errors,
        );
        if revolution.factions.is_empty() {
            errors.push(format!("révolution {} : aucune faction", revolution.id));
        }
        check_unique(
            revolution.factions.iter().map(|faction| &faction.id),
            &format!("révolution {}.factions", revolution.id),
            errors,
        );
        for faction in &revolution.factions {
            check_percentage(
                faction.support,
                &format!("faction {}.support", faction.id),
                errors,
            );
            check_percentage(
                faction.legitimacy,
                &format!("faction {}.legitimacy", faction.id),
                errors,
            );
            for support in &faction.foreign_support {
                if !nation_ids.contains(&support.nation) {
                    errors.push(format!("faction {} : soutien étranger inconnu", faction.id));
                }
                if support.amount < 0.0 {
                    errors.push(format!("faction {} : soutien négatif", faction.id));
                }
            }
        }
        for support in &revolution.support_by_province {
            if world.province(&support.province).is_none() {
                errors.push(format!(
                    "révolution {} : province de soutien inconnue",
                    revolution.id
                ));
            }
            check_percentage(
                support.support,
                &format!("révolution {}.province.support", revolution.id),
                errors,
            );
        }
        for province in &revolution.controlled_provinces {
            if world.province(province).is_none() {
                errors.push(format!(
                    "révolution {} : province contrôlée inconnue",
                    revolution.id
                ));
            }
        }
        check_unit(
            revolution.government_response.intensity,
            &format!("révolution {}.government_response.intensity", revolution.id),
            errors,
        );
        check_optional_date(
            revolution.government_response.started_date,
            &format!(
                "révolution {}.government_response.started_date",
                revolution.id
            ),
            errors,
        );
        for intervention in &revolution.foreign_interventions {
            if !nation_ids.contains(&intervention.nation) {
                errors.push(format!(
                    "révolution {} : intervention inconnue",
                    revolution.id
                ));
            }
            if intervention.amount < 0.0 {
                errors.push(format!(
                    "révolution {} : intervention négative",
                    revolution.id
                ));
            }
        }
    }
}

fn validate_missiles(
    missiles: &[MissileInstance],
    world: &World,
    nation_ids: &BTreeSet<NationId>,
    errors: &mut Vec<String>,
) {
    check_unique(
        missiles.iter().map(|missile| &missile.id),
        "missile",
        errors,
    );
    for missile in missiles {
        if !nation_ids.contains(&missile.owner) {
            errors.push(format!("missile {} : propriétaire inconnu", missile.id));
        }
        if world.province(&missile.launcher_province).is_none() {
            errors.push(format!(
                "missile {} : province de lancement inconnue",
                missile.id
            ));
        }
        if world.province(&missile.target.province).is_none() {
            errors.push(format!("missile {} : province cible inconnue", missile.id));
        }
        if missile.arrival_tick < missile.launch_tick {
            errors.push(format!(
                "missile {} : arrivée avant le lancement",
                missile.id
            ));
        }
        best_effort_trajectory(missile, errors);
        for interceptor in &missile.interceptors {
            if !nation_ids.contains(&interceptor.by) {
                errors.push(format!("missile {} : intercepteur inconnu", missile.id));
            }
            check_unit(
                interceptor.chance,
                &format!("missile {}.interceptor.chance", missile.id),
                errors,
            );
        }
        for nation in &missile.detected_by {
            if !nation_ids.contains(nation) {
                errors.push(format!(
                    "missile {} : détection par une nation inconnue",
                    missile.id
                ));
            }
        }
        if let Some(report) = &missile.damage_report {
            check_percentage(
                report.infrastructure_damage,
                &format!("missile {}.damage.infrastructure", missile.id),
                errors,
            );
            check_percentage(
                report.escalation,
                &format!("missile {}.damage.escalation", missile.id),
                errors,
            );
            for province in &report.provinces_affected {
                if world.province(province).is_none() {
                    errors.push(format!(
                        "missile {} : province affectée inconnue",
                        missile.id
                    ));
                }
            }
        }
    }
}

fn best_effort_trajectory(missile: &MissileInstance, errors: &mut Vec<String>) {
    if let Some(trajectory) = &missile.trajectory {
        if trajectory.points.len() == 1 {
            errors.push(format!(
                "missile {} : trajectoire à un seul point",
                missile.id
            ));
        }
        if trajectory.apogee_km.is_some_and(|value| value < 0.0)
            || trajectory
                .terminal_speed_kmh
                .is_some_and(|value| value < 0.0)
        {
            errors.push(format!("missile {} : trajectoire négative", missile.id));
        }
    }
}

/// Vérifie qu'un identifiant de bataille existe dans une guerre.
pub fn find_battle<'a>(war: &'a War, battle: &BattleId) -> Option<&'a crate::war::Battle> {
    war.battles.iter().find(|value| &value.id == battle)
}
