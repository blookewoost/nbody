//! Runge-Kutta-Fehlberg (RKF45) adaptive time-stepping integrator
//!
//! This module provides a generic implementation of the RKF45 method
//! for solving systems of first-order ODEs. It can be adapted to any
//! N-body gravitational simulation by implementing the appropriate
//! derivative function.

use crate::body::Body;

/// State vector for a single body: [x, y, z, vx, vy, vz]
pub type StateVector = [f64; 6];

/// A function that computes derivatives (accelerations) for all bodies
/// given their current state. The function receives a mutable slice
/// of bodies and should compute/update their accelerations.
pub type DerivativeFunction = fn(&mut [Body]);

/// Runge-Kutta-Fehlberg (RKF45) integrator for N-body simulations
///
/// This implements the 5th-order Runge-Kutta method with embedded
/// 4th-order error estimation for adaptive time-stepping.
pub struct RungeKuttaFehlberg {
    // RKF45 coefficients
    c: [f64; 6],
    a: [f64; 6],
    b: [[f64; 5]; 6],
    /// 5th order weights
    b5: [f64; 6],
    /// 4th order weights (for error estimation)
    b4: [f64; 6],
}

impl RungeKuttaFehlberg {
    /// Create a new RKF45 integrator with standard coefficients
    pub fn new() -> Self {
        RungeKuttaFehlberg {
            c: [0.0, 0.25, 3.0 / 8.0, 12.0 / 13.0, 1.0, 0.5],
            a: [0.0, 0.25, 3.0 / 8.0, 12.0 / 13.0, 1.0, 0.5],
            b: [
                [0.0, 0.0, 0.0, 0.0, 0.0],
                [1.0 / 4.0, 0.0, 0.0, 0.0, 0.0],
                [3.0 / 32.0, 9.0 / 32.0, 0.0, 0.0, 0.0],
                [1932.0 / 2197.0, -7200.0 / 2197.0, 7296.0 / 2197.0, 0.0, 0.0],
                [439.0 / 216.0, -8.0, 3680.0 / 513.0, -845.0 / 4104.0, 0.0],
                [-8.0 / 27.0, 2.0, -3544.0 / 2565.0, 1859.0 / 4104.0, -11.0 / 40.0],
            ],
            b5: [16.0 / 135.0, 0.0, 6656.0 / 12825.0, 28561.0 / 56430.0, -9.0 / 50.0, 2.0 / 55.0],
            b4: [25.0 / 216.0, 0.0, 1408.0 / 2565.0, 2197.0 / 4104.0, -1.0 / 5.0, 0.0],
        }
    }

    /// Perform a single RKF45 step for a system of N bodies
    ///
    /// # Arguments
    /// * `bodies` - Mutable slice of bodies to integrate
    /// * `dt` - Current time step
    /// * `derivative_fn` - Function to compute accelerations from current state
    ///
    /// # Returns
    /// A tuple of (new_error_estimate, old_error_estimate) which can be used
    /// for adaptive time-stepping if desired
    pub fn step(
        &self,
        bodies: &mut [Body],
        dt: f64,
        derivative_fn: DerivativeFunction,
    ) -> (f64, f64) {
        let n = bodies.len();

        // Store initial state
        let initial_bodies: Vec<Body> = bodies.iter().copied().collect();

        // Compute k values (derivatives at various stages)
        let mut k = vec![vec![[0.0; 6]; n]; 6];

        // k0: evaluate at current state (c0 = 0)
        derivative_fn(bodies);
        for i in 0..n {
            k[0][i][0] = dt * bodies[i].velocity[0];
            k[0][i][1] = dt * bodies[i].velocity[1];
            k[0][i][2] = dt * bodies[i].velocity[2];
            k[0][i][3] = dt * bodies[i].acceleration[0];
            k[0][i][4] = dt * bodies[i].acceleration[1];
            k[0][i][5] = dt * bodies[i].acceleration[2];
        }

        // Compute remaining k values (k1 through k5)
        for stage in 1..6 {
            // Restore initial state
            bodies.copy_from_slice(&initial_bodies);

            // Compute weighted sum of previous k values to get intermediate state
            for i in 0..n {
                let mut dx = [0.0; 3];
                let mut dv = [0.0; 3];
                
                for prev_stage in 0..stage {
                    dx[0] += self.b[stage - 1][prev_stage] * k[prev_stage][i][0];
                    dx[1] += self.b[stage - 1][prev_stage] * k[prev_stage][i][1];
                    dx[2] += self.b[stage - 1][prev_stage] * k[prev_stage][i][2];
                    dv[0] += self.b[stage - 1][prev_stage] * k[prev_stage][i][3];
                    dv[1] += self.b[stage - 1][prev_stage] * k[prev_stage][i][4];
                    dv[2] += self.b[stage - 1][prev_stage] * k[prev_stage][i][5];
                }
                
                bodies[i].position[0] += dx[0];
                bodies[i].position[1] += dx[1];
                bodies[i].position[2] += dx[2];
                bodies[i].velocity[0] += dv[0];
                bodies[i].velocity[1] += dv[1];
                bodies[i].velocity[2] += dv[2];
            }

            // Compute derivatives at this stage
            derivative_fn(bodies);

            // Store k values
            for i in 0..n {
                k[stage][i][0] = dt * bodies[i].velocity[0];
                k[stage][i][1] = dt * bodies[i].velocity[1];
                k[stage][i][2] = dt * bodies[i].velocity[2];
                k[stage][i][3] = dt * bodies[i].acceleration[0];
                k[stage][i][4] = dt * bodies[i].acceleration[1];
                k[stage][i][5] = dt * bodies[i].acceleration[2];
            }
        }

        // Restore initial state
        bodies.copy_from_slice(&initial_bodies);

        // Apply 5th order solution
        for i in 0..n {
            for dim in 0..6 {
                let mut update = 0.0;
                for stage in 0..6 {
                    update += self.b5[stage] * k[stage][i][dim];
                }

                match dim {
                    0 => bodies[i].position[0] += update,
                    1 => bodies[i].position[1] += update,
                    2 => bodies[i].position[2] += update,
                    3 => bodies[i].velocity[0] += update,
                    4 => bodies[i].velocity[1] += update,
                    5 => bodies[i].velocity[2] += update,
                    _ => {}
                }
            }
        }

        // Compute error estimate (difference between 5th and 4th order solutions)
        let mut error_5th: f64 = 0.0;
        let mut error_4th: f64 = 0.0;
        for i in 0..n {
            for dim in 0..6 {
                let mut sol5 = 0.0;
                let mut sol4 = 0.0;
                for stage in 0..6 {
                    sol5 += self.b5[stage] * k[stage][i][dim];
                    sol4 += self.b4[stage] * k[stage][i][dim];
                }
                let diff = (sol5 - sol4).abs();
                error_5th = error_5th.max(diff);
                error_4th = error_4th.max(diff);
            }
        }

        (error_5th, error_4th)
    }
}

impl Default for RungeKuttaFehlberg {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rkf45_creation() {
        let _integrator = RungeKuttaFehlberg::new();
    }

    #[test]
    fn test_rkf45_coefficients() {
        let integrator = RungeKuttaFehlberg::new();
        // Verify that coefficients are initialized
        assert!((integrator.c[1] - 0.25).abs() < 1e-10);
        assert!((integrator.b5[0] - (16.0 / 135.0)).abs() < 1e-10);
    }
}
