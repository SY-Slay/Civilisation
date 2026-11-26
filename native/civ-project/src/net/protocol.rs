//! Définition formelle du protocole réseau Horizon pour Civilisations.

use crate::civ_core::diplomacy::DiplomaticRequestKind;
use crate::civ_core::error::{CoreError, CoreResult};
use crate::civ_core::ids::{DiplomaticRequestId, NationId, OrganizationId, ProvinceId};
use serde::{Deserialize, Serialize};

/// Canal serveur -> client.
pub const CHANNEL_CLIENT: &str = "civ:client";
/// Canal client -> serveur.
pub const CHANNEL_SERVER: &str = "civ:server";

/// Taille maximale autorisée pour un message réseau (limite Horizon : 30 000 octets).
pub const MAX_PACKET_SIZE: usize = 28_000;

/// Intentions envoyées par un client au serveur autoritaire.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientIntent {
    /// Commande textuelle `/civ ...`
    Command { line: String },
    /// Demande de sélection de nation
    SelectNation { nation: NationId },
    /// Clic d'inspection de province sur la carte
    InspectProvince { province: ProvinceId },
    /// Proposition diplomatique créée côté serveur après validation.
    ProposeDiplomaticRequest {
        kind: DiplomaticRequestKind,
        from: NationId,
        to: NationId,
    },
    /// Réponse de la nation destinataire à une requête.
    RespondDiplomaticRequest {
        request: DiplomaticRequestId,
        recipient: NationId,
        accept: bool,
    },
    /// Annulation par l'émetteur ou le destinataire.
    CancelDiplomaticRequest {
        request: DiplomaticRequestId,
        actor: NationId,
    },
    /// Inspection d'une alliance.
    InspectAlliance { alliance: OrganizationId },
    /// Demande de snapshot complet
    RequestFullSnapshot,
}

impl ClientIntent {
    pub fn encode(&self) -> CoreResult<Vec<u8>> {
        let bytes = serde_json::to_vec(self)?;
        if bytes.len() > MAX_PACKET_SIZE {
            return Err(CoreError::data(format!(
                "Paquet réseau trop volumineux ({} > {})",
                bytes.len(),
                MAX_PACKET_SIZE
            )));
        }
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> CoreResult<Self> {
        if bytes.len() > MAX_PACKET_SIZE {
            return Err(CoreError::data("Paquet entrant supérieur à la limite"));
        }
        Ok(serde_json::from_slice::<ClientIntent>(bytes)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_intent() {
        let intent = ClientIntent::SelectNation {
            nation: NationId::new("ardenne").unwrap(),
        };
        let bytes = intent.encode().unwrap();
        assert!(bytes.len() < MAX_PACKET_SIZE);
        let dec = ClientIntent::decode(&bytes).unwrap();
        assert_eq!(intent, dec);
    }

    #[test]
    fn round_trip_new_server_intents() {
        let propose = ClientIntent::ProposeDiplomaticRequest {
            kind: crate::civ_core::diplomacy::DiplomaticRequestKind::Alliance,
            from: NationId::new("ardenne").unwrap(),
            to: NationId::new("kesh").unwrap(),
        };
        let respond = ClientIntent::RespondDiplomaticRequest {
            request: DiplomaticRequestId::new("diplo_request_001").unwrap(),
            recipient: NationId::new("kesh").unwrap(),
            accept: true,
        };
        let inspect = ClientIntent::InspectAlliance {
            alliance: OrganizationId::new("alliance_001").unwrap(),
        };
        let cancel = ClientIntent::CancelDiplomaticRequest {
            request: DiplomaticRequestId::new("diplo_request_001").unwrap(),
            actor: NationId::new("ardenne").unwrap(),
        };
        for intent in [propose, respond, cancel, inspect] {
            let bytes = intent.encode().unwrap();
            assert!(bytes.len() <= MAX_PACKET_SIZE);
            assert_eq!(intent, ClientIntent::decode(&bytes).unwrap());
        }
    }
}
