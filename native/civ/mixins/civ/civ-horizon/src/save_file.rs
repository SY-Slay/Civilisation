//! Façade de sauvegarde pour l'adaptateur.

use std::path::Path;

use crate::civ_core::error::CoreResult;
use crate::civ_core::save::Save;

/// Sauvegarde vérifiée prête à être écrite.
#[derive(Clone, Debug)]
pub struct SaveFile {
    save: Save,
}

impl SaveFile {
    /// Vérifie et emballe une sauvegarde existante.
    pub fn new(save: Save) -> CoreResult<Self> {
        save.check_compatibility()?;
        save.verify()?;
        save.validate_payload()?;
        Ok(Self { save })
    }

    /// Charge une sauvegarde depuis le disque.
    pub fn load(path: &Path) -> CoreResult<Self> {
        Self::new(Save::from_path(path)?)
    }

    /// Accès à la sauvegarde.
    pub fn save(&self) -> &Save {
        &self.save
    }

    /// Écrit sans recalculer le checksum.
    pub fn write(&self, path: &Path) -> CoreResult<()> {
        self.save.write_to_path(path)
    }

    /// Recalcule le checksum puis écrit.
    pub fn seal_and_write(mut self, path: &Path) -> CoreResult<()> {
        self.save.seal()?;
        self.save.write_to_path(path)
    }
}
