//! Thème visuel, palette de couleurs et typographie de l'interface Civilisations.
//!
//! Ce module centralise les constantes visuelles et helpers de formatage
//! partagés par tous les écrans du mod.

/// Couleur RGBA flottante normalisée [0.0..1.0].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorRgba {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl ColorRgba {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Décode une couleur hexadécimale `#RRGGBB` ou `#RRGGBBAA`.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let clean = hex.strip_prefix('#').unwrap_or(hex);
        match clean.len() {
            6 => {
                let r = u8::from_str_radix(&clean[0..2], 16).ok()? as f32 / 255.0;
                let g = u8::from_str_radix(&clean[2..4], 16).ok()? as f32 / 255.0;
                let b = u8::from_str_radix(&clean[4..6], 16).ok()? as f32 / 255.0;
                Some(Self::new(r, g, b, 1.0))
            }
            8 => {
                let r = u8::from_str_radix(&clean[0..2], 16).ok()? as f32 / 255.0;
                let g = u8::from_str_radix(&clean[2..4], 16).ok()? as f32 / 255.0;
                let b = u8::from_str_radix(&clean[4..6], 16).ok()? as f32 / 255.0;
                let a = u8::from_str_radix(&clean[6..8], 16).ok()? as f32 / 255.0;
                Some(Self::new(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Encode en u32 `0xAARRGGBB` pour les buffers de rendu.
    pub fn to_argb_u32(self) -> u32 {
        let a = (self.a.clamp(0.0, 1.0) * 255.0) as u32;
        let r = (self.r.clamp(0.0, 1.0) * 255.0) as u32;
        let g = (self.g.clamp(0.0, 1.0) * 255.0) as u32;
        let b = (self.b.clamp(0.0, 1.0) * 255.0) as u32;
        (a << 24) | (r << 16) | (g << 8) | b
    }
}

pub mod colors {
    use super::ColorRgba;

    // Interface sombre
    pub const PANEL_BG: ColorRgba = ColorRgba::new(0.08, 0.09, 0.11, 0.92);
    pub const PANEL_BORDER: ColorRgba = ColorRgba::new(0.40, 0.43, 0.49, 1.0);
    pub const HEADER_BG: ColorRgba = ColorRgba::new(0.12, 0.14, 0.18, 0.95);
    pub const ROW_HOVER: ColorRgba = ColorRgba::new(0.20, 0.24, 0.32, 0.8);
    pub const ACCENT: ColorRgba = ColorRgba::new(0.85, 0.65, 0.20, 1.0); // Doré géopolitique
    pub const FOCUS: ColorRgba = ColorRgba::new(1.0, 0.88, 0.40, 1.0);

    // Typographie
    pub const TEXT_PRIMARY: ColorRgba = ColorRgba::new(0.94, 0.95, 0.96, 1.0);
    pub const TEXT_MUTED: ColorRgba = ColorRgba::new(0.60, 0.64, 0.70, 1.0);
    pub const TEXT_HIGHLIGHT: ColorRgba = ColorRgba::new(1.0, 0.88, 0.40, 1.0);

    // Statuts diplomatiques
    pub const STATUS_PEACE: ColorRgba = ColorRgba::new(0.20, 0.65, 0.30, 1.0);
    pub const STATUS_WAR: ColorRgba = ColorRgba::new(1.0, 0.36, 0.36, 1.0);
    pub const STATUS_ALLIANCE: ColorRgba = ColorRgba::new(0.15, 0.55, 0.85, 1.0);
    pub const STATUS_TENSION: ColorRgba = ColorRgba::new(0.90, 0.50, 0.10, 1.0);
    pub const STATUS_NEUTRAL: ColorRgba = ColorRgba::new(0.50, 0.50, 0.55, 1.0);

    // Carte
    pub const MAP_UNCLAIMED: ColorRgba = ColorRgba::new(0.52, 0.56, 0.62, 1.0);
    pub const MAP_BORDER: ColorRgba = ColorRgba::new(0.10, 0.10, 0.12, 0.8);
    pub const MAP_HOVER_BORDER: ColorRgba = ColorRgba::new(1.0, 1.0, 1.0, 0.9);
}

/// Formate un grand nombre de façon compacte (ex: 1.2M, 450k).
pub fn format_compact_number(val: u64) -> String {
    if val >= 1_000_000_000 {
        format!("{:.1}B", val as f64 / 1_000_000_000.0)
    } else if val >= 1_000_000 {
        format!("{:.1}M", val as f64 / 1_000_000.0)
    } else if val >= 1_000 {
        format!("{:.1}k", val as f64 / 1_000.0)
    } else {
        val.to_string()
    }
}

/// Formate un pourcentage de 0.0 à 1.0 (ex: "74.5%").
pub fn format_percent(val: f64) -> String {
    format!("{:.1}%", (val * 100.0).clamp(0.0, 100.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_colors() {
        let col = ColorRgba::from_hex("#FF0000").unwrap();
        assert_eq!(col, ColorRgba::new(1.0, 0.0, 0.0, 1.0));

        let col_alpha = ColorRgba::from_hex("#00FF0080").unwrap();
        assert!((col_alpha.g - 1.0).abs() < 1e-3);
        assert!((col_alpha.a - 0.5019).abs() < 1e-2);
    }

    #[test]
    fn format_numbers() {
        assert_eq!(format_compact_number(450), "450");
        assert_eq!(format_compact_number(12_500), "12.5k");
        assert_eq!(format_compact_number(3_400_000), "3.4M");
        assert_eq!(format_percent(0.852), "85.2%");
    }
}
