//! Composants et primitives d'interface abstraits.
//!
//! Ce module sépare la logique d'agencement et d'interaction des appels
//! directs au moteur de rendu Horizon. Les écrans émettent des `DrawCommand`.

use super::theme::{colors, ColorRgba};

/// Rectangle 2D pour le layout et les clics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.width && py >= self.y && py <= self.y + self.height
    }

    pub fn shrink(&self, pad: f32) -> Self {
        Self {
            x: self.x + pad,
            y: self.y + pad,
            width: (self.width - pad * 2.0).max(0.0),
            height: (self.height - pad * 2.0).max(0.0),
        }
    }
}

/// Primitive graphique émise par les widgets.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawCommand {
    /// Rectangle plein.
    FillRect { rect: Rect, color: ColorRgba },
    /// Contour de rectangle.
    StrokeRect {
        rect: Rect,
        color: ColorRgba,
        thickness: f32,
    },
    /// Ligne 2D.
    Line {
        start: (f32, f32),
        end: (f32, f32),
        color: ColorRgba,
        thickness: f32,
    },
    /// Texte.
    Text {
        x: f32,
        y: f32,
        text: String,
        color: ColorRgba,
        scale: f32,
    },
}

/// Bouton interactif.
#[derive(Clone, Debug)]
pub struct Button {
    pub id: String,
    pub label: String,
    pub rect: Rect,
    pub disabled: bool,
    pub hovered: bool,
    pub focused: bool,
}

impl Button {
    pub fn new(id: impl Into<String>, label: impl Into<String>, rect: Rect) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            rect,
            disabled: false,
            hovered: false,
            focused: false,
        }
    }

    pub fn handle_cursor(&mut self, cx: f32, cy: f32) {
        self.hovered = !self.disabled && self.rect.contains(cx, cy);
    }

    pub fn is_clicked(&self, cx: f32, cy: f32) -> bool {
        !self.disabled && self.rect.contains(cx, cy)
    }

    pub fn activate_key(&self, key: &str) -> bool {
        self.focused && !self.disabled && matches!(key.to_ascii_lowercase().as_str(), "enter" | " ")
    }

    pub fn render(&self, out: &mut Vec<DrawCommand>) {
        let bg = if self.disabled {
            ColorRgba::new(0.15, 0.15, 0.18, 0.6)
        } else if self.hovered {
            colors::ROW_HOVER
        } else {
            colors::HEADER_BG
        };

        out.push(DrawCommand::FillRect {
            rect: self.rect,
            color: bg,
        });
        out.push(DrawCommand::StrokeRect {
            rect: self.rect,
            color: if self.focused {
                colors::FOCUS
            } else if self.hovered {
                colors::ACCENT
            } else {
                colors::PANEL_BORDER
            },
            thickness: if self.focused { 2.0 } else { 1.0 },
        });
        out.push(DrawCommand::Text {
            x: self.rect.x + 8.0,
            y: self.rect.y + 4.0,
            text: self.label.clone(),
            color: if self.disabled {
                colors::TEXT_MUTED
            } else {
                colors::TEXT_PRIMARY
            },
            scale: 1.0,
        });
    }
}

/// Sélecteur d'onglets pour l'écran principal.
#[derive(Clone, Debug)]
pub struct TabControl {
    pub tabs: Vec<String>,
    pub selected: usize,
    pub rect: Rect,
    pub focused: bool,
}

impl TabControl {
    pub fn new(tabs: Vec<String>, rect: Rect) -> Self {
        Self {
            tabs,
            selected: 0,
            rect,
            focused: false,
        }
    }

    pub fn tab_rect(&self, index: usize) -> Option<Rect> {
        if index >= self.tabs.len() || self.tabs.is_empty() {
            return None;
        }
        let tab_w = self.rect.width / self.tabs.len() as f32;
        Some(Rect::new(
            self.rect.x + index as f32 * tab_w,
            self.rect.y,
            tab_w,
            self.rect.height,
        ))
    }

    pub fn handle_click(&mut self, cx: f32, cy: f32) -> bool {
        for (i, _) in self.tabs.iter().enumerate() {
            if let Some(r) = self.tab_rect(i) {
                if r.contains(cx, cy) {
                    if self.selected != i {
                        self.selected = i;
                        return true;
                    }
                    return false;
                }
            }
        }
        false
    }

    pub fn handle_key(&mut self, key: &str) -> bool {
        if !self.focused || self.tabs.is_empty() {
            return false;
        }
        let next = match key.to_ascii_lowercase().as_str() {
            "arrowleft" | "left" => {
                if self.selected == 0 {
                    self.tabs.len() - 1
                } else {
                    self.selected - 1
                }
            }
            "arrowright" | "right" => (self.selected + 1) % self.tabs.len(),
            "home" => 0,
            "end" => self.tabs.len() - 1,
            _ => return false,
        };
        if next == self.selected {
            false
        } else {
            self.selected = next;
            true
        }
    }

    pub fn render(&self, out: &mut Vec<DrawCommand>) {
        out.push(DrawCommand::FillRect {
            rect: self.rect,
            color: colors::HEADER_BG,
        });
        if self.focused {
            out.push(DrawCommand::StrokeRect {
                rect: self.rect,
                color: colors::FOCUS,
                thickness: 2.0,
            });
        }
        for (i, name) in self.tabs.iter().enumerate() {
            if let Some(r) = self.tab_rect(i) {
                let is_sel = i == self.selected;
                if is_sel {
                    out.push(DrawCommand::FillRect {
                        rect: r,
                        color: colors::ROW_HOVER,
                    });
                    out.push(DrawCommand::Line {
                        start: (r.x, r.y + r.height - 2.0),
                        end: (r.x + r.width, r.y + r.height - 2.0),
                        color: colors::ACCENT,
                        thickness: 2.0,
                    });
                }
                out.push(DrawCommand::Text {
                    x: r.x + 12.0,
                    y: r.y + 6.0,
                    text: name.clone(),
                    color: if is_sel {
                        colors::TEXT_HIGHLIGHT
                    } else {
                        colors::TEXT_MUTED
                    },
                    scale: 1.0,
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains_and_shrink() {
        let r = Rect::new(10.0, 10.0, 100.0, 50.0);
        assert!(r.contains(15.0, 20.0));
        assert!(!r.contains(5.0, 20.0));
        assert!(!r.contains(120.0, 20.0));

        let s = r.shrink(5.0);
        assert_eq!(s, Rect::new(15.0, 15.0, 90.0, 40.0));
    }

    #[test]
    fn button_interaction() {
        let mut btn = Button::new("btn_ok", "Valider", Rect::new(10.0, 10.0, 80.0, 30.0));
        assert!(!btn.hovered);
        btn.handle_cursor(20.0, 20.0);
        assert!(btn.hovered);
        assert!(btn.is_clicked(20.0, 20.0));

        let mut commands = Vec::new();
        btn.render(&mut commands);
        assert!(!commands.is_empty());
    }

    #[test]
    fn tab_switching() {
        let mut tabs = TabControl::new(
            vec!["Carte".into(), "Nations".into(), "Guerre".into()],
            Rect::new(0.0, 0.0, 300.0, 30.0),
        );
        assert_eq!(tabs.selected, 0);
        // Clic sur l'onglet 1 (Nations : x de 100 à 200)
        let changed = tabs.handle_click(150.0, 15.0);
        assert!(changed);
        assert_eq!(tabs.selected, 1);
    }
}
