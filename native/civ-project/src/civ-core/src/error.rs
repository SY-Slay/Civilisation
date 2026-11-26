//! Erreurs typées du cœur de simulation.
//!
//! Le cœur ne dépend ni de Minecraft ni d'Horizon. Toutes les erreurs de
//! chargement, de validation et de sauvegarde passent par ce type.

use thiserror::Error;

use crate::ids::IdError;

/// Résultat standard du cœur.
pub type CoreResult<T> = Result<T, CoreError>;

/// Erreur du cœur de simulation.
#[derive(Debug, Error)]
pub enum CoreError {
    /// Un identifiant ne respecte pas le format commun.
    #[error("identifiant invalide : {0}")]
    InvalidId(#[from] IdError),

    /// Une date ou un calendrier est invalide.
    #[error("date invalide : {0}")]
    InvalidDate(String),

    /// Les données chargées sont incohérentes.
    #[error("données invalides : {0}")]
    InvalidData(String),

    /// Le document JSON est illisible.
    #[error("JSON invalide : {0}")]
    Json(#[from] serde_json::Error),

    /// Une lecture ou une écriture locale a échoué.
    #[error("entrée/sortie : {0}")]
    Io(#[from] std::io::Error),

    /// La sauvegarde vient d'une version inconnue.
    #[error("version de sauvegarde non supportée : {0}")]
    UnsupportedSaveVersion(u32),

    /// La sauvegarde est incompatible avec cette installation.
    #[error("sauvegarde incompatible : {0}")]
    IncompatibleSave(String),

    /// Le contenu ne correspond pas à son checksum.
    #[error("checksum invalide : attendu {expected}, calculé {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    /// Le générateur aléatoire enregistré n'est pas celui du runtime.
    #[error("algorithme RNG non supporté : {0}")]
    UnsupportedRng(String),
}

impl CoreError {
    /// Construit une erreur de données avec un message statique ou formaté.
    pub fn data(message: impl Into<String>) -> Self {
        Self::InvalidData(message.into())
    }
}
