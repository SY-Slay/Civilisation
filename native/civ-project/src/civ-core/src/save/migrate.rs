//! Migrations explicites des sauvegardes.
//!
//! La version 1 est la première version publique. La migration gère aussi le
//! renommage de l'algorithme RNG utilisé par les brouillons de la version 1.

use serde_json::Value;

use crate::error::{CoreError, CoreResult};
use crate::rng::RNG_ALGORITHM;

/// Version courante du format de sauvegarde.
pub const CURRENT_SAVE_VERSION: u32 = 1;

/// Applique les migrations connues à une valeur JSON.
pub fn migrate_value(value: Value) -> CoreResult<Value> {
    let version = value
        .get("schema_version")
        .and_then(Value::as_u64)
        .unwrap_or(0) as u32;
    match version {
        0 => Err(CoreError::UnsupportedSaveVersion(0)),
        CURRENT_SAVE_VERSION => migrate_v1(value),
        other => Err(CoreError::UnsupportedSaveVersion(other)),
    }
}

fn migrate_v1(mut value: Value) -> CoreResult<Value> {
    // Les brouillons de la v1 utilisaient le nom `splitmix64_pcg_v1`. Le
    // générateur à compteur est compatible avec les champs seed/counter/stream.
    if let Some(rng) = value.get_mut("rng").and_then(Value::as_object_mut) {
        if let Some(algorithm) = rng.get_mut("algorithm") {
            if algorithm == "splitmix64_pcg_v1" {
                *algorithm = Value::String(RNG_ALGORITHM.to_owned());
            }
        }
    }
    Ok(value)
}

#[cfg(all(test, not(feature = "embedded-core")))]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn migre_le_nom_du_rng() {
        let value = json!({
            "schema_version": 1,
            "rng": {"algorithm": "splitmix64_pcg_v1", "seed": 1, "counter": 0, "stream": 0}
        });
        let migrated = migrate_value(value).unwrap();
        assert_eq!(migrated["rng"]["algorithm"].as_str(), Some(RNG_ALGORITHM));
    }

    #[test]
    fn refuse_une_version_inconnue() {
        let value = json!({"schema_version": 99});
        assert!(migrate_value(value).is_err());
    }
}
