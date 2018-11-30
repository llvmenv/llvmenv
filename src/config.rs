use dirs;
use failure::bail;
use log::info;
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};

use crate::error::Result;

pub const APP_NAME: &'static str = "llvmenv";
pub const ENTRY_TOML: &'static str = "entry.toml";

const LLVM_MIRROR: &str = r#"
[llvm-mirror]
url    = "https://github.com/llvm-mirror/llvm"
target = ["X86"]

[[llvm-mirror.tools]]
name = "clang"
url = "https://github.com/llvm-mirror/clang"

[[llvm-mirror.tools]]
name = "clang-extra"
url = "https://github.com/llvm-mirror/clang-tools-extra"
relative_path = "tools/clang/tools/extra"
"#;

pub fn config_dir() -> PathBuf {
    let home = match env::var("XDG_CONFIG_HOME") {
        Ok(path) => path.into(),
        Err(_) => dirs::home_dir()
            .expect("$HOME does not found")
            .join(".config"), // Use $HOME/.config
    };
    let path = home.join(APP_NAME);
    if !path.exists() {
        fs::create_dir_all(&path).expect(&format!("Cannot create configure at {}", path.display()));
    }
    path
}

pub fn cache_dir() -> PathBuf {
    let home = match env::var("XDG_CACHE_HOME") {
        Ok(path) => path.into(),
        Err(_) => dirs::home_dir()
            .expect("$HOME does not found")
            .join(".cache"), // Use $HOME/.cache
    };
    let path = home.join(APP_NAME);
    if !path.exists() {
        fs::create_dir_all(&path).expect(&format!(
            "Cannot create cache directory at {}",
            path.display()
        ));
    }
    path
}

pub fn data_dir() -> PathBuf {
    let home = match env::var("XDG_DATA_HOME") {
        Ok(path) => path.into(),
        Err(_) => dirs::home_dir()
            .expect("$HOME does not found")
            .join(".local")
            .join("share"), // Use $HOME/.local/share/llvmenv
    };
    let path = home.join(APP_NAME);
    if !path.exists() {
        fs::create_dir_all(&path).expect(&format!(
            "Cannot create data directory at {}",
            path.display()
        ));
    }
    path
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
        f.write(LLVM_MIRROR.as_bytes())?;
    } else {
        bail!("Setting already exists.");
    }
    Ok(())
}
