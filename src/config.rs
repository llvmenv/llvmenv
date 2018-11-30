use dirs;
use failure::bail;
use log::info;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::error::Result;

pub const APP_NAME: &'static str = "llvmenv";
pub const ENTRY_TOML: &'static str = "entry.toml";

const LLVM_MIRROR: &str = include_str!("llvm-mirror.toml");

pub fn config_dir() -> PathBuf {
    let path = dirs::home_dir().expect("Unsupported OS").join(APP_NAME);
    if !path.exists() {
        fs::create_dir_all(&path).expect(&format!("Cannot create configure at {}", path.display()));
    }
    path
}

pub fn cache_dir() -> PathBuf {
    let path = dirs::cache_dir().expect("Unsupported OS").join(APP_NAME);
    if !path.exists() {
        fs::create_dir_all(&path).expect(&format!(
            "Cannot create cache directory at {}",
            path.display()
        ));
    }
    path
}

pub fn data_dir() -> PathBuf {
    let path = dirs::data_dir().expect("Unsupported OS").join(APP_NAME);
    if !path.exists() {
        fs::create_dir_all(&path).expect(&format!(
            "Cannot create data directory at {}",
            path.display()
        ));
    }
    path
}

/// Initialize configure file
pub fn init_config() -> Result<()> {
    let dir = config_dir();
    let entry = dir.join(ENTRY_TOML);
    if !entry.exists() {
        info!("Create default entry setting: {}", entry.display());
        let mut f = fs::File::create(entry)?;
        f.write(LLVM_MIRROR.as_bytes())?;
    } else {
        bail!("Setting already exists.");
    }
    Ok(())
}
