use std::env;
use std::process;
use std::path::Path;
use std::collections::HashMap;

use files::file_check;
use files::ini_filemap;

use body::Body;

mod files;
mod body;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: ./nbody <filename>");
        process::exit(0);
    } else {
        let filepath: &String = &args[1];
        if file_check(filepath) {
            let mut bodies: Vec<Body> = Vec::new();
            let map = ini_filemap(filepath).unwrap();
            for (section, props) in map {
                let body = Body::new(section, props);
                println!("The value of x for body:{} is {}", body.name, body.x);
                //bodies.push(Body::new(section, props));
                //for (k, v) in props {
                //    println!("{}:{}", k, match v {
                //        Some(v) => v,
                //        None => "0".to_string(),
                //    })
                //}
            }
        } else {
            println!("Booboo");
        }
    }
}
