use std::env;
use std::process;
use std::path::Path;

use files::file_check;
use files::ini_filemap;

mod files;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: ./nbody <filename>");
        process::exit(0);
    } else {
        let filepath: &String = &args[1];
        if file_check(filepath) {
            ini_filemap(filepath);
        } else {
            println!("Booboo");
        }
    }
}
