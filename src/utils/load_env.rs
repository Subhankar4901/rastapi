//! # load_env
//!
//! It's a module that helps loading local enviornment variable in .env file into *std::env*.
//! It has only one function *load_env()* which when called reads the .env file and loads the
//! enviornment variables into *std::env*.
//! just call it once before using any enviornment variables.
//!
//! ## Example
//! ```no_run
//!     use rastapi::utils::load_env::load_env;
//!     use std::env;
//!     fn main(){
//!         load_env();
//!         // Now use enviornment variables in .env using *std::env*.
//!     }
//! ```
use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
pub fn load_env() {
    // Open the .env file
    let base_dir = env!("CARGO_MANIFEST_DIR");
    let p = PathBuf::from(base_dir).join(".env");
    // let p=PathBuf::from_str(".env").unwrap();
    let file = match File::open(&p) {
        Ok(f) => f,
        Err(_e) => {
            panic!("Couldn't find .env file. at {:?}", p);
        }
    };
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    let _n = match reader.read_to_string(&mut content) {
        Ok(n) => n,
        Err(e) => panic!("Couldn't read from buffer.\n{e}"),
    };
    // Iterate over each line in the file
    for line in content.lines() {
        // Split each line by the "=" character to separate key and value
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() == 2 {
            let key = parts[0].trim();
            let value = parts[1].trim();
            // Set the environment variable
            env::set_var(key, value);
        }
    }
}
