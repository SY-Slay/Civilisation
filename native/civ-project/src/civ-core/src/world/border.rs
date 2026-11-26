//! Frontières partagées entre deux provinces.

use serde::{Deserialize, Serialize};

use crate::ids::{BorderId, ProvinceId};
use crate::math::Polyline;

/// Nature d'une frontière.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BorderKind {
    #[default]
    Land,
    River,
    Mountain,
    Coast,
    Strait,
    Artificial,
}

/// Frontière décrite une seule fois pour une paire de provinces.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Border {
    pub id: BorderId,
    pub a: ProvinceId,
    pub b: ProvinceId,
    #[serde(default)]
    pub kind: BorderKind,
    pub points: Polyline,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub length_blocks: Option<f64>,
    #[serde(default)]
    pub natural_barrier: bool,
    #[serde(default = "default_crossing_cost")]
    pub crossing_cost: f64,
}

fn default_crossing_cost() -> f64 {
    1.0
}

impl Border {
    /// Vrai si la frontière relie la province indiquée à une autre.
    pub fn touches(&self, province: &ProvinceId) -> bool {
        &self.a == province || &self.b == province
    }

    /// Retourne l'autre province, si `province` appartient à la frontière.
    pub fn other(&self, province: &ProvinceId) -> Option<&ProvinceId> {
        if &self.a == province {
            Some(&self.b)
        } else if &self.b == province {
            Some(&self.a)
        } else {
            None
        }
    }

    /// Longueur mesurée si elle n'est pas fournie par les données.
    pub fn effective_length(&self) -> f64 {
        self.length_blocks.unwrap_or_else(|| self.points.length())
    }
}

#[cfg(all(test, not(feature = "embedded-core")))]
mod tests {
    use super::*;
    use crate::math::Vec2;

    #[test]
    fn retrouve_l_autre_province() {
        let border = Border {
            id: BorderId::new("border_test").unwrap(),
            a: ProvinceId::new("prov_a").unwrap(),
            b: ProvinceId::new("prov_b").unwrap(),
            kind: BorderKind::River,
            points: Polyline::new(vec![Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0)]).unwrap(),
            length_blocks: None,
            natural_barrier: false,
            crossing_cost: 1.0,
        };
        assert_eq!(
            border.other(&border.a).map(|id| id.as_str()),
            Some("prov_b")
        );
    }
}
