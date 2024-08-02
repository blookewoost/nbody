use std::fmt::Error;
use std::io::Read;
use std::path::Path;
use std::collections::HashMap;
use std::fs::File;
use ini::Ini;


// Check if a file exists.
pub fn file_check(filepath: &String) -> bool {
    if Path::new(filepath).exists() {
        return true;
    } else {
        return false;
    }
}

// Generate nested HashMap containing body data from .ini file.
pub fn ini_filemap(filepath: &String) -> Result<HashMap<String, HashMap<String, f64>>, Error> {
    let mut ini_file = match File::open(filepath) {
        Ok(ini_file) => ini_file,
        Err(e) => {
            return Err(e);
        }
    };

    let mut contents = String::new();
    match ini_file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            return Err(e);
        }
    };

    
}

