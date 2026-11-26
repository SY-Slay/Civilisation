//! Sauvegarde complète et reproductible.

pub mod checksum;
pub mod migrate;

use serde::{Deserialize, Serialize};

use crate::diplomacy::Diplomacy;
use crate::error::{CoreError, CoreResult};
use crate::event::EventState;
use crate::ids::{NationId, ScenarioId};
use crate::missile::MissileInstance;
use crate::nation::Nation;
use crate::revolution::Revolution;
use crate::rng::RngState;
use crate::scenario::{Difficulty, Scenario};
use crate::text::LocalizedText;
use crate::time::{GameDate, Tick};
use crate::war::{Army, War};
use crate::world::World;

/// Version du jeu Horizon attendue.
pub const GAME_VERSION: &str = "26.3";
/// Révision Horizon épinglée par le projet.
pub const HORIZON_REVISION: &str = "9f42064c11efe7ef3e5ed2baad7ddc2a3b36eea0";
/// Version du mod.
pub const MOD_VERSION: &str = "0.1.0";

/// Informations de compatibilité d'une sauvegarde.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Compatibility {
    pub game_version: String,
    pub mod_version: String,
    pub horizon_revision: String,
    pub data_fingerprint: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<u32>,
}

/// Métadonnées de confort.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub save_name: String,
    #[serde(default)]
    pub player_nation: Option<NationId>,
    pub playtime_seconds: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default)]
    pub multiplayer: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeeklyRankingEntry {
    pub nation: NationId,
    pub rank: u32,
    pub previous_rank: Option<u32>,
    pub score: f64,
    pub treasury: f64,
    pub gdp: f64,
    pub population: u64,
    pub week: u32,
}

/// Instantané complet d'une partie.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Save {
    pub schema_version: u32,
    pub save_id: String,
    pub created_at: String,
    pub updated_at: String,
    pub compatibility: Compatibility,
    pub scenario_id: ScenarioId,
    pub seed: u64,
    pub tick: Tick,
    pub date: GameDate,
    /// Date initiale du scénario, conservée pour les délais et l'IA.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_date: Option<GameDate>,
    /// Difficulté initiale, conservée pour que l'IA reste identique après rechargement.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<Difficulty>,
    /// Nombre de ticks par jour fictif, conservé pour rejouer la même horloge.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticks_per_day: Option<u32>,
    pub world: World,
    pub nations: Vec<Nation>,
    pub diplomacy: Diplomacy,
    pub wars: Vec<War>,
    pub revolutions: Vec<Revolution>,
    pub armies: Vec<Army>,
    pub missiles_in_flight: Vec<MissileInstance>,
    pub event_state: EventState,
    pub rng: RngState,
    pub checksum: String,
    pub metadata: SaveMetadata,
    #[serde(default)]
    pub weekly_ranking: Vec<WeeklyRankingEntry>,
    #[serde(default)]
    pub last_ranking_week: u32,
}

impl Save {
    /// Crée une sauvegarde à partir d'un scénario, en conservant l'état initial.
    pub fn from_scenario(
        scenario: &Scenario,
        save_id: impl Into<String>,
        save_name: impl Into<String>,
        created_at: impl Into<String>,
        updated_at: impl Into<String>,
    ) -> CoreResult<Self> {
        scenario.validate()?;
        let data_fingerprint = checksum::checksum_of(scenario)?;
        let save = Self {
            schema_version: migrate::CURRENT_SAVE_VERSION,
            save_id: save_id.into(),
            created_at: created_at.into(),
            updated_at: updated_at.into(),
            compatibility: Compatibility {
                game_version: GAME_VERSION.to_owned(),
                mod_version: MOD_VERSION.to_owned(),
                horizon_revision: HORIZON_REVISION.to_owned(),
                data_fingerprint,
                schema_version: Some(migrate::CURRENT_SAVE_VERSION),
            },
            scenario_id: scenario.id.clone(),
            seed: scenario.seed,
            tick: Tick::ZERO,
            date: scenario.start_date,
            start_date: Some(scenario.start_date),
            difficulty: Some(scenario.difficulty),
            ticks_per_day: None,
            world: scenario.world.clone(),
            nations: scenario.nations.clone(),
            diplomacy: scenario.diplomacy.clone(),
            wars: scenario.wars.clone(),
            revolutions: scenario.revolutions.clone(),
            armies: scenario.armies.clone(),
            missiles_in_flight: scenario.missiles_in_flight.clone(),
            event_state: EventState::default(),
            rng: RngState::new(scenario.seed, 0, 0),
            checksum: String::new(),
            metadata: SaveMetadata {
                save_name: save_name.into(),
                player_nation: scenario.player_nation.clone(),
                playtime_seconds: 0,
                notes: None,
                multiplayer: false,
                host: None,
            },
            weekly_ranking: Vec::new(),
            last_ranking_week: 0,
        };
        save.validate_identity()?;
        Ok(save)
    }

    /// Vérifie les champs d'identité qui ne dépendent pas du sceau.
    pub fn validate_identity(&self) -> CoreResult<()> {
        if !is_uuid_v4(&self.save_id) {
            return Err(CoreError::InvalidData(format!(
                "save_id doit être un UUID v4 : {}",
                self.save_id
            )));
        }
        if self.metadata.save_name.trim().is_empty() || self.metadata.save_name.len() > 120 {
            return Err(CoreError::InvalidData(
                "save_name doit contenir entre 1 et 120 caractères".to_owned(),
            ));
        }
        if self
            .metadata
            .notes
            .as_ref()
            .is_some_and(|notes| notes.len() > 2000)
        {
            return Err(CoreError::InvalidData(
                "notes dépasse 2000 caractères".to_owned(),
            ));
        }
        if self.created_at.trim().is_empty() || self.updated_at.trim().is_empty() {
            return Err(CoreError::InvalidData(
                "created_at et updated_at sont requis".to_owned(),
            ));
        }
        if self.ticks_per_day == Some(0) {
            return Err(CoreError::InvalidData(
                "ticks_per_day doit être strictement positif".to_owned(),
            ));
        }
        if !is_lower_hex(&self.compatibility.data_fingerprint, 64) {
            return Err(CoreError::InvalidData(
                "data_fingerprint doit être un SHA-256 hexadécimal".to_owned(),
            ));
        }
        if !is_lower_hex(&self.compatibility.horizon_revision, 40) {
            return Err(CoreError::InvalidData(
                "horizon_revision doit être un commit de 40 caractères hexadécimaux".to_owned(),
            ));
        }
        Ok(())
    }

    /// Calcule le checksum du contenu, champ `checksum` exclu.
    pub fn computed_checksum(&self) -> CoreResult<String> {
        checksum::checksum_of(self)
    }

    /// Met à jour le checksum après une modification.
    pub fn seal(&mut self) -> CoreResult<()> {
        self.checksum = self.computed_checksum()?;
        Ok(())
    }

    /// Vérifie le checksum enregistré.
    pub fn verify(&self) -> CoreResult<()> {
        let actual = self.computed_checksum()?;
        if self.checksum == actual {
            Ok(())
        } else {
            Err(CoreError::ChecksumMismatch {
                expected: self.checksum.clone(),
                actual,
            })
        }
    }

    /// Valide les données de jeu contenues dans la sauvegarde.
    ///
    /// Le scénario est reconstruit uniquement pour réutiliser les invariants
    /// métier de `civ-core` (voisinage, propriété, références, etc.).
    pub fn validate_payload(&self) -> CoreResult<()> {
        let scenario = Scenario {
            schema_version: self.schema_version,
            id: self.scenario_id.clone(),
            name: LocalizedText::plain("Sauvegarde"),
            description: None,
            start_date: self.start_date.unwrap_or(self.date),
            seed: self.seed,
            world: self.world.clone(),
            nations: self.nations.clone(),
            diplomacy: self.diplomacy.clone(),
            wars: self.wars.clone(),
            revolutions: self.revolutions.clone(),
            armies: self.armies.clone(),
            missiles_in_flight: self.missiles_in_flight.clone(),
            player_nation: self.metadata.player_nation.clone(),
            difficulty: self.difficulty.unwrap_or(Difficulty::Normal),
            tags: Vec::new(),
        };
        crate::validation::validate_scenario(&scenario)
    }

    /// Vérifie les versions de compatibilité.
    pub fn check_compatibility(&self) -> CoreResult<()> {
        if self.schema_version != migrate::CURRENT_SAVE_VERSION {
            return Err(CoreError::UnsupportedSaveVersion(self.schema_version));
        }
        if self.compatibility.game_version != GAME_VERSION {
            return Err(CoreError::IncompatibleSave(format!(
                "Horizon {} au lieu de {}",
                self.compatibility.game_version, GAME_VERSION
            )));
        }
        Ok(())
    }

    /// Sérialise la sauvegarde en JSON lisible.
    pub fn to_json_pretty(&self) -> CoreResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Charge et vérifie une sauvegarde depuis une chaîne JSON.
    pub fn from_json_str(input: &str) -> CoreResult<Self> {
        let value: serde_json::Value = serde_json::from_str(input)?;
        // Vérifie le sceau sur le document d'origine avant toute migration.
        let original: Self = serde_json::from_value(value.clone())?;
        original.check_compatibility()?;
        original.validate_identity()?;
        original.verify()?;
        let migrated = migrate::migrate_value(value)?;
        let mut save: Self = serde_json::from_value(migrated)?;
        save.check_compatibility()?;
        save.validate_identity()?;
        save.validate_payload()?;
        // Une migration peut renommer un champ ; le sceau est recalculé pour
        // que l'objet retourné reste auto-cohérent.
        save.seal()?;
        Ok(save)
    }

    /// Charge une sauvegarde depuis un fichier.
    pub fn from_path(path: &std::path::Path) -> CoreResult<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::from_json_str(&content)
    }

    /// Écrit la sauvegarde avec un fichier temporaire et une copie `.bak`.
    pub fn write_to_path(&self, path: &std::path::Path) -> CoreResult<()> {
        self.verify()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let temp = path.with_extension("civ.tmp");
        let backup = path.with_extension("json.bak");
        let json = self.to_json_pretty()?;
        {
            use std::io::Write;
            let mut file = std::fs::File::create(&temp)?;
            file.write_all(json.as_bytes())?;
            file.sync_all()?;
        }
        if path.exists() {
            if backup.exists() {
                std::fs::remove_file(&backup)?;
            }
            std::fs::rename(path, &backup)?;
        }
        if let Err(error) = std::fs::rename(&temp, path) {
            if backup.exists() {
                let _ = std::fs::rename(&backup, path);
            }
            return Err(error.into());
        }
        Ok(())
    }
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn is_uuid_v4(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 36 {
        return false;
    }
    for (index, byte) in bytes.iter().enumerate() {
        match index {
            8 | 13 | 18 | 23 => {
                if *byte != b'-' {
                    return false;
                }
            }
            14 => {
                if *byte != b'4' {
                    return false;
                }
            }
            19 => {
                if !matches!(*byte, b'8' | b'9' | b'a' | b'b') {
                    return false;
                }
            }
            _ => {
                if !byte.is_ascii_hexdigit() || byte.is_ascii_uppercase() {
                    return false;
                }
            }
        }
    }
    true
}

#[cfg(all(test, not(feature = "embedded-core")))]
mod tests {
    use super::*;

    fn fixture_save() -> Save {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("scenario.json");
        let scenario = Scenario::from_path(&path).expect("scénario de test");
        let now = "2026-01-01T00:00:00Z".to_owned();
        Save::from_scenario(
            &scenario,
            "00000000-0000-4000-8000-000000000001",
            "Test",
            now.clone(),
            now,
        )
        .unwrap()
    }

    #[test]
    fn sceau_et_verification() {
        let mut save = fixture_save();
        save.seal().unwrap();
        save.verify().unwrap();
        assert_eq!(save.checksum.len(), 64);
        save.tick = Tick::new(1);
        assert!(save.verify().is_err());
    }

    #[test]
    fn refuse_un_save_id_non_uuid() {
        let mut save = fixture_save();
        save.save_id = "save_test".to_owned();
        assert!(save.validate_identity().is_err());
    }

    #[test]
    fn empreinte_de_donnees_est_un_sha256() {
        let save = fixture_save();
        assert_eq!(save.compatibility.data_fingerprint.len(), 64);
        assert_ne!(save.compatibility.data_fingerprint, "0".repeat(64));
    }

    #[test]
    fn valide_les_donnees_de_jeu() {
        let save = fixture_save();
        save.validate_payload().unwrap();
        let mut invalid = save;
        invalid.nations.clear();
        assert!(invalid.validate_payload().is_err());
    }

    #[test]
    fn round_trip_json() {
        let mut save = fixture_save();
        save.seal().unwrap();
        let json = save.to_json_pretty().unwrap();
        let loaded = Save::from_json_str(&json).unwrap();
        assert_eq!(loaded.checksum, save.checksum);
        assert_eq!(loaded.schema_version, 1);
    }

    #[test]
    fn migration_du_nom_rng() {
        let mut save = fixture_save();
        save.seal().unwrap();
        let mut value = serde_json::to_value(&save).unwrap();
        value["rng"]["algorithm"] = serde_json::Value::String("splitmix64_pcg_v1".to_owned());
        let checksum = checksum::checksum_of(&value).unwrap();
        value["checksum"] = serde_json::Value::String(checksum);
        let json = serde_json::to_string(&value).unwrap();
        let loaded = Save::from_json_str(&json).unwrap();
        assert_eq!(loaded.rng.algorithm, crate::rng::RNG_ALGORITHM);
        loaded.verify().unwrap();
    }
}
