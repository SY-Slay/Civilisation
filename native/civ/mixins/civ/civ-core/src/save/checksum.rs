//! Checksum SHA-256 du contenu canonique d'une sauvegarde.

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::error::CoreResult;

/// Encode un résumé SHA-256 en hexadécimal minuscule.
pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

/// Sérialise une valeur en JSON canonique compact.
pub fn canonical_json(value: &Value) -> CoreResult<String> {
    Ok(serde_json::to_string(value)?)
}

/// Calcule le checksum d'une valeur en excluant le champ `checksum`.
pub fn checksum_of<T: Serialize>(value: &T) -> CoreResult<String> {
    let mut value = serde_json::to_value(value)?;
    if let Value::Object(map) = &mut value {
        map.remove("checksum");
    }
    Ok(sha256_hex(canonical_json(&value)?.as_bytes()))
}

#[cfg(all(test, not(feature = "embedded-core")))]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn le_checksum_ignore_l_ordre_d_insertion() {
        let first = json!({"a": 1, "b": [2, 3], "checksum": "ancien"});
        let second = json!({"checksum": "ancien", "b": [2, 3], "a": 1});
        assert_eq!(checksum_of(&first).unwrap(), checksum_of(&second).unwrap());
    }

    #[test]
    fn le_checksum_change_avec_le_contenu() {
        let first = json!({"a": 1});
        let second = json!({"a": 2});
        assert_ne!(checksum_of(&first).unwrap(), checksum_of(&second).unwrap());
    }

    #[test]
    fn longueur_du_checksum() {
        assert_eq!(sha256_hex(b"civ").len(), 64);
    }
}
