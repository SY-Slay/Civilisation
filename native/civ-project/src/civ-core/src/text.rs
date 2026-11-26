//! Textes localisables.
//!
//! Une donnée peut contenir soit un texte direct, soit une clé de
//! localisation avec un repli optionnel. Le cœur ne dépend d'aucun moteur de
//! traduction : il expose la clé et le repli, l'adaptateur choisit la langue.

use serde::{Deserialize, Serialize};

/// Texte direct ou clé de localisation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LocalizedText {
    /// Clé stable avec repli anglais facultatif.
    Key {
        key: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        default: Option<String>,
    },
    /// Texte direct.
    Plain(String),
}

impl LocalizedText {
    /// Construit un texte direct.
    pub fn plain(value: impl Into<String>) -> Self {
        Self::Plain(value.into())
    }

    /// Construit une clé de localisation.
    pub fn key(value: impl Into<String>) -> Self {
        Self::Key {
            key: value.into(),
            default: None,
        }
    }

    /// Retourne le texte à afficher si aucune traduction n'est disponible.
    pub fn resolve(&self) -> &str {
        match self {
            Self::Key { key, default } => default.as_deref().unwrap_or(key),
            Self::Plain(value) => value,
        }
    }

    /// Vrai si la valeur est une clé de localisation.
    pub fn is_key(&self) -> bool {
        matches!(self, Self::Key { .. })
    }
}

impl From<&str> for LocalizedText {
    fn from(value: &str) -> Self {
        Self::Plain(value.to_owned())
    }
}

impl From<String> for LocalizedText {
    fn from(value: String) -> Self {
        Self::Plain(value)
    }
}

impl std::fmt::Display for LocalizedText {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.resolve())
    }
}

#[cfg(all(test, not(feature = "embedded-core")))]
mod tests {
    use super::*;

    #[test]
    fn texte_direct() {
        let text: LocalizedText = serde_json::from_str("\"Ardenne\"").unwrap();
        assert_eq!(text.resolve(), "Ardenne");
        assert!(!text.is_key());
    }

    #[test]
    fn cle_avec_repli() {
        let text: LocalizedText =
            serde_json::from_str(r#"{"key":"nation.ardenne","default":"Ardenne"}"#).unwrap();
        assert_eq!(text.resolve(), "Ardenne");
        assert!(text.is_key());
    }
}
