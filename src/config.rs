use dirs;
use log::info;
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};

use crate::error::Result;

pub const APP_NAME: &'static str = "llvmenv";
pub const ENTRY_TOML: &'static str = "entry.toml";

/// Example of entry.toml
const DEFAULT_ENTRY: &'static [u8] = br#"
[llvm-dev]
llvm_git  = "https://github.com/llvm-mirror/llvm"
clang_git = "https://github.com/llvm-mirror/clang"
build     = "Release"
target    = ["X86"]
example   = 0
document  = 0
"#;

pub fn config_dir() -> PathBuf {
    let home = match env::var("XDG_CONFIG_HOME") {
        Ok(path) => path.into(),
        Err(_) => dirs::home_dir()
            .expect("$HOME does not found")
            .join(".config"), // Use $HOME/.config
    };
    home.join(APP_NAME)
}

pub fn cache_dir() -> PathBuf {
    let home = match env::var("XDG_CACHE_HOME") {
        Ok(path) => path.into(),
        Err(_) => dirs::home_dir()
            .expect("$HOME does not found")
            .join(".cache"), // Use $HOME/.cache
    };
    home.join(APP_NAME)
}

pub fn data_dir() -> PathBuf {
    let home = match env::var("XDG_DATA_HOME") {
        Ok(path) => path.into(),
        Err(_) => dirs::home_dir()
            .expect("$HOME does not found")
            .join(".local")
            .join("share"), // Use $HOME/.local/share/llvmenv
    };
    home.join(APP_NAME)
}

/// Initialize configure directory `$XDG_CONFIG_HOME/llvmenv/`
pub fn init_config() -> Result<()> {
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
