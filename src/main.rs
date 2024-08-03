use std::env;
use std::process;
use std::path::Path;
use std::collections::HashMap;

use files::file_check;
use files::ini_filemap;

use body::Body;
use physics::calculate_forces;

mod files;
mod body;
mod physics;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: ./nbody <filename>");
        process::exit(0);
    } else {
        let filepath: &String = &args[1];
        if file_check(filepath) {
            let mut bodies: Vec<Body> = Vec::new();
            let map: HashMap<String, HashMap<String, Option<String>>> = ini_filemap(filepath).unwrap();
            for (section, props) in map {
                let mut body = Body::new(section);
                body.populate(props);
                bodies.push(body);
            }
            calculate_forces(bodies);
            
        } else {
            println!("Booboo");
        }
    }
}
