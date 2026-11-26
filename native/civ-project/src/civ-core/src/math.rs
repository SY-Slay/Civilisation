//! Géométrie 2D déterministe de la carte.
//!
//! Les provinces sont des polygones irréguliers en coordonnées blocs
//! `[x, z]`. Le cœur ne convertit jamais ces coordonnées en coordonnées
//! Minecraft : l'adaptateur s'en charge.

use serde::{Deserialize, Serialize};

use crate::error::{CoreError, CoreResult};

const EPSILON: f64 = 1.0e-9;

impl std::ops::Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0, self.1 + other.1)
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0, self.1 - other.1)
    }
}

/// Point ou vecteur 2D `[x, z]`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec2(pub f64, pub f64);

impl Vec2 {
    /// Construit un point.
    pub const fn new(x: f64, z: f64) -> Self {
        Self(x, z)
    }

    /// Coordonnée X.
    pub const fn x(self) -> f64 {
        self.0
    }

    /// Coordonnée Z.
    pub const fn z(self) -> f64 {
        self.1
    }

    /// Multiplication scalaire.
    pub fn scale(self, factor: f64) -> Self {
        Self(self.0 * factor, self.1 * factor)
    }

    /// Produit scalaire.
    pub fn dot(self, other: Self) -> f64 {
        self.0 * other.0 + self.1 * other.1
    }

    /// Produit vectoriel 2D.
    pub fn cross(self, other: Self) -> f64 {
        self.0 * other.1 - self.1 * other.0
    }

    /// Longueur euclidienne.
    pub fn length(self) -> f64 {
        self.dot(self).sqrt()
    }

    /// Distance à un autre point.
    pub fn distance(self, other: Self) -> f64 {
        (self - other).length()
    }

    /// Interpolation linéaire.
    pub fn lerp(self, other: Self, t: f64) -> Self {
        Self(
            self.0 + (other.0 - self.0) * t,
            self.1 + (other.1 - self.1) * t,
        )
    }
}

/// Point 3D `[x, y, z]` utilisé pour les trajectoires de missiles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Vec3(pub f64, pub f64, pub f64);

impl Vec3 {
    /// Construit un point 3D.
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self(x, y, z)
    }

    /// Coordonnée X.
    pub const fn x(self) -> f64 {
        self.0
    }

    /// Coordonnée Y.
    pub const fn y(self) -> f64 {
        self.1
    }

    /// Coordonnée Z.
    pub const fn z(self) -> f64 {
        self.2
    }

    /// Distance à un autre point.
    pub fn distance(self, other: Self) -> f64 {
        let dx = self.0 - other.0;
        let dy = self.1 - other.1;
        let dz = self.2 - other.2;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Polyligne ouverte, par exemple une frontière partagée.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Polyline(pub Vec<Vec2>);

impl Polyline {
    /// Crée une polyligne d'au moins deux points.
    pub fn new(points: Vec<Vec2>) -> CoreResult<Self> {
        if points.len() < 2 {
            return Err(CoreError::data("une polyligne exige au moins deux points"));
        }
        Ok(Self(points))
    }

    /// Longueur totale de la polyligne.
    pub fn length(&self) -> f64 {
        self.0
            .windows(2)
            .map(|segment| segment[0].distance(segment[1]))
            .sum()
    }

    /// Points de la polyligne.
    pub fn points(&self) -> &[Vec2] {
        &self.0
    }
}

/// Polygone simple, sans auto-intersection.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Polygon(pub Vec<Vec2>);

impl Polygon {
    /// Crée un polygone simple d'au moins trois points.
    pub fn new(points: Vec<Vec2>) -> CoreResult<Self> {
        let polygon = Self(points);
        polygon.validate()?;
        Ok(polygon)
    }

    /// Vérifie la taille, l'aire et l'absence d'auto-intersection.
    pub fn validate(&self) -> CoreResult<()> {
        if self.0.len() < 3 {
            return Err(CoreError::data("un polygone exige au moins trois points"));
        }
        if self.area() <= EPSILON {
            return Err(CoreError::data("un polygone doit avoir une aire positive"));
        }
        if !self.is_simple() {
            return Err(CoreError::data(
                "un polygone de province ne doit pas s'auto-intersecter",
            ));
        }
        Ok(())
    }

    /// Aire absolue du polygone.
    pub fn area(&self) -> f64 {
        let count = self.0.len();
        if count < 3 {
            return 0.0;
        }
        let mut twice_area = 0.0;
        for index in 0..count {
            let current = self.0[index];
            let next = self.0[(index + 1) % count];
            twice_area += current.cross(next);
        }
        (twice_area * 0.5).abs()
    }

    /// Centre de masse du polygone.
    pub fn centroid(&self) -> Vec2 {
        let count = self.0.len();
        if count == 0 {
            return Vec2::default();
        }
        if count < 3 {
            let sum = self
                .0
                .iter()
                .fold(Vec2::default(), |accumulator, point| accumulator + *point);
            return sum.scale(1.0 / count as f64);
        }
        let mut twice_area = 0.0;
        let mut x = 0.0;
        let mut z = 0.0;
        for index in 0..count {
            let current = self.0[index];
            let next = self.0[(index + 1) % count];
            let cross = current.cross(next);
            twice_area += cross;
            x += (current.x() + next.x()) * cross;
            z += (current.z() + next.z()) * cross;
        }
        if twice_area.abs() <= EPSILON {
            let sum = self
                .0
                .iter()
                .fold(Vec2::default(), |accumulator, point| accumulator + *point);
            return sum.scale(1.0 / count as f64);
        }
        let factor = 1.0 / (3.0 * twice_area);
        Vec2::new(x * factor, z * factor)
    }

    /// Test d'appartenance par lancer de rayon.
    pub fn contains(&self, point: Vec2) -> bool {
        if self.0.len() < 3 {
            return false;
        }
        let mut inside = false;
        let count = self.0.len();
        let mut previous = self.0[count - 1];
        for current in &self.0 {
            let current = *current;
            let crosses = (current.z() > point.z()) != (previous.z() > point.z());
            if crosses {
                let denominator = previous.z() - current.z();
                if denominator.abs() > EPSILON {
                    let intersection_x = (previous.x() - current.x()) * (point.z() - current.z())
                        / denominator
                        + current.x();
                    if point.x() < intersection_x {
                        inside = !inside;
                    }
                }
            }
            previous = current;
        }
        inside
    }

    /// Boîte englobante `(min, max)`.
    pub fn bounds(&self) -> (Vec2, Vec2) {
        if self.0.is_empty() {
            return (Vec2::default(), Vec2::default());
        }
        let mut min = self.0[0];
        let mut max = self.0[0];
        for point in &self.0[1..] {
            min = Vec2::new(min.x().min(point.x()), min.z().min(point.z()));
            max = Vec2::new(max.x().max(point.x()), max.z().max(point.z()));
        }
        (min, max)
    }

    /// Vrai si aucun segment non adjacent ne se croise.
    pub fn is_simple(&self) -> bool {
        let count = self.0.len();
        if count < 3 {
            return false;
        }
        for first in 0..count {
            let first_start = self.0[first];
            let first_end = self.0[(first + 1) % count];
            for second in (first + 1)..count {
                if second == first + 1 {
                    continue;
                }
                if first == 0 && second == count - 1 {
                    continue;
                }
                let second_start = self.0[second];
                let second_end = self.0[(second + 1) % count];
                if segments_intersect(first_start, first_end, second_start, second_end) {
                    return false;
                }
            }
        }
        true
    }

    /// Points du contour.
    pub fn points(&self) -> &[Vec2] {
        &self.0
    }
}

/// Vrai si deux segments se touchent ou se croisent.
pub fn segments_intersect(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> bool {
    let o1 = orientation(a1, a2, b1);
    let o2 = orientation(a1, a2, b2);
    let o3 = orientation(b1, b2, a1);
    let o4 = orientation(b1, b2, a2);

    if ((o1 > EPSILON && o2 < -EPSILON) || (o1 < -EPSILON && o2 > EPSILON))
        && ((o3 > EPSILON && o4 < -EPSILON) || (o3 < -EPSILON && o4 > EPSILON))
    {
        return true;
    }
    if o1.abs() <= EPSILON && point_on_segment(a1, a2, b1) {
        return true;
    }
    if o2.abs() <= EPSILON && point_on_segment(a1, a2, b2) {
        return true;
    }
    if o3.abs() <= EPSILON && point_on_segment(b1, b2, a1) {
        return true;
    }
    if o4.abs() <= EPSILON && point_on_segment(b1, b2, a2) {
        return true;
    }
    false
}

fn orientation(a: Vec2, b: Vec2, c: Vec2) -> f64 {
    (b - a).cross(c - a)
}

fn point_on_segment(a: Vec2, b: Vec2, point: Vec2) -> bool {
    if orientation(a, b, point).abs() > EPSILON {
        return false;
    }
    let dot = (point - a).dot(b - a);
    let length_squared = (b - a).dot(b - a);
    dot >= -EPSILON && dot <= length_squared + EPSILON
}

#[cfg(all(test, not(feature = "embedded-core")))]
mod tests {
    use super::*;

    fn square() -> Polygon {
        Polygon::new(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(0.0, 10.0),
        ])
        .unwrap()
    }

    #[test]
    fn aire_et_centre_d_un_carre() {
        let polygon = square();
        assert!((polygon.area() - 100.0).abs() < 1.0e-9);
        let center = polygon.centroid();
        assert!((center.x() - 5.0).abs() < 1.0e-9);
        assert!((center.z() - 5.0).abs() < 1.0e-9);
    }

    #[test]
    fn point_interieur_exterieur() {
        let polygon = square();
        assert!(polygon.contains(Vec2::new(5.0, 5.0)));
        assert!(!polygon.contains(Vec2::new(15.0, 5.0)));
    }

    #[test]
    fn detecte_une_auto_intersection() {
        let polygon = Polygon(vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(10.0, 10.0),
            Vec2::new(10.0, 0.0),
            Vec2::new(0.0, 10.0),
        ]);
        assert!(!polygon.is_simple());
        assert!(polygon.validate().is_err());
    }

    #[test]
    fn polyligne_longueur() {
        let line = Polyline::new(vec![Vec2::new(0.0, 0.0), Vec2::new(3.0, 4.0)]).unwrap();
        assert!((line.length() - 5.0).abs() < 1.0e-9);
        assert!(Polyline::new(vec![Vec2::new(0.0, 0.0)]).is_err());
    }
}
