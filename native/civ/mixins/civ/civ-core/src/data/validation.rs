//! Validation croisée des registres et de la cohérence globale d'un scénario.

use super::registry::DataPackRegistries;
use super::GameData;
use crate::error::{CoreError, CoreResult};
use crate::scenario::Scenario;

/// Valide la cohérence référentielle d'un scénario vis-à-vis des registres et définitions.
pub fn validate_scenario_against_registries(
    scenario: &Scenario,
    registries: &DataPackRegistries,
    data: &GameData,
) -> CoreResult<()> {
    // 1. Validation des nations
    for nation in &scenario.nations {
        if !registries.governments.contains_key(&nation.government) {
            return Err(CoreError::data(format!(
                "Nation {} : gouvernement inconnu '{}'",
                nation.id, nation.government
            )));
        }
        if !registries.ideologies.contains_key(&nation.ideology) {
            return Err(CoreError::data(format!(
                "Nation {} : idéologie inconnue '{}'",
                nation.id, nation.ideology
            )));
        }
        if !registries.cultures.contains_key(&nation.culture) {
            return Err(CoreError::data(format!(
                "Nation {} : culture inconnue '{}'",
                nation.id, nation.culture
            )));
        }
        if !registries.religions.contains_key(&nation.religion) {
            return Err(CoreError::data(format!(
                "Nation {} : religion inconnue '{}'",
                nation.id, nation.religion
            )));
        }

        // Vérification des stocks de missiles
        for missile_id in nation.military.missile_stockpile.keys() {
            if data.missile(missile_id).is_none() {
                return Err(CoreError::data(format!(
                    "Nation {} : type de missile en stock inconnu '{}'",
                    nation.id, missile_id
                )));
            }
        }

        // Vérification des technologies recherchées
        for tech_id in &nation.technology.researched {
            if data.technology(tech_id).is_none() {
                return Err(CoreError::data(format!(
                    "Nation {} : technologie recherchée inconnue '{}'",
                    nation.id, tech_id
                )));
            }
        }
    }

    // 2. Validation des provinces
    for prov in &scenario.world.provinces {
        if !registries.cultures.contains_key(&prov.culture.primary) {
            return Err(CoreError::data(format!(
                "Province {} : culture primaire inconnue '{}'",
                prov.id, prov.culture.primary
            )));
        }
        if !registries.religions.contains_key(&prov.religion.primary) {
            return Err(CoreError::data(format!(
                "Province {} : religion primaire inconnue '{}'",
                prov.id, prov.religion.primary
            )));
        }
        for deposit in &prov.resources {
            if !registries.resources.contains_key(&deposit.resource) {
                return Err(CoreError::data(format!(
                    "Province {} : ressource gisement inconnue '{}'",
                    prov.id, deposit.resource
                )));
            }
        }
    }

    Ok(())
}
