//! Nations, économie, armée et technologie.

pub mod economy;
pub mod military;
pub mod model;
pub mod technology;

pub use economy::{Economy, EconomySectors, SanctionRef};
pub use military::{Doctrine, Military};
pub use model::{
    AiArchetype, AiProfile, Claim, ClaimKind, Flag, FlagPattern, Leader, Nation, NationPopulation,
    RedLine, Subject, SubjectKind,
};
pub use technology::TechnologyState;
