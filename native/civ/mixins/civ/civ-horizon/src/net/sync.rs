//! Synchronisation état serveur -> clients et gestion des désynchronisations.

use super::protocol::MAX_PACKET_SIZE;
use super::NetMessage;
use crate::civ_core::error::{CoreError, CoreResult};
use crate::civ_core::time::{GameDate, Tick};

/// Delta périodique léger diffusé à chaque tick de simulation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TickDelta {
    pub tick: Tick,
    pub date: GameDate,
    pub checksum: String,
    pub events_count: usize,
}

/// Gestionnaire de diffusion réseau côté serveur.
#[derive(Clone, Debug)]
pub struct SyncManager {
    pub full_sync_interval_ticks: u64,
    pub last_full_sync_tick: u64,
}

impl Default for SyncManager {
    fn default() -> Self {
        Self {
            full_sync_interval_ticks: 200, // Toutes les 10 secondes réelles
            last_full_sync_tick: 0,
        }
    }
}

impl SyncManager {
    pub fn new(interval_ticks: u64) -> Self {
        Self {
            full_sync_interval_ticks: interval_ticks,
            last_full_sync_tick: 0,
        }
    }

    /// Détermine si un snapshot complet doit être envoyé au tick courant.
    pub fn should_send_full_sync(&self, current_tick: u64) -> bool {
        current_tick >= self.last_full_sync_tick + self.full_sync_interval_ticks
    }

    /// Enregistre l'envoi d'un snapshot complet.
    pub fn mark_full_sync_sent(&mut self, current_tick: u64) {
        self.last_full_sync_tick = current_tick;
    }

    /// Prépare le paquet d'instantané de tick.
    pub fn prepare_tick_message(
        &self,
        tick: u64,
        date: GameDate,
        checksum: &str,
    ) -> CoreResult<Vec<u8>> {
        let msg = NetMessage::snapshot(tick, date, checksum);
        let bytes = serde_json::to_vec(&msg)?;
        if bytes.len() > MAX_PACKET_SIZE {
            return Err(CoreError::data("Message de tick trop volumineux"));
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_sync_interval() {
        let mut sync = SyncManager::new(50);
        assert!(sync.should_send_full_sync(50));
        sync.mark_full_sync_sent(50);
        assert!(!sync.should_send_full_sync(60));
        assert!(sync.should_send_full_sync(100));
    }
}
