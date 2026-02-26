use vello::kurbo::RoundedRect;

/// Axis-aligned rectangle in logical pixels.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Creates a rect from origin and size.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Converts a Taffy layout node to a Rect, accumulating parent offsets.
    pub fn from_layout(layout: &taffy::Layout, parent_x: f32, parent_y: f32) -> Self {
        Self {
            x: parent_x + layout.location.x,
            y: parent_y + layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        }
    }

    /// Returns true if the point falls inside this rect.
    pub fn contains(&self, point_x: f32, point_y: f32) -> bool {
        point_x >= self.x
            && point_x <= self.x + self.width
            && point_y >= self.y
            && point_y <= self.y + self.height
    }

    /// Center point of the rect.
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Returns a new rect shrunk by `amount` on all sides.
    pub fn inset(&self, amount: f32) -> Self {
        Self {
            x: self.x + amount,
            y: self.y + amount,
            width: (self.width - 2.0 * amount).max(0.0),
            height: (self.height - 2.0 * amount).max(0.0),
        }
    }

    /// Converts to a `vello::kurbo::Rect`.
    pub fn to_kurbo(self) -> vello::kurbo::Rect {
        vello::kurbo::Rect::new(
            self.x as f64,
            self.y as f64,
            (self.x + self.width) as f64,
            (self.y + self.height) as f64,
        )
    }

    /// Converts to a rounded `kurbo` rect.
    pub fn to_rounded(self, radius: f64) -> RoundedRect {
        RoundedRect::new(
            self.x as f64,
            self.y as f64,
            (self.x + self.width) as f64,
            (self.y + self.height) as f64,
            radius,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use taffy::style_helpers::{length, zero, TaffyMaxContent};
    use taffy::{Size, Style, TaffyTree};

    #[test]
    fn contains_inside_point() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert!(rect.contains(50.0, 40.0));
    }

    #[test]
    fn contains_outside_point() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert!(!rect.contains(5.0, 40.0));
        assert!(!rect.contains(50.0, 75.0));
    }

    #[test]
    fn contains_edge_points() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert!(rect.contains(10.0, 20.0));
        assert!(rect.contains(110.0, 70.0));
    }

    #[test]
    fn from_layout_accumulates_parent_offset() {
        let mut tree: TaffyTree<()> = TaffyTree::new();
        let child = tree
            .new_leaf(Style {
                size: Size {
                    width: length(80.0),
                    height: length(40.0),
                },
                ..Default::default()
            })
            .unwrap();
        let root = tree
            .new_with_children(
                Style {
                    padding: taffy::Rect {
                        left: length(10.0),
                        top: length(20.0),
                        right: zero(),
                        bottom: zero(),
                    },
                    size: Size {
                        width: length(200.0),
                        height: length(100.0),
                    },
                    ..Default::default()
                },
                &[child],
            )
            .unwrap();

        tree.compute_layout(root, Size::MAX_CONTENT).unwrap();

        let root_layout = tree.layout(root).unwrap();
        let root_rect = Rect::from_layout(root_layout, 5.0, 3.0);
        assert_eq!(root_rect.x, 5.0);
        assert_eq!(root_rect.y, 3.0);
        assert_eq!(root_rect.width, 200.0);
        assert_eq!(root_rect.height, 100.0);

        let child_layout = tree.layout(child).unwrap();
        let child_rect = Rect::from_layout(child_layout, root_rect.x, root_rect.y);
        assert_eq!(child_rect.x, 15.0);
        assert_eq!(child_rect.y, 23.0);
        assert_eq!(child_rect.width, 80.0);
        assert_eq!(child_rect.height, 40.0);
    }
}
