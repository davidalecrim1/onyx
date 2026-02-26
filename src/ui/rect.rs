use vello::kurbo::RoundedRect;

/// Axis-aligned rectangle in logical pixels.
#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Result of splitting a rect into two halves.
pub struct SplitPair {
    pub left: Rect,
    pub right: Rect,
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

    /// Centers a child of `child_width x child_height` inside this rect.
    pub fn center_child(&self, child_width: f32, child_height: f32) -> Self {
        Self {
            x: self.x + (self.width - child_width) / 2.0,
            y: self.y + (self.height - child_height) / 2.0,
            width: child_width,
            height: child_height,
        }
    }

    /// Splits vertically at `left_width` pixels from the left edge.
    pub fn split_vertical(&self, left_width: f32) -> SplitPair {
        SplitPair {
            left: Rect::new(self.x, self.y, left_width, self.height),
            right: Rect::new(
                self.x + left_width,
                self.y,
                (self.width - left_width).max(0.0),
                self.height,
            ),
        }
    }

    /// Splits horizontally at `top_height` pixels from the top edge.
    pub fn split_horizontal(&self, top_height: f32) -> SplitPair {
        SplitPair {
            left: Rect::new(self.x, self.y, self.width, top_height),
            right: Rect::new(
                self.x,
                self.y + top_height,
                self.width,
                (self.height - top_height).max(0.0),
            ),
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
    fn center_child_is_centered() {
        let parent = Rect::new(0.0, 0.0, 200.0, 100.0);
        let child = parent.center_child(50.0, 30.0);
        assert_eq!(child.x, 75.0);
        assert_eq!(child.y, 35.0);
        assert_eq!(child.width, 50.0);
        assert_eq!(child.height, 30.0);
    }

    #[test]
    fn split_vertical_sums_to_parent() {
        let parent = Rect::new(10.0, 20.0, 300.0, 100.0);
        let split = parent.split_vertical(80.0);

        assert_eq!(split.left.x, 10.0);
        assert_eq!(split.left.width, 80.0);
        assert_eq!(split.right.x, 90.0);
        assert_eq!(split.right.width, 220.0);
        assert_eq!(split.left.height, split.right.height);
    }

    #[test]
    fn split_horizontal_sums_to_parent() {
        let parent = Rect::new(0.0, 0.0, 200.0, 300.0);
        let split = parent.split_horizontal(100.0);

        assert_eq!(split.left.y, 0.0);
        assert_eq!(split.left.height, 100.0);
        assert_eq!(split.right.y, 100.0);
        assert_eq!(split.right.height, 200.0);
    }
}
