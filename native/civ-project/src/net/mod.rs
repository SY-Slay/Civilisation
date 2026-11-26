//! Protocole réseau minimal, sérialisé en JSON.
//!
//! Les messages sont envoyés par `Host::send_net`. Ce module ne crée aucune
//! socket et ne dépend d'aucune API réseau Horizon.

pub mod protocol;
pub mod sync;

use serde::{Deserialize, Serialize};

use crate::civ_core::error::CoreResult;
use crate::civ_core::time::GameDate;

use crate::ui::UiModel;

/// Version du protocole applicatif.
pub const PROTOCOL_VERSION: u32 = 1;

/// Niveau d'un message affiché au client.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeLevel {
    Info,
    Warn,
    Error,
}

/// Message échangé entre serveur et clients.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NetMessage {
    Hello {
        protocol: u32,
        mod_version: String,
    },
    Snapshot {
        protocol: u32,
        tick: u64,
        date: GameDate,
        checksum: String,
    },
    Command {
        line: String,
    },
    Notice {
        level: NoticeLevel,
        message: String,
    },
    /// Modèle complet destiné à l'écran et au HUD du client.
    FullSnapshot {
        protocol: u32,
        model: UiModel,
    },
    /// Demande de séquence cinématique (impact de missile, déclaration de guerre).
    Cinematic {
        protocol: u32,
        center: [f64; 3],
        radius: f32,
        height: f32,
        duration_seconds: f32,
        title: String,
        subtitle: String,
    },
    /// Secousse de caméra demandée par la simulation serveur.
    CameraShake {
        intensity: f32,
        duration_ticks: u32,
    },
}

impl NetMessage {
    /// Construit le message de poignée de main.
    pub fn hello(mod_version: impl Into<String>) -> Self {
        Self::Hello {
            protocol: PROTOCOL_VERSION,
            mod_version: mod_version.into(),
        }
    }

    /// Construit un instantané léger.
    pub fn snapshot(tick: u64, date: GameDate, checksum: impl Into<String>) -> Self {
        Self::Snapshot {
            protocol: PROTOCOL_VERSION,
            tick,
            date,
            checksum: checksum.into(),
        }
    }

    /// Construit un instantané complet de l'interface.
    pub fn full_snapshot(model: UiModel) -> Self {
        Self::FullSnapshot {
            protocol: PROTOCOL_VERSION,
            model,
        }
    }

    /// Construit une demande de cinématique orbitale.
    pub fn cinematic(
        center: [f64; 3],
        radius: f32,
        height: f32,
        duration_seconds: f32,
        title: impl Into<String>,
        subtitle: impl Into<String>,
    ) -> Self {
        Self::Cinematic {
            protocol: PROTOCOL_VERSION,
            center,
            radius,
            height,
            duration_seconds,
            title: title.into(),
            subtitle: subtitle.into(),
        }
    }

    /// Construit une secousse de caméra.
    pub fn camera_shake(intensity: f32, duration_ticks: u32) -> Self {
        Self::CameraShake {
            intensity,
            duration_ticks,
        }
    }
}

/// Encode un message en JSON compact.
pub fn encode(message: &NetMessage) -> CoreResult<Vec<u8>> {
    Ok(serde_json::to_vec(message)?)
}

/// Décode un message JSON.
pub fn decode(bytes: &[u8]) -> CoreResult<NetMessage> {
    Ok(serde_json::from_slice::<NetMessage>(bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let message = NetMessage::hello("0.1.0");
        let bytes = encode(&message).unwrap();
        let decoded = decode(&bytes).unwrap();
        assert_eq!(message, decoded);
    }

    #[test]
    fn snapshot_contient_la_version() {
        let message = NetMessage::snapshot(12, GameDate::default(), "abc");
        match message {
            NetMessage::Snapshot { protocol, .. } => assert_eq!(protocol, PROTOCOL_VERSION),
            _ => panic!("mauvais message"),
        }
    }
}
