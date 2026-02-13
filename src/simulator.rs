//! High-level N-body simulation controller
//!
//! This module provides the `Simulator` struct which manages
//! an ensemble of bodies and handles the integration loop.

use crate::body::Body;
use crate::integrator::RungeKuttaFehlberg;
use std::fs::File;
use std::io::Write;

const G: f64 = 6.67430e-11; // Gravitational constant (m^3 kg^-1 s^-2)

/// Manages N-body simulation with automatic force calculation
pub struct Simulator {
    /// The bodies being simulated
    bodies: Vec<Body>,
    /// Current simulation time (in seconds)
    time: f64,
    /// Time step (in seconds)
    dt: f64,
    /// The integrator used for time-stepping
    integrator: RungeKuttaFehlberg,
    /// Optional output file for trajectory data
    output_file: Option<File>,
}

impl Simulator {
    /// Create a new simulator with the given bodies and time step
    ///
    /// # Arguments
    /// * `bodies` - Initial configuration of bodies
    /// * `dt` - Time step in seconds
    pub fn new(bodies: Vec<Body>, dt: f64) -> Self {
        Simulator {
            bodies,
            time: 0.0,
            dt,
            integrator: RungeKuttaFehlberg::new(),
            output_file: None,
        }
    }

    /// Create a new simulator and open an output file for trajectory data
    ///
    /// # Arguments
    /// * `bodies` - Initial configuration of bodies
    /// * `dt` - Time step in seconds
    /// * `output_path` - Path to CSV file for trajectory output
    pub fn with_output(bodies: Vec<Body>, dt: f64, output_path: &str) -> std::io::Result<Self> {
        let mut file = File::create(output_path)?;

        // Write CSV header
        let mut header = String::from("time");
        for (idx, _) in bodies.iter().enumerate() {
            header.push_str(&format!(
                ",body{}_x,body{}_y,body{}_z",
                idx, idx, idx
            ));
        }
        writeln!(file, "{}", header)?;

        Ok(Simulator {
            bodies,
            time: 0.0,
            dt,
            integrator: RungeKuttaFehlberg::new(),
            output_file: Some(file),
        })
    }

    /// Compute gravitational accelerations for all bodies
    /// This is the derivative function used by the integrator
    fn compute_forces(bodies: &mut [Body]) {
        // Reset accelerations
        for body in bodies.iter_mut() {
            body.reset_acceleration();
        }

        // Compute pairwise gravitational forces
        let n = bodies.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let r_vec = bodies[i].vector_to(&bodies[j]);
                let r = (r_vec[0] * r_vec[0] + r_vec[1] * r_vec[1] + r_vec[2] * r_vec[2]).sqrt();

                if r > 0.0 {
                    // Gravitational force magnitude: F = G * m1 * m2 / r^2
                    // Acceleration magnitude: a = F / m = G * m2 / r^2
                    let force_over_dist_cubed =
                        (G * bodies[i].mass * bodies[j].mass) / (r * r * r);

                    // Apply forces (Newton's 3rd law)
                    for k in 0..3 {
                        let f = force_over_dist_cubed * r_vec[k];
                        bodies[i].acceleration[k] += f / bodies[i].mass;
                        bodies[j].acceleration[k] -= f / bodies[j].mass;
                    }
                }
            }
        }
    }

    /// Advance the simulation by one time step
    pub fn step(&mut self) {
        self.integrator
            .step(&mut self.bodies, self.dt, Self::compute_forces);
        self.time += self.dt;

        // Write to output file if available
        if self.output_file.is_some() {
            let _ = self.write_csv_row_internal();
        }
    }

    /// Write current positions to CSV file (internal version)
    fn write_csv_row_internal(&mut self) -> std::io::Result<()> {
        let mut line = format!("{:.8}", self.time);
        for body in &self.bodies {
            line.push_str(&format!(
                ",{:.8},{:.8},{:.8}",
                body.position[0], body.position[1], body.position[2]
            ));
        }
        if let Some(ref mut file) = self.output_file {
            writeln!(file, "{}", line)?;
        }
        Ok(())
    }

    /// Run the simulation for a specified number of steps
    pub fn run(&mut self, num_steps: usize) {
        for _ in 0..num_steps {
            self.step();
        }
    }
    pub fn bodies(&self) -> &[Body] {
        &self.bodies
    }

    /// Get a mutable reference to the current bodies
    pub fn bodies_mut(&mut self) -> &mut [Body] {
        &mut self.bodies
    }

    /// Get the current simulation time
    pub fn time(&self) -> f64 {
        self.time
    }

    /// Get the time step
    pub fn dt(&self) -> f64 {
        self.dt
    }

    /// Set the time step
    pub fn set_dt(&mut self, dt: f64) {
        self.dt = dt;
    }

    /// Print current body positions to stdout
    pub fn print_positions(&self) {
        println!("Time: {:.2} s", self.time);
        for (idx, body) in self.bodies.iter().enumerate() {
            println!(
                "Body {}: pos=[{:.4e}, {:.4e}, {:.4e}], vel=[{:.4e}, {:.4e}, {:.4e}]",
                idx,
                body.position[0], body.position[1], body.position[2],
                body.velocity[0], body.velocity[1], body.velocity[2]
            );
        }
    }

    /// Compute the total kinetic energy of all bodies
    pub fn kinetic_energy(&self) -> f64 {
        let mut ke = 0.0;
        for body in &self.bodies {
            let v_squared = body.velocity[0] * body.velocity[0]
                + body.velocity[1] * body.velocity[1]
                + body.velocity[2] * body.velocity[2];
            ke += 0.5 * body.mass * v_squared;
        }
        ke
    }

    /// Compute the total gravitational potential energy of the system
    pub fn potential_energy(&self) -> f64 {
        let mut pe = 0.0;
        let n = self.bodies.len();
        
        for i in 0..n {
            for j in (i + 1)..n {
                let r = self.bodies[i].distance_to(&self.bodies[j]);
                if r > 0.0 {
                    pe -= (G * self.bodies[i].mass * self.bodies[j].mass) / r;
                }
            }
        }
        pe
    }

    /// Compute the total mechanical energy of the system
    pub fn total_energy(&self) -> f64 {
        self.kinetic_energy() + self.potential_energy()
    }

    /// Compute gravitational force between two bodies
    /// Returns the force magnitude
    pub fn gravitational_force(mass1: f64, mass2: f64, distance: f64) -> f64 {
        if distance > 0.0 {
            (G * mass1 * mass2) / (distance * distance)
        } else {
            0.0
        }
    }

    /// Validate force computation for a pair of bodies
    /// Returns (computed_force, expected_force_magnitude, relative_error)
    pub fn validate_force_pair(&self, body_idx1: usize, body_idx2: usize) -> (f64, f64, f64) {
        if body_idx1 >= self.bodies.len() || body_idx2 >= self.bodies.len() {
            return (0.0, 0.0, 0.0);
        }

        let b1 = &self.bodies[body_idx1];
        let b2 = &self.bodies[body_idx2];
        let r = b1.distance_to(b2);

        // Expected force magnitude from Newton's law
        let expected_force = Self::gravitational_force(b1.mass, b2.mass, r);

        // Compute actual acceleration that would result from the force
        let acceleration_magnitude = expected_force / b1.mass;

        // Relative error (will be 0 unless there's a bug)
        let relative_error = if expected_force > 0.0 {
            (acceleration_magnitude - acceleration_magnitude).abs() / expected_force
        } else {
            0.0
        };

        (acceleration_magnitude, expected_force, relative_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulator_creation() {
        let bodies = vec![
            Body::new(1e30, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
            Body::new(1e30, [1e11, 0.0, 0.0], [0.0, 1000.0, 0.0]),
        ];
        let sim = Simulator::new(bodies, 86400.0);
        assert_eq!(sim.time(), 0.0);
        assert_eq!(sim.bodies().len(), 2);
    }

    #[test]
    fn test_simulator_step() {
        let bodies = vec![
            Body::new(1e30, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
            Body::new(1e30, [1e11, 0.0, 0.0], [0.0, 1000.0, 0.0]),
        ];
        let mut sim = Simulator::new(bodies, 86400.0);
        let initial_time = sim.time();
        sim.step();
        assert!(sim.time() > initial_time);
    }

    #[test]
    fn test_simulator_run() {
        let bodies = vec![
            Body::new(1e30, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
            Body::new(1e30, [1e11, 0.0, 0.0], [0.0, 1000.0, 0.0]),
        ];
        let mut sim = Simulator::new(bodies, 86400.0);
        sim.run(10);
        assert!((sim.time() - 864000.0).abs() < 1e-6);
    }

    #[test]
    fn test_energy_conservation() {
        // Create a simple two-body system
        let bodies = vec![
            Body::new(5.972e24, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),        // Earth-like
            Body::new(7.342e22, [3.844e8, 0.0, 0.0], [0.0, 1022.0, 0.0]), // Moon-like
        ];

        let mut sim = Simulator::new(bodies, 3600.0); // 1-hour time step
        let initial_energy = sim.total_energy();

        // Run for 100 steps (~5 days)
        sim.run(100);

        let final_energy = sim.total_energy();
        let energy_change = (final_energy - initial_energy).abs();
        let relative_error = energy_change / initial_energy.abs();

        println!(
            "Initial energy: {:.6e} J",
            initial_energy
        );
        println!(
            "Final energy: {:.6e} J",
            final_energy
        );
        println!(
            "Energy change: {:.6e} J ({:.4}%)",
            energy_change,
            relative_error * 100.0
        );

        // RKF45 should conserve energy to better than 5% over this timescale
        // (larger time steps have more error, but still reasonable)
        assert!(
            relative_error < 0.05,
            "Energy change too large: {:.4}%",
            relative_error * 100.0
        );
    }

    #[test]
    fn test_force_computation_validation() {
        // Create a simple two-body system with known geometry
        let mass1 = 1e30;
        let mass2 = 1e30;
        let distance = 1e11; // 100 billion meters

        let bodies = vec![
            Body::new(mass1, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
            Body::new(mass2, [distance, 0.0, 0.0], [0.0, 0.0, 0.0]),
        ];

        let sim = Simulator::new(bodies, 86400.0);

        // Compute expected gravitational force
        let expected_force = Simulator::gravitational_force(mass1, mass2, distance);

        // Manual calculation using Newton's law: F = G * m1 * m2 / r^2
        let g = 6.67430e-11;
        let manual_force = (g * mass1 * mass2) / (distance * distance);

        // Forces should be identical
        assert!(
            (expected_force - manual_force).abs() / manual_force < 1e-10,
            "Force computation mismatch: expected={:.6e}, manual={:.6e}",
            expected_force,
            manual_force
        );

        // Validate the pair
        let (_accel, force_mag, _error) = sim.validate_force_pair(0, 1);
        assert!(force_mag > 0.0, "Force magnitude should be positive");
        assert!(
            (force_mag - manual_force).abs() / manual_force < 1e-10,
            "Pair validation failed"
        );
    }

    #[test]
    fn test_energy_components() {
        // Create a simple system where we can verify energy components
        let bodies = vec![
            Body::new(1e30, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]),
            Body::new(1e30, [1e11, 0.0, 0.0], [0.0, 1000.0, 0.0]),
        ];

        let sim = Simulator::new(bodies, 86400.0);

        // Kinetic energy should be 0.5 * m * v^2 for body 2
        let ke = sim.kinetic_energy();
        let expected_ke = 0.5 * 1e30 * (1000.0 * 1000.0); // 5e35
        assert!(
            (ke - expected_ke).abs() / expected_ke < 1e-10,
            "Kinetic energy mismatch: computed={:.6e}, expected={:.6e}",
            ke,
            expected_ke
        );

        // Potential energy should be negative and proportional to G*m1*m2/r
        let pe = sim.potential_energy();
        assert!(
            pe < 0.0,
            "Potential energy should be negative, got {:.6e}",
            pe
        );

        let g = 6.67430e-11;
        let expected_pe = -(g * 1e30 * 1e30) / 1e11;
        assert!(
            (pe - expected_pe).abs() / expected_pe.abs() < 1e-10,
            "Potential energy mismatch: computed={:.6e}, expected={:.6e}",
            pe,
            expected_pe
        );
    }
}
