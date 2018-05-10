extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
extern crate itertools;

mod build;
mod config;
mod error;

use config::*;

fn main() {
    init_config().expect("Initialization failed");
    let entries = load_entries().unwrap();
    for entry in entries.iter() {
        entry.clone().unwrap();
        entry.fetch().unwrap();
        entry.prebuild().unwrap();
    }
}
