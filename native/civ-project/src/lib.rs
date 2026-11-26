//! Civilisations — adaptateur testable pour Horizon.
//!
//! Le bundle natif d'Horizon n'autorise pas les dépendances `path` entre
//! crates. Pour rester compilable comme une seule crate native, ce crate
//! inclut les modules purs de `civ-core` à sa racine, puis expose l'adaptateur
//! (`commands`, `host`, `net`, `ui`, `cinematic`, `render`) au-dessus.
//!
//! `civ-core` reste une crate autonome pour les tests et la validation du
//! noyau. L'alias public `civ_core` permet au code de l'adaptateur de garder
//! ses chemins `crate::civ_core::...` sans dépendance de registre.

#![forbid(unsafe_code)]

// ---- Noyau pur, inclus à la racine pour le bundle natif ----

#[path = "civ-core/src/data/mod.rs"]
pub mod data;
#[path = "civ-core/src/diplomacy/mod.rs"]
pub mod diplomacy;
#[path = "civ-core/src/effects.rs"]
pub mod effects;
#[path = "civ-core/src/error.rs"]
pub mod error;
#[path = "civ-core/src/event.rs"]
pub mod event;
#[path = "civ-core/src/ids.rs"]
pub mod ids;
#[path = "civ-core/src/math.rs"]
pub mod math;
#[path = "civ-core/src/missile/mod.rs"]
pub mod missile;
#[path = "civ-core/src/nation/mod.rs"]
pub mod nation;
#[path = "civ-core/src/revolution/mod.rs"]
pub mod revolution;
#[path = "civ-core/src/rng.rs"]
pub mod rng;
#[path = "civ-core/src/save/mod.rs"]
pub mod save;
#[path = "civ-core/src/scenario.rs"]
pub mod scenario;
#[path = "civ-core/src/sim/mod.rs"]
pub mod sim;
#[path = "civ-core/src/text.rs"]
pub mod text;
#[path = "civ-core/src/time.rs"]
pub mod time;
#[path = "civ-core/src/validation.rs"]
pub mod validation;
#[path = "civ-core/src/war/mod.rs"]
pub mod war;
#[path = "civ-core/src/world/mod.rs"]
pub mod world;

/// Alias interne et public du noyau inclus.
pub use crate as civ_core;
pub use error::{CoreError, CoreResult};
pub use save::Save;
pub use scenario::{Difficulty, Scenario};

// ---- Adaptateur au-dessus du noyau ----

pub mod cinematic;
pub mod commands;
pub mod host;
pub mod net;
pub mod render;
pub mod render_effects;
pub mod save_file;
pub mod state;
pub mod ui;

pub use commands::{Command, CommandError};
pub use host::{Host, RecordingHost};
pub use state::{Session, SessionStatus};
