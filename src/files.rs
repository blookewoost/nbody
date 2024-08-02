use std::path::Path;
use std::collections::HashMap;
use ini::configparser::ini::Ini;


// Check if a file exists.
pub fn file_check(filepath: &String) -> bool {
    if Path::new(filepath).exists() {
        return true;
    } else {
        return false;
    }
}

// Generate nested HashMap containing body data from .ini file.
pub fn ini_filemap(filepath: &String) -> Result<HashMap<String, HashMap<String, Option<String>>>, String> {

    let ini = match Ini::new().load(filepath) {
        Ok(ini) => {
            println!("Initialization file loaded.");
            ini
        },
        Err(e) => {
            return Err(e);
        }
    };

    for (sec, prop) in ini.iter() {
        println!("Section: {:?}", sec);
        for (k, v) in prop.iter() {
            println!("{}:{}", k, match v {
                Some(v) => v,
                None => "0"
            });
        }
    }

    return Ok(ini);

}

