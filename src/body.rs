use std::collections::HashMap;

pub struct Body {
    pub name: String,
    pub mass: f64,
    pub fx: f64,
    pub fy: f64,
    pub fz: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub ax: f64,
    pub ay: f64,
    pub az: f64
}

impl Body {
    pub fn populate(&mut self, data: HashMap<String, Option<String>>) {
        for (key, value) in data {
            let parsed_value = match value {
                Some(string_value) => match string_value.parse::<f64>() {
                    Ok(parsed) => parsed,
                    Err(e) => {
                        println!("Error parsing value for field: {}... populating with 0", key);
                        0.0
                    }
                }
                None => 0.0
            };
            
            self.keymap(&key, parsed_value);
        }
    }

    fn keymap(&mut self, key: &str, value: f64) {
        match key {
            "mass" => self.mass = value,
            "position_x" => self.x = value,
            "position_y" => self.y = value,
            "position_z" => self.z = value,
            "velocity_x" => self.vx = value,
            "velocity_y" => self.vy = value,
            "velocity_z" => self.vz = value,
            _ => println!("Skipping unknown field: {}", key)
        }
    }

    pub fn new(name: String) -> Body {
        return Body {name, ..Default::default()}
    }
}


impl Default for Body {
    fn default() -> Self {
        Body {
            mass: 0.0,
            name: "Default".to_string(),
            fx: 0.0,
            fy: 0.0,
            fz: 0.0,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            ax: 0.0,
            ay: 0.0,
            az: 0.0
        }
    }
}