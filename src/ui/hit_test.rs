use crate::ui::rect::Rect;

/// Opaque identifier for a clickable region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HitId(pub u32);

/// Region entry stored during paint.
struct HitRegion {
    id: HitId,
    bounds: Rect,
}

/// Collects clickable regions during paint and resolves clicks to the topmost hit.
pub struct HitSink {
    regions: Vec<HitRegion>,
}

impl Default for HitSink {
    fn default() -> Self {
        Self::new()
    }
}

impl HitSink {
    /// Creates an empty sink.
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
        }
    }

    /// Removes all registered regions (call at the start of each frame).
    pub fn clear(&mut self) {
        self.regions.clear();
    }

    /// Registers a clickable region — later pushes win on overlap.
    pub fn push(&mut self, id: HitId, bounds: Rect) {
        self.regions.push(HitRegion { id, bounds });
    }

    /// Returns the topmost hit (last-pushed) region containing the point.
    pub fn test(&self, point_x: f32, point_y: f32) -> Option<HitId> {
        self.regions
            .iter()
            .rev()
            .find(|region| region.bounds.contains(point_x, point_y))
            .map(|region| region.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_returns_none_when_empty() {
        let sink = HitSink::new();
        assert_eq!(sink.test(50.0, 50.0), None);
    }

    #[test]
    fn test_returns_matching_region() {
        let mut sink = HitSink::new();
        sink.push(HitId(1), Rect::new(0.0, 0.0, 100.0, 100.0));
        assert_eq!(sink.test(50.0, 50.0), Some(HitId(1)));
    }

    #[test]
    fn test_returns_none_outside() {
        let mut sink = HitSink::new();
        sink.push(HitId(1), Rect::new(0.0, 0.0, 100.0, 100.0));
        assert_eq!(sink.test(150.0, 50.0), None);
    }

    #[test]
    fn last_pushed_wins_on_overlap() {
        let mut sink = HitSink::new();
        sink.push(HitId(1), Rect::new(0.0, 0.0, 100.0, 100.0));
        sink.push(HitId(2), Rect::new(0.0, 0.0, 100.0, 100.0));
        assert_eq!(sink.test(50.0, 50.0), Some(HitId(2)));
    }

    #[test]
    fn clear_removes_all_regions() {
        let mut sink = HitSink::new();
        sink.push(HitId(1), Rect::new(0.0, 0.0, 100.0, 100.0));
        sink.clear();
        assert_eq!(sink.test(50.0, 50.0), None);
    }
}
