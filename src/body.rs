use std::collections::HashMap;

pub struct Body {
    name: String,
    fx: f64,
    fy: f64,
    fz: f64,
    x: f64,
    y: f64,
    z: f64,
    vx: f64,
    vy: f64,
    vz: f64,
    ax: f64,
    ay: f64,
    az: f64
}

impl Body {
    pub fn new(name: String, data: HashMap<String, Option<String>>) -> Body {
        let x = match data.get("position_x") {
            Some(Some(v)) => v,
            Some(None) => "0.0",
            None => "0",
        };
        let value = match x.parse::<f64>() {
            Ok(value) => value,
            Err(e) => 0.0
        };
        
        return Body {x:value, ..Default::default()}
    }

    //fn retrieve_value()

    fn from_map(map: HashMap<String, Option<String>>) {

    }
}

impl Default for Body {
    fn default() -> Self {
        Body {
            name: "test".to_string(),
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