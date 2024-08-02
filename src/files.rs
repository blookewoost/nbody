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
            return Ok(ini);
        },
        Err(e) => {
            return Err(e);
        }
    };

}

