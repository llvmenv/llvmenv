use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{env, fs};
use toml;

use config::*;

#[derive(Deserialize, Debug)]
struct MakeOption {
    subdir: Option<String>,
    build_path: Option<String>,
    target: Option<Vec<String>>,
    example: Option<bool>,
    document: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct Entry {
    git: String,
    option: Option<MakeOption>,
}

type Entries = HashMap<String, Entry>;

pub fn entries() -> Result<Entries> {
    let toml = config_dir().join(ENTRY_TOML);
    let mut f = fs::File::open(toml)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(toml::from_str(&s)?)
}
