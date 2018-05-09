extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::Read;

#[derive(Deserialize, Debug)]
struct MakeOption {
    target: Option<Vec<String>>,
    example: Option<bool>,
    document: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct Entry {
    git: String,
    option: Option<MakeOption>,
}

type Configure = HashMap<String, Entry>;

fn load_config() -> Result<Configure, Box<Error>> {
    let mut f = fs::File::open("sample.toml")?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(toml::from_str(&s)?)
}

fn main() {
    let cfg = load_config().unwrap();
    println!("cfg = {:?}", cfg);
}
