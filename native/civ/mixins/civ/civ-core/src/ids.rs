//! Identifiants stables et typés.
//!
//! Tous les identifiants suivent le motif JSON `^[a-z][a-z0-9_]{2,63}$`.
//! La désérialisation valide le motif : une sauvegarde ne peut pas introduire
//! silencieusement un identifiant malformé.

use std::fmt;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// Erreur de validation d'un identifiant.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum IdError {
    #[error("identifiant vide")]
    Empty,
    #[error("identifiant trop court : {0}")]
    TooShort(String),
    #[error("identifiant trop long : {0}")]
    TooLong(String),
    #[error("l'identifiant doit commencer par une lettre minuscule : {0}")]
    InvalidStart(String),
    #[error("caractère invalide dans l'identifiant : {0}")]
    InvalidChar(String),
}

/// Valide le format commun d'un identifiant.
pub fn validate_id(value: &str) -> Result<(), IdError> {
    if value.is_empty() {
        return Err(IdError::Empty);
    }
    if value.len() < 3 {
        return Err(IdError::TooShort(value.to_owned()));
    }
    if value.len() > 64 {
        return Err(IdError::TooLong(value.to_owned()));
    }
    let bytes = value.as_bytes();
    if !bytes[0].is_ascii_lowercase() {
        return Err(IdError::InvalidStart(value.to_owned()));
    }
    for byte in &bytes[1..] {
        if !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_') {
            return Err(IdError::InvalidChar(value.to_owned()));
        }
    }
    Ok(())
}

/// Déclare un newtype d'identifiant avec sérialisation validante.
macro_rules! define_id {
    ($name:ident) => {
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// Crée un identifiant après validation.
            pub fn new(value: impl Into<String>) -> Result<Self, IdError> {
                let value = value.into();
                validate_id(&value)?;
                Ok(Self(value))
            }

            /// Retourne la représentation textuelle.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Consomme l'identifiant et retourne sa chaîne.
            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = IdError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = <String as Deserialize>::deserialize(deserializer)?;
                Self::new(value).map_err(|error| D::Error::custom(error.to_string()))
            }
        }
    };
}

define_id!(NationId);
define_id!(ProvinceId);
define_id!(BorderId);
define_id!(RegionId);
define_id!(StraitId);
define_id!(ClimateZoneId);
define_id!(ScenarioId);
define_id!(CultureId);
define_id!(ReligionId);
define_id!(IdeologyId);
define_id!(GovernmentId);
define_id!(ResourceId);
define_id!(TechnologyId);
define_id!(UnitTypeId);
define_id!(UnitId);
define_id!(MissileTypeId);
define_id!(MissileInstanceId);
define_id!(EventId);
define_id!(EventChoiceId);
define_id!(WarId);
define_id!(TreatyId);
define_id!(OrganizationId);
define_id!(SanctionId);
define_id!(GuaranteeId);
define_id!(EspionageId);
define_id!(TradeRouteId);
define_id!(WarGoalId);
define_id!(FrontId);
define_id!(BattleId);
define_id!(ArmyId);
define_id!(PeaceOfferId);
define_id!(RevolutionId);
define_id!(FactionId);
define_id!(DiplomaticRequestId);
define_id!(AssaultZoneId);

#[cfg(all(test, not(feature = "embedded-core")))]
mod tests {
    use super::*;

    #[test]
    fn valide_les_identifiants_du_schema() {
        assert!(NationId::new("ardenne").is_ok());
        assert!(NationId::new("prov_0_0").is_ok());
        assert!(NationId::new("civ").is_ok());
        assert!(NationId::new("ab").is_err());
        assert!(NationId::new("Ardenne").is_err());
        assert!(NationId::new("ardenne-1").is_err());
        assert!(NationId::new("1ardenne").is_err());
    }

    #[test]
    fn refuse_les_identifiants_trop_longs() {
        let value = format!("a{}", "b".repeat(64));
        assert!(validate_id(&value).is_err());
    }

    #[test]
    fn serde_refuse_un_identifiant_invalide() {
        let parsed: Result<NationId, _> = serde_json::from_str("\"Ardenne\"");
        assert!(parsed.is_err());
    }
}
