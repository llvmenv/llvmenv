extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{env, fs};

const APP_NAME: &'static str = "llvmenv";
const ENTRY_TOML: &'static str = "entry.toml";

const DEFAULT_ENTRY: &'static [u8] = br#"
[llvm-dev]
git = "https://github.com/llvm-mirror/llvm"

[llvm-dev.option]
target   = ["X86"]
example  = false
document = false
"#;

type Result<T> = ::std::result::Result<T, Box<::std::error::Error>>;

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

fn entries() -> Result<Entries> {
    let toml = config_dir().join(ENTRY_TOML);
    let mut f = fs::File::open(toml)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    Ok(toml::from_str(&s)?)
}

fn config_dir() -> PathBuf {
    let home = match env::var("XDG_CONFIG_HOME") {
        Ok(path) => path.into(),
        Err(_) => env::home_dir()
            .expect("$HOME does not found")
            .join(".config"), // Use $HOME/.config
    };
    home.join(APP_NAME)
}

fn cache_dir() -> PathBuf {
    let home = match env::var("XDG_CACHE_HOME") {
        Ok(path) => path.into(),
        Err(_) => env::home_dir()
            .expect("$HOME does not found")
            .join(".cache"), // Use $HOME/.cache
    };
    home.join(APP_NAME)
}

fn data_dir() -> PathBuf {
    let home = match env::var("XDG_DATA_HOME") {
        Ok(path) => path.into(),
        Err(_) => env::home_dir()
            .expect("$HOME does not found")
            .join(".local")
            .join("share"), // Use $HOME/.local/share/llvmenv
    };
    home.join(APP_NAME)
}

/// Initialize configure directory `$XDG_CONFIG_HOME/llvmenv/`
fn init_config() -> Result<()> {
    let dir = config_dir();
    if !dir.exists() {
        info!("Create directory: {}", dir.display());
        fs::create_dir_all(&dir)?;
    }
    let entry = dir.join(ENTRY_TOML);
    if !entry.exists() {
        info!("Create default entry setting: {}", entry.display());
        let mut f = fs::File::create(entry)?;
        f.write(DEFAULT_ENTRY)?;
    }
    Ok(())
}

fn main() {
    init_config().expect("Failed to init...");
    let e = entries().expect(&format!("Failed to load {}", ENTRY_TOML));
    println!("entries = {:?}", e);
}
