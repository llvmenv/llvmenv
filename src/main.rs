extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
#[macro_use]
extern crate log;

mod build;
mod config;
mod error;

use build::*;
use config::*;

fn main() {
    init_config().expect("Failed to init...");
    let e = entries().expect(&format!("Failed to load {}", ENTRY_TOML));
    println!("entries = {:?}", e);
}
