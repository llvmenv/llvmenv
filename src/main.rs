extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
extern crate itertools;

mod build;
mod config;
mod error;

use build::*;
use config::*;

fn main() {
    init_config().expect("Initialization failed");
    let entries = load_entries().expect(&format!("Failed to load {}", ENTRY_TOML));
    for entry in entries.iter() {
        entry.fetch().unwrap();
        entry.prebuild().unwrap();
    }
}
