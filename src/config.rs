use anyhow::{bail, format_err};
use dirs;
use log::info;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::error::Result;

pub const APP_NAME: &'static str = "llvmenv";
pub const ENTRY_TOML: &'static str = "entry.toml";

const LLVM_MIRROR: &str = include_str!("llvm-mirror.toml");

pub fn config_dir() -> Result<PathBuf> {
    let path = dirs::config_dir()
        .ok_or(format_err!("Unsupported OS"))?
        .join(APP_NAME);
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}

pub fn cache_dir() -> Result<PathBuf> {
    let path = dirs::cache_dir()
        .ok_or(format_err!("Unsupported OS"))?
        .join(APP_NAME);
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}

pub fn data_dir() -> Result<PathBuf> {
    let path = dirs::data_dir()
        .ok_or(format_err!("Unsupported OS"))?
        .join(APP_NAME);
    if !path.exists() {
        fs::create_dir_all(&path)?;
    }
    Ok(path)
}

/// Initialize configure file
pub fn init_config() -> Result<()> {
    let dir = config_dir()?;
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
