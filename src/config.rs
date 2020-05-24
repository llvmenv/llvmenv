use log::info;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::error::*;

pub const APP_NAME: &str = "llvmenv";
pub const ENTRY_TOML: &str = "entry.toml";

const LLVM_PROJECT: &str = include_str!("llvm-project.toml");

pub fn config_dir() -> Result<PathBuf> {
    let path = dirs::config_dir()
        .ok_or(Error::UnsupportedOS)?
        .join(APP_NAME);
    if !path.exists() {
        fs::create_dir_all(&path).with(&path)?;
    }
    Ok(path)
}

pub fn cache_dir() -> Result<PathBuf> {
    let path = dirs::cache_dir()
        .ok_or(Error::UnsupportedOS)?
        .join(APP_NAME);
    if !path.exists() {
        fs::create_dir_all(&path).with(&path)?;
    }
    Ok(path)
}

pub fn data_dir() -> Result<PathBuf> {
    let path = dirs::data_dir().ok_or(Error::UnsupportedOS)?.join(APP_NAME);
    if !path.exists() {
        fs::create_dir_all(&path).with(&path)?;
    }
    Ok(path)
}

/// Initialize configure file
pub fn init_config() -> Result<()> {
    let dir = config_dir()?;
    let entry = dir.join(ENTRY_TOML);
    if !entry.exists() {
        info!("Create default entry setting: {}", entry.display());
        let mut f = fs::File::create(&entry).with(&entry)?;
        f.write(LLVM_PROJECT.as_bytes()).with(&entry)?;
        Ok(())
    } else {
        Err(Error::ConfigureAlreadyExists { path: entry })
    }
}
