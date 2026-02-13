//! Physical body representation for N-body simulation

use std::fmt;

/// A physical body with mass, position, and velocity in 3D space
#[derive(Debug, Clone, Copy)]
pub struct Body {
    /// Mass of the body (in kg)
    pub mass: f64,
    /// Position vector [x, y, z]
    pub position: [f64; 3],
    /// Velocity vector [vx, vy, vz]
    pub velocity: [f64; 3],
    /// Acceleration vector [ax, ay, az] (computed during integration)
    pub acceleration: [f64; 3],
}

impl Body {
    /// Create a new body with the given properties
    pub fn new(mass: f64, position: [f64; 3], velocity: [f64; 3]) -> Self {
        Body {
            mass,
            position,
            velocity,
            acceleration: [0.0; 3],
        }
    }

    /// Calculate the distance to another body
    pub fn distance_to(&self, other: &Body) -> f64 {
        let dx = other.position[0] - self.position[0];
        let dy = other.position[1] - self.position[1];
        let dz = other.position[2] - self.position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Calculate the vector from this body to another
    pub fn vector_to(&self, other: &Body) -> [f64; 3] {
        [
            other.position[0] - self.position[0],
            other.position[1] - self.position[1],
            other.position[2] - self.position[2],
        ]
    }

    /// Set acceleration to zero
    pub fn reset_acceleration(&mut self) {
        self.acceleration = [0.0; 3];
    }

    /// Add to the current acceleration
    pub fn add_acceleration(&mut self, acc: [f64; 3]) {
        self.acceleration[0] += acc[0];
        self.acceleration[1] += acc[1];
        self.acceleration[2] += acc[2];
    }
}

impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Body {{ mass: {:.2e}, pos: [{:.2e}, {:.2e}, {:.2e}], vel: [{:.2e}, {:.2e}, {:.2e}] }}",
            self.mass,
            self.position[0], self.position[1], self.position[2],
            self.velocity[0], self.velocity[1], self.velocity[2]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_body_creation() {
        let body = Body::new(1e30, [1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
        assert_eq!(body.mass, 1e30);
        assert_eq!(body.position, [1.0, 2.0, 3.0]);
        assert_eq!(body.velocity, [4.0, 5.0, 6.0]);
        assert_eq!(body.acceleration, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_distance_calculation() {
        let body1 = Body::new(1e30, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]);
        let body2 = Body::new(1e30, [3.0, 4.0, 0.0], [0.0, 0.0, 0.0]);
        assert!((body1.distance_to(&body2) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_vector_to() {
        let body1 = Body::new(1e30, [1.0, 2.0, 3.0], [0.0, 0.0, 0.0]);
        let body2 = Body::new(1e30, [4.0, 6.0, 8.0], [0.0, 0.0, 0.0]);
        let vec = body1.vector_to(&body2);
        assert_eq!(vec, [3.0, 4.0, 5.0]);
    }
}
