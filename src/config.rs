//! Configuration and initial conditions file parsing
//!
//! Handles parsing INI-format initial condition files for N-body simulations.

use crate::body::Body;
use std::fs;

/// Configuration for a simulation run
#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub bodies: Vec<Body>,
    pub time_step: f64,
    pub num_steps: usize,
    pub output_file: String,
}

/// Parse an INI file and extract body initial conditions
///
/// Expected format (as shown in this text example, not valid Rust):
/// ```text
/// [Body1]
/// mass = 4e29
/// position_x = 0
/// position_y = 1e11
/// position_z = -1e11
/// velocity_x = -600
/// velocity_y = 0
/// velocity_z = 2600
/// ```
pub fn parse_ini_file(path: &str) -> std::io::Result<SimulationConfig> {
    let content = fs::read_to_string(path)?;
    parse_ini_content(&content)
}

/// Parse INI content from a string
fn parse_ini_content(content: &str) -> std::io::Result<SimulationConfig> {
    let mut bodies = Vec::new();
    let mut body_data: Option<BodyData> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }

        // Check for section headers like [Body1], [Body2], etc.
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            // Save previous body if exists
            if let Some(body) = body_data.take() {
                if let Ok(b) = body.to_body() {
                    bodies.push(b);
                }
            }

            let section_name = &trimmed[1..trimmed.len() - 1];
            if section_name.to_lowercase().starts_with("body") {
                body_data = Some(BodyData::new());
            }
            continue;
        }

        // Parse key=value pairs
        if let Some(ref mut body) = body_data {
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim().to_lowercase();
                let mut value_str = trimmed[eq_pos + 1..].trim();
                
                // Strip inline comments
                if let Some(hash_pos) = value_str.find('#') {
                    value_str = &value_str[..hash_pos].trim();
                }
                if let Some(semi_pos) = value_str.find(';') {
                    value_str = &value_str[..semi_pos].trim();
                }

                if let Ok(value) = value_str.parse::<f64>() {
                    match key.as_str() {
                        "mass" => body.mass = value,
                        "position_x" => body.position_x = value,
                        "position_y" => body.position_y = value,
                        "position_z" => body.position_z = value,
                        "velocity_x" => body.velocity_x = value,
                        "velocity_y" => body.velocity_y = value,
                        "velocity_z" => body.velocity_z = value,
                        _ => {} // Ignore unknown keys
                    }
                }
            }
        }
    }

    // Don't forget the last body
    if let Some(body) = body_data {
        if let Ok(b) = body.to_body() {
            bodies.push(b);
        }
    }

    if bodies.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "No bodies found in configuration file",
        ));
    }

    Ok(SimulationConfig {
        bodies,
        time_step: 86400.0, // 1 day default
        num_steps: 1000,    // 1000 steps default
        output_file: String::from("results.csv"),
    })
}

/// Temporary structure to hold body data while parsing
#[derive(Debug, Clone)]
struct BodyData {
    mass: f64,
    position_x: f64,
    position_y: f64,
    position_z: f64,
    velocity_x: f64,
    velocity_y: f64,
    velocity_z: f64,
}

impl BodyData {
    fn new() -> Self {
        BodyData {
            mass: 0.0,
            position_x: 0.0,
            position_y: 0.0,
            position_z: 0.0,
            velocity_x: 0.0,
            velocity_y: 0.0,
            velocity_z: 0.0,
        }
    }

    fn to_body(&self) -> std::io::Result<Body> {
        if self.mass <= 0.0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Body mass must be positive",
            ));
        }

        Ok(Body::new(
            self.mass,
            [self.position_x, self.position_y, self.position_z],
            [self.velocity_x, self.velocity_y, self.velocity_z],
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
        let content = r#"
[Body1]
mass = 1e30
position_x = 0
position_y = 0
position_z = 0
velocity_x = 0
velocity_y = 0
velocity_z = 0

[Body2]
mass = 2e30
position_x = 1e11
position_y = 0
position_z = 0
velocity_x = 0
velocity_y = 500
velocity_z = 0
"#;

        let config = parse_ini_content(content).unwrap();
        assert_eq!(config.bodies.len(), 2);
        assert_eq!(config.bodies[0].mass, 1e30);
        assert_eq!(config.bodies[1].mass, 2e30);
        assert_eq!(config.bodies[1].velocity[1], 500.0);
    }

    #[test]
    fn test_parse_ignores_comments() {
        let content = r#"
# This is a comment
[Body1]
mass = 1e30  # Inline comment
position_x = 0
position_y = 0
position_z = 0
velocity_x = 0
velocity_y = 0
velocity_z = 0
"#;

        let config = parse_ini_content(content).unwrap();
        assert_eq!(config.bodies.len(), 1);
    }

    #[test]
    fn test_empty_config_fails() {
        let content = "# Just comments\n; More comments\n";
        let result = parse_ini_content(content);
        assert!(result.is_err());
    }
}
