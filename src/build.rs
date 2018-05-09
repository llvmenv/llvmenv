use std::collections::HashMap;
use std::fs;
use std::io::Read;
use toml;

use config::*;
use error::Result;

#[derive(Deserialize, Debug)]
pub struct MakeOption {
    subdir: Option<String>,
    build_path: Option<String>,
    target: Option<Vec<String>>,
    example: Option<bool>,
    document: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct Entry {
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

/// Download Git repository using `git clone`
fn download(name: &str, entry: &Entry) -> Result<()> {
    Ok(())
}
