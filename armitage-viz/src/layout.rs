//! Layout algorithms for network visualization

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

pub trait Layout {
    fn compute(&self, node_count: usize) -> Vec<Position>;
}

#[derive(Debug, Clone)]
pub struct CircleLayout {
    pub radius: f64,
}

impl CircleLayout {
    pub fn new(radius: f64) -> Self {
        Self { radius }
    }
}

impl Layout for CircleLayout {
    fn compute(&self, node_count: usize) -> Vec<Position> {
        if node_count == 0 {
            return Vec::new();
        }

        let mut positions = Vec::with_capacity(node_count);
        let angle_step = 2.0 * std::f64::consts::PI / node_count as f64;

        for i in 0..node_count {
            let angle = angle_step * i as f64;
            let x = self.radius * angle.cos();
            let y = self.radius * angle.sin();
            positions.push(Position { x, y });
        }

        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_layout() {
        let layout = CircleLayout::new(100.0);
        let positions = layout.compute(4);

        assert_eq!(positions.len(), 4);
        // First node should be on the right
        assert!((positions[0].x - 100.0).abs() < 0.1);
        assert!(positions[0].y.abs() < 0.1);
    }
}
