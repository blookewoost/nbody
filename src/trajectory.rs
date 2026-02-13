//! Trajectory data loading and management
//!
//! Parses CSV files from the N-body simulator and stores trajectory data

use std::fs::File;
use std::path::Path;

/// A single position sample for one body at one time step
#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Position {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Position {
            x: x as f32,
            y: y as f32,
            z: z as f32,
        }
    }
}

/// Complete trajectory for a single body
#[derive(Debug, Clone)]
pub struct BodyTrajectory {
    pub positions: Vec<Position>,
}

impl BodyTrajectory {
    pub fn new() -> Self {
        BodyTrajectory {
            positions: Vec::new(),
        }
    }

    pub fn add_position(&mut self, pos: Position) {
        self.positions.push(pos);
    }

    pub fn get_position(&self, frame: usize) -> Option<Position> {
        self.positions.get(frame).copied()
    }

    pub fn len(&self) -> usize {
        self.positions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.positions.is_empty()
    }
}

/// Complete trajectory data for all bodies in a simulation
#[derive(Debug, Clone)]
pub struct TrajectoryData {
    pub bodies: Vec<BodyTrajectory>,
    pub num_frames: usize,
}

impl TrajectoryData {
    pub fn new() -> Self {
        TrajectoryData {
            bodies: Vec::new(),
            num_frames: 0,
        }
    }

    pub fn load_csv<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let file = File::open(path)?;
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_reader(file);

        let mut data = TrajectoryData::new();
        let mut first_row = true;

        for result in reader.records() {
            let record = result.map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
            })?;

            // Initialize body trajectories on first row based on column count
            if first_row {
                // Format: time, body0_x, body0_y, body0_z, body1_x, body1_y, body1_z, ...
                // Number of bodies = (num_fields - 1) / 3
                let num_fields = record.len();
                if num_fields < 4 {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "CSV must have at least time and one body (4 columns)",
                    ));
                }

                let num_bodies = (num_fields - 1) / 3;
                for _ in 0..num_bodies {
                    data.bodies.push(BodyTrajectory::new());
                }
                first_row = false;
            }

            // Parse time and positions
            let mut fields = record.iter();
            let _time: f64 = fields
                .next()
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing time field")
                })?
                .parse()
                .map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid time value")
                })?;

            // Parse body positions
            for body in &mut data.bodies {
                let x: f64 = fields
                    .next()
                    .ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing x field")
                    })?
                    .parse()
                    .map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid x value")
                    })?;

                let y: f64 = fields
                    .next()
                    .ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing y field")
                    })?
                    .parse()
                    .map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid y value")
                    })?;

                let z: f64 = fields
                    .next()
                    .ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing z field")
                    })?
                    .parse()
                    .map_err(|_| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid z value")
                    })?;

                body.add_position(Position::new(x, y, z));
            }

            data.num_frames += 1;
        }

        if data.bodies.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "No bodies found in trajectory data",
            ));
        }

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_creation() {
        let pos = Position::new(1.0, 2.0, 3.0);
        assert_eq!(pos.x, 1.0);
        assert_eq!(pos.y, 2.0);
        assert_eq!(pos.z, 3.0);
    }

    #[test]
    fn test_body_trajectory() {
        let mut traj = BodyTrajectory::new();
        assert!(traj.is_empty());

        traj.add_position(Position::new(0.0, 0.0, 0.0));
        traj.add_position(Position::new(1.0, 2.0, 3.0));

        assert_eq!(traj.len(), 2);
        assert_eq!(traj.get_position(0).unwrap().x, 0.0);
        assert_eq!(traj.get_position(1).unwrap().y, 2.0);
    }
}
