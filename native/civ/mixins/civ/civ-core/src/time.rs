//! Temps de simulation indépendant du temps Minecraft.
//!
//! Un tick de simulation est l'unité déterministe du cœur. Le calendrier
//! fictif utilise douze mois de trente jours, sans année bissextile.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// Date du calendrier fictif.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GameDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl GameDate {
    /// Crée une date valide.
    pub fn new(year: i32, month: u8, day: u8) -> Result<Self, CoreError> {
        let date = Self { year, month, day };
        date.validate()?;
        Ok(date)
    }

    /// Vérifie les bornes du schéma commun.
    pub fn validate(&self) -> Result<(), CoreError> {
        if !(1..=99_999).contains(&self.year) {
            return Err(CoreError::InvalidDate(format!(
                "année hors bornes : {}",
                self.year
            )));
        }
        if !(1..=12).contains(&self.month) {
            return Err(CoreError::InvalidDate(format!(
                "mois hors bornes : {}",
                self.month
            )));
        }
        if !(1..=30).contains(&self.day) {
            return Err(CoreError::InvalidDate(format!(
                "jour hors bornes : {}",
                self.day
            )));
        }
        Ok(())
    }

    /// Retourne le jour suivant, en saturant à la fin du calendrier.
    pub fn next_day(self) -> Self {
        let mut year = self.year;
        let mut month = self.month;
        let mut day = self.day + 1;
        if day > 30 {
            day = 1;
            month += 1;
        }
        if month > 12 {
            month = 1;
            year += 1;
        }
        if year > 99_999 {
            year = 99_999;
            month = 12;
            day = 30;
        }
        Self { year, month, day }
    }

    /// Numéro de jour absolu, utile pour les délais déterministes.
    pub fn ordinal(self) -> u64 {
        let year = (self.year.max(1) - 1) as u64;
        let month = (self.month.max(1) - 1) as u64;
        let day = (self.day.max(1) - 1) as u64;
        year * 360 + month * 30 + day
    }

    /// Nombre de jours entre deux dates, positif si `self` est après `other`.
    pub fn days_since(self, other: Self) -> i64 {
        self.ordinal() as i64 - other.ordinal() as i64
    }
}

impl Default for GameDate {
    fn default() -> Self {
        Self {
            year: 2026,
            month: 1,
            day: 1,
        }
    }
}

impl fmt::Display for GameDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

/// Tick de simulation.
#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct Tick(u64);

impl Tick {
    /// Tick zéro.
    pub const ZERO: Self = Self(0);

    /// Crée un tick.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Retourne la valeur brute.
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Addition saturante.
    pub const fn saturating_add(self, other: u64) -> Self {
        Self(self.0.saturating_add(other))
    }
}

impl From<u64> for Tick {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl fmt::Display for Tick {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Horloge de simulation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimClock {
    pub tick: Tick,
    pub date: GameDate,
    pub ticks_per_day: u32,
}

impl SimClock {
    /// Crée une horloge à partir d'une date de départ.
    pub fn new(date: GameDate, ticks_per_day: u32) -> Result<Self, CoreError> {
        if ticks_per_day == 0 {
            return Err(CoreError::InvalidDate(
                "ticks_per_day doit être strictement positif".to_owned(),
            ));
        }
        date.validate()?;
        Ok(Self {
            tick: Tick::ZERO,
            date,
            ticks_per_day,
        })
    }

    /// Avance d'un tick et met à jour la date au changement de jour.
    pub fn advance(&mut self) -> GameDate {
        self.tick = self.tick.saturating_add(1);
        if self.tick.as_u64() % u64::from(self.ticks_per_day) == 0 {
            self.date = self.date.next_day();
        }
        self.date
    }

    /// Avance de plusieurs ticks.
    pub fn advance_ticks(&mut self, ticks: u64) -> GameDate {
        for _ in 0..ticks {
            self.advance();
        }
        self.date
    }
}

impl Default for SimClock {
    fn default() -> Self {
        Self {
            tick: Tick::ZERO,
            date: GameDate::default(),
            ticks_per_day: 2400,
        }
    }
}

#[cfg(all(test, not(feature = "embedded-core")))]
mod tests {
    use super::*;

    #[test]
    fn change_de_mois_et_annee() {
        let date = GameDate::new(2026, 12, 30).unwrap();
        assert_eq!(date.next_day(), GameDate::new(2027, 1, 1).unwrap());
    }

    #[test]
    fn horloge_avance_un_jour() {
        let mut clock = SimClock::new(GameDate::default(), 4).unwrap();
        clock.advance_ticks(4);
        assert_eq!(clock.tick.as_u64(), 4);
        assert_eq!(clock.date.day, 2);
    }

    #[test]
    fn refuse_une_date_invalide() {
        assert!(GameDate::new(0, 1, 1).is_err());
        assert!(GameDate::new(2026, 13, 1).is_err());
        assert!(GameDate::new(2026, 1, 31).is_err());
    }

    #[test]
    fn ordinal_est_strictement_croissant() {
        let a = GameDate::new(2026, 1, 1).unwrap();
        let b = GameDate::new(2026, 1, 2).unwrap();
        assert!(b.ordinal() > a.ordinal());
        assert_eq!(b.days_since(a), 1);
    }
}
