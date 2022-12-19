use log::info;
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};

use crate::error::*;

pub const APP_NAME: &str = "llvmenv";
pub const ENTRY_TOML: &str = "entry.toml";

const LLVM_MIRROR: &str = include_str!("llvm-mirror.toml");

pub fn config_dir() -> Result<PathBuf> {
    let custom_llvmenv_config_dir = env::var("LLVMENV_CONFIG_DIR");
    let path = match custom_llvmenv_config_dir {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => dirs::config_dir()
            .ok_or(Error::UnsupportedOS)?
            .join(APP_NAME),
    };
    if !path.exists() {
        fs::create_dir_all(&path).with(&path)?;
    }
    return Ok(path);
}

pub fn cache_dir() -> Result<PathBuf> {
    let custom_llvmenv_cache_dir = env::var("LLVMENV_CACHE_DIR");

    let path = match custom_llvmenv_cache_dir {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => dirs::cache_dir()
            .ok_or(Error::UnsupportedOS)?
            .join(APP_NAME),
    };
    if !path.exists() {
        fs::create_dir_all(&path).with(&path)?;
    }
    Ok(path)
}

pub fn data_dir() -> Result<PathBuf> {
    let custom_llvmenv_cache_dir = env::var("LLVMENV_DATA_DIR");
    let path = match custom_llvmenv_cache_dir {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => dirs::data_dir().ok_or(Error::UnsupportedOS)?.join(APP_NAME),
    };
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
        f.write(LLVM_MIRROR.as_bytes()).with(&entry)?;
        Ok(())
    } else {
        Err(Error::ConfigureAlreadyExists { path: entry })
    }
}
