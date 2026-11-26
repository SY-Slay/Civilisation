//! Vue interactive de la carte géopolitique.
//!
//! Gère la projection des coordonnées mondiales sur l'écran, le déplacement (pan),
//! le niveau de zoom, et le survol/sélection des provinces.

use super::theme::{colors, ColorRgba};
use super::widgets::{DrawCommand, Rect};
use super::{MapMarker, UiModel};
use crate::civ_core::ids::ProvinceId;

const FIT_PADDING: f32 = 24.0;
const MIN_ZOOM: f32 = 0.02;
const MAX_ZOOM: f32 = 8.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct FitKey {
    bounds: Rect,
    min_x: f64,
    max_x: f64,
    min_z: f64,
    max_z: f64,
}

/// État de navigation de la carte 2D.
#[derive(Clone, Debug)]
pub struct MapView {
    pub bounds: Rect,
    pub offset_x: f32,
    pub offset_z: f32,
    pub zoom: f32,
    pub hovered_province: Option<ProvinceId>,
    pub selected_province: Option<ProvinceId>,
    pub focused_province: Option<ProvinceId>,
    fit_key: Option<FitKey>,
}

impl MapView {
    pub fn new(bounds: Rect) -> Self {
        Self {
            bounds,
            offset_x: 0.0,
            offset_z: 0.0,
            zoom: 1.0,
            hovered_province: None,
            selected_province: None,
            focused_province: None,
            fit_key: None,
        }
    }

    /// Ajuste la vue à toutes les géométries de province, sans réinitialiser
    /// le pan ou le zoom tant que les limites et la zone de rendu ne changent pas.
    pub fn fit_to_markers(&mut self, markers: &[MapMarker]) -> bool {
        let mut extents: Option<(f64, f64, f64, f64)> = None;
        for marker in markers {
            let points = marker
                .polygon
                .iter()
                .map(|point| (point[0], point[1]))
                .chain(std::iter::once((marker.x, marker.z)));
            for (x, z) in points {
                if !x.is_finite() || !z.is_finite() {
                    continue;
                }
                extents = Some(match extents {
                    Some((min_x, max_x, min_z, max_z)) => {
                        (min_x.min(x), max_x.max(x), min_z.min(z), max_z.max(z))
                    }
                    None => (x, x, z, z),
                });
            }
        }

        let Some((min_x, max_x, min_z, max_z)) = extents else {
            self.fit_key = None;
            return false;
        };
        let key = FitKey {
            bounds: self.bounds,
            min_x,
            max_x,
            min_z,
            max_z,
        };
        if self.fit_key == Some(key) {
            return false;
        }

        self.fit_key = Some(key);
        self.offset_x = ((min_x + max_x) * 0.5) as f32;
        self.offset_z = ((min_z + max_z) * 0.5) as f32;
        let span_x = (max_x - min_x).max(1.0) as f32;
        let span_z = (max_z - min_z).max(1.0) as f32;
        let available_width = (self.bounds.width - FIT_PADDING * 2.0).max(1.0);
        let available_height = (self.bounds.height - FIT_PADDING * 2.0).max(1.0);
        self.zoom = (available_width / span_x)
            .min(available_height / span_z)
            .clamp(MIN_ZOOM, MAX_ZOOM);
        true
    }

    /// Garde la province focalisée visible après un déplacement ou un zoom.
    pub fn keep_focused_visible(&mut self, markers: &[MapMarker]) {
        let Some(focused) = self.focused_province.as_ref() else {
            return;
        };
        let Some(marker) = markers.iter().find(|marker| &marker.province == focused) else {
            return;
        };
        let (sx, sy) = self.world_to_screen(marker.x, marker.z);
        if !self.bounds.shrink(8.0).contains(sx, sy) {
            self.offset_x = marker.x as f32;
            self.offset_z = marker.z as f32;
        }
    }

    /// Convertit des coordonnées monde (X, Z) en pixels écran.
    pub fn world_to_screen(&self, wx: f64, wz: f64) -> (f32, f32) {
        let center_x = self.bounds.x + self.bounds.width * 0.5;
        let center_y = self.bounds.y + self.bounds.height * 0.5;
        let sx = center_x + (wx as f32 - self.offset_x) * self.zoom;
        let sy = center_y + (wz as f32 - self.offset_z) * self.zoom;
        (sx, sy)
    }

    /// Convertit des coordonnées écran en coordonnées monde (X, Z).
    pub fn screen_to_world(&self, sx: f32, sy: f32) -> (f64, f64) {
        let center_x = self.bounds.x + self.bounds.width * 0.5;
        let center_y = self.bounds.y + self.bounds.height * 0.5;
        let wx = ((sx - center_x) / self.zoom + self.offset_x) as f64;
        let wz = ((sy - center_y) / self.zoom + self.offset_z) as f64;
        (wx, wz)
    }

    /// Met à jour la province survolée en fonction de la position du curseur.
    pub fn update_cursor(&mut self, cursor_x: f32, cursor_y: f32, markers: &[MapMarker]) {
        if !self.bounds.contains(cursor_x, cursor_y) {
            self.hovered_province = None;
            return;
        }

        // Rayon de sélection basé sur le zoom
        let hit_radius = 20.0 * self.zoom;
        let mut closest = None;
        let mut min_dist_sq = hit_radius * hit_radius;

        for marker in markers {
            let (sx, sy) = self.world_to_screen(marker.x, marker.z);
            let dx = cursor_x - sx;
            let dy = cursor_y - sy;
            let d_sq = dx * dx + dy * dy;
            if d_sq < min_dist_sq {
                min_dist_sq = d_sq;
                closest = Some(marker.province.clone());
            }
        }

        self.hovered_province = closest;
    }

    /// Gère un clic pour sélectionner une province.
    pub fn handle_click(&mut self, cx: f32, cy: f32) -> bool {
        if self.bounds.contains(cx, cy) {
            self.selected_province = self.hovered_province.clone();
            true
        } else {
            false
        }
    }

    /// Keyboard province navigation with stable model order.
    pub fn handle_key(&mut self, key: &str, markers: &[MapMarker]) -> bool {
        if markers.is_empty() {
            return false;
        }
        let current = self.focused_province.as_ref().and_then(|province| {
            markers
                .iter()
                .position(|marker| &marker.province == province)
        });
        let next = match key.to_ascii_lowercase().as_str() {
            "arrowleft" | "left" => current
                .map(|index| index.saturating_sub(1))
                .unwrap_or(markers.len() - 1),
            "arrowright" | "right" => current
                .map(|index| (index + 1) % markers.len())
                .unwrap_or(0),
            "arrowup" | "up" => current
                .map(|index| index.saturating_sub(1))
                .unwrap_or(markers.len() - 1),
            "arrowdown" | "down" => current
                .map(|index| (index + 1) % markers.len())
                .unwrap_or(0),
            "enter" | " " => {
                if let Some(province) = self.focused_province.clone() {
                    self.selected_province = Some(province);
                    return true;
                }
                0
            }
            "tab" => current
                .map(|index| (index + 1) % markers.len())
                .unwrap_or(0),
            "+" | "=" => {
                self.zoom = (self.zoom * 1.2).min(MAX_ZOOM);
                self.keep_focused_visible(markers);
                return true;
            }
            "-" | "_" => {
                self.zoom = (self.zoom / 1.2).max(MIN_ZOOM);
                self.keep_focused_visible(markers);
                return true;
            }
            _ => return false,
        };
        self.focused_province = Some(markers[next].province.clone());
        self.keep_focused_visible(markers);
        true
    }

    /// Rendu des éléments de la carte.
    pub fn render(&self, model: &UiModel, out: &mut Vec<DrawCommand>) {
        // Fond de carte
        out.push(DrawCommand::FillRect {
            rect: self.bounds,
            color: ColorRgba::new(0.05, 0.06, 0.07, 1.0),
        });

        if model.map.is_empty() {
            out.push(DrawCommand::Text {
                x: self.bounds.x + 12.0,
                y: self.bounds.y + 12.0,
                text: "Aucune province disponible".to_owned(),
                color: colors::TEXT_PRIMARY,
                scale: 1.0,
            });
            return;
        }

        // Marqueurs de provinces
        for marker in &model.map {
            let (sx, sy) = self.world_to_screen(marker.x, marker.z);
            let is_hovered = self.hovered_province.as_ref() == Some(&marker.province);
            let is_selected = self.selected_province.as_ref() == Some(&marker.province);
            let is_focused = self.focused_province.as_ref() == Some(&marker.province);

            let marker_color = if let Some(owner) = &marker.owner {
                model
                    .nations
                    .iter()
                    .find(|n| &n.id == owner)
                    .and_then(|n| ColorRgba::from_hex(&n.color))
                    .unwrap_or(colors::MAP_UNCLAIMED)
            } else {
                colors::MAP_UNCLAIMED
            };

            for index in 0..marker.polygon.len() {
                let first = &marker.polygon[index];
                let second = &marker.polygon[(index + 1) % marker.polygon.len()];
                let start = self.world_to_screen(first[0], first[1]);
                let end = self.world_to_screen(second[0], second[1]);
                if let Some((start, end)) = clip_segment(self.bounds, start, end) {
                    out.push(DrawCommand::Line {
                        start,
                        end,
                        color: marker_color,
                        thickness: 2.0,
                    });
                }
            }

            if !self.bounds.contains(sx, sy) {
                continue;
            }
            let radius = if is_selected {
                8.0
            } else if is_hovered {
                6.0
            } else {
                4.0
            };

            out.push(DrawCommand::FillRect {
                rect: Rect::new(sx - radius, sy - radius, radius * 2.0, radius * 2.0),
                color: marker_color,
            });

            if is_hovered || is_selected || is_focused {
                out.push(DrawCommand::StrokeRect {
                    rect: Rect::new(
                        sx - radius - 2.0,
                        sy - radius - 2.0,
                        (radius + 2.0) * 2.0,
                        (radius + 2.0) * 2.0,
                    ),
                    color: if is_focused {
                        colors::FOCUS
                    } else if is_selected {
                        colors::ACCENT
                    } else {
                        colors::MAP_HOVER_BORDER
                    },
                    thickness: if is_focused { 3.0 } else { 2.0 },
                });
            }

            // Nom de la province si zoom suffisant ou survolé
            if self.zoom >= 0.8 || is_hovered || is_selected || is_focused {
                out.push(DrawCommand::Text {
                    x: sx + radius + 4.0,
                    y: sy - 4.0,
                    text: marker.name.clone(),
                    color: if is_hovered {
                        colors::TEXT_HIGHLIGHT
                    } else {
                        colors::TEXT_PRIMARY
                    },
                    scale: 1.0,
                });
            }
            if marker.contested {
                out.push(DrawCommand::Text {
                    x: sx + radius + 4.0,
                    y: sy + 8.0,
                    text: format!("Contestée {:.0}", marker.assault_score),
                    color: colors::TEXT_HIGHLIGHT,
                    scale: 0.85,
                });
            }
        }

        out.push(DrawCommand::Text {
            x: self.bounds.x + 8.0,
            y: self.bounds.y + self.bounds.height - 18.0,
            text: "Tab : province · Entrée : sélectionner · flèches : déplacer · +/- : zoom"
                .to_owned(),
            color: colors::TEXT_MUTED,
            scale: 1.0,
        });
    }
}

fn clip_segment(
    bounds: Rect,
    start: (f32, f32),
    end: (f32, f32),
) -> Option<((f32, f32), (f32, f32))> {
    let dx = end.0 - start.0;
    let dy = end.1 - start.1;
    let p = [-dx, dx, -dy, dy];
    let q = [
        start.0 - bounds.x,
        bounds.x + bounds.width - start.0,
        start.1 - bounds.y,
        bounds.y + bounds.height - start.1,
    ];
    let mut enter: f32 = 0.0;
    let mut exit: f32 = 1.0;

    for index in 0..4 {
        if p[index] == 0.0 {
            if q[index] < 0.0 {
                return None;
            }
            continue;
        }
        let ratio = q[index] / p[index];
        if p[index] < 0.0 {
            enter = enter.max(ratio);
        } else {
            exit = exit.min(ratio);
        }
        if enter > exit {
            return None;
        }
    }

    let clamp = |point: (f32, f32)| {
        (
            point.0.clamp(bounds.x, bounds.x + bounds.width),
            point.1.clamp(bounds.y, bounds.y + bounds.height),
        )
    };
    Some((
        clamp((start.0 + enter * dx, start.1 + enter * dy)),
        clamp((start.0 + exit * dx, start.1 + exit * dy)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civ_core::ids::NationId;
    use crate::civ_core::time::{GameDate, Tick};
    use crate::ui::{DiplomacySummary, NationSummary, UiModel};

    fn marker(id: &str, x: f64, z: f64, polygon: Vec<[f64; 2]>) -> MapMarker {
        MapMarker {
            province: ProvinceId::new(id).unwrap(),
            name: id.to_owned(),
            owner: None,
            controller: None,
            x,
            z,
            polygon,
            terrain: "plains".to_owned(),
            unrest: 0.0,
            contested: false,
            assault_score: 0.0,
        }
    }

    fn ui_model(map: Vec<MapMarker>) -> UiModel {
        UiModel {
            date: GameDate::new(2026, 1, 1).unwrap(),
            tick: Tick::new(0),
            player_nation: None,
            nations: Vec::<NationSummary>::new(),
            map,
            diplomacy: DiplomacySummary {
                relations: 0,
                active_treaties: 0,
                organizations: 0,
                sanctions: 0,
                guarantees: 0,
                war_pairs: 0,
            },
            wars: Vec::new(),
            missiles: Vec::new(),
            diplomatic_requests: Vec::new(),
            assault_zones: Vec::new(),
            alliances: Vec::new(),
            weekly_ranking: Vec::<(NationId, u32, Option<u32>, f64)>::new(),
        }
    }

    #[test]
    fn coordinate_transformations() {
        let view = MapView::new(Rect::new(0.0, 0.0, 800.0, 600.0));
        let (sx, sy) = view.world_to_screen(0.0, 0.0);
        assert_eq!((sx, sy), (400.0, 300.0));

        let (wx, wz) = view.screen_to_world(400.0, 300.0);
        assert!((wx - 0.0).abs() < 1e-4);
        assert!((wz - 0.0).abs() < 1e-4);
    }

    #[test]
    fn fit_places_non_origin_province_geometry_inside_the_map() {
        let markers = vec![
            marker(
                "province_west",
                1_000.0,
                -250.0,
                vec![[900.0, -300.0], [1_100.0, -300.0], [1_100.0, -200.0]],
            ),
            marker(
                "province_east",
                2_000.0,
                750.0,
                vec![[1_900.0, 700.0], [2_100.0, 700.0], [2_100.0, 800.0]],
            ),
        ];
        let mut view = MapView::new(Rect::new(20.0, 40.0, 800.0, 500.0));

        assert!(view.fit_to_markers(&markers));
        for marker in &markers {
            let (x, y) = view.world_to_screen(marker.x, marker.z);
            assert!(view.bounds.contains(x, y));
            for point in &marker.polygon {
                let (x, y) = view.world_to_screen(point[0], point[1]);
                assert!(view.bounds.contains(x, y));
            }
        }
        let target = &markers[0];
        let (x, y) = view.world_to_screen(target.x, target.z);
        view.update_cursor(x, y, &markers);
        assert_eq!(view.hovered_province, Some(target.province.clone()));
        assert!(view.handle_click(x, y));
        assert_eq!(view.selected_province, Some(target.province.clone()));
    }

    #[test]
    fn fit_is_stable_for_pan_and_recomputes_after_resize() {
        let markers = vec![marker(
            "province",
            1_000.0,
            500.0,
            vec![[900.0, 400.0], [1_100.0, 400.0], [1_100.0, 600.0]],
        )];
        let mut view = MapView::new(Rect::new(0.0, 0.0, 800.0, 600.0));
        assert!(view.fit_to_markers(&markers));
        view.offset_x += 20.0;
        assert!(!view.fit_to_markers(&markers));
        assert_eq!(view.offset_x, 1_020.0);
        view.bounds.width = 400.0;
        assert!(view.fit_to_markers(&markers));
        assert_eq!(view.offset_x, 1_000.0);
    }

    #[test]
    fn province_edges_are_clipped_to_the_map_view() {
        let bounds = Rect::new(10.0, 20.0, 100.0, 80.0);
        let (start, end) = clip_segment(bounds, (0.0, 60.0), (150.0, 60.0)).unwrap();
        assert!((start.0 - 10.0).abs() < 1e-4 && (start.1 - 60.0).abs() < 1e-4);
        assert!((end.0 - 110.0).abs() < 1e-4 && (end.1 - 60.0).abs() < 1e-4);
        assert!(clip_segment(bounds, (0.0, 0.0), (5.0, 5.0)).is_none());
    }

    #[test]
    fn province_focus_is_visible_and_can_be_selected() {
        let markers = vec![marker("province", 10_000.0, -8_000.0, Vec::new())];
        let mut view = MapView::new(Rect::new(0.0, 0.0, 800.0, 600.0));
        view.focused_province = Some(markers[0].province.clone());
        view.zoom = 2.0;
        view.offset_x = -10_000.0;
        view.keep_focused_visible(&markers);
        let (x, y) = view.world_to_screen(markers[0].x, markers[0].z);
        assert!(view.bounds.contains(x, y));
        assert!(view.handle_key("enter", &markers));
        assert_eq!(view.selected_province, Some(markers[0].province.clone()));
    }

    #[test]
    fn tab_cycles_province_focus() {
        let markers = vec![
            marker("province_a", 10.0, 20.0, Vec::new()),
            marker("province_b", 30.0, 40.0, Vec::new()),
        ];
        let mut view = MapView::new(Rect::new(0.0, 0.0, 800.0, 600.0));

        assert!(view.handle_key("tab", &markers));
        assert_eq!(view.focused_province, Some(markers[0].province.clone()));
        assert!(view.handle_key("tab", &markers));
        assert_eq!(view.focused_province, Some(markers[1].province.clone()));
    }

    #[test]
    fn render_draws_province_edges_and_a_visible_focus_outline() {
        let markers = vec![marker(
            "province",
            50.0,
            50.0,
            vec![[20.0, 20.0], [80.0, 20.0], [80.0, 80.0]],
        )];
        let mut view = MapView::new(Rect::new(0.0, 0.0, 200.0, 200.0));
        view.focused_province = Some(markers[0].province.clone());
        let mut commands = Vec::new();
        view.render(&ui_model(markers), &mut commands);

        assert!(commands
            .iter()
            .any(|command| matches!(command, DrawCommand::Line { .. })));
        assert!(commands.iter().any(|command| matches!(
            command,
            DrawCommand::StrokeRect { color, .. } if *color == colors::FOCUS
        )));
    }
}
