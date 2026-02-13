//! N-body physics simulator using Runge-Kutta-Fehlberg integration
//!
//! This library provides generic functions for simulating N-body gravitational dynamics
//! using the Runge-Kutta-Fehlberg (RKF45) method for adaptive time-stepping.

pub mod integrator;
pub mod body;
pub mod simulator;
pub mod config;

pub use integrator::RungeKuttaFehlberg;
pub use body::Body;
pub use simulator::Simulator;
pub use config::{SimulationConfig, parse_ini_file};
