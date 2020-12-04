use log::info;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::error::*;

pub const APP_NAME: &str = "llvmenv";
pub const ENTRY_TOML: &str = "entry.toml";

const LLVM_MIRROR: &str = include_str!("llvm-mirror.toml");

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

#[cfg(target_os = "macos")]
pub fn homebrew_dir() -> Option<PathBuf> {
    std::process::Command::new("brew")
        .arg("--prefix")
        .check_output()
        .ok()
        .map(|(stdout, _)| {
            let path = PathBuf::from(stdout.trim()).join("opt");

            info!("found homebrew @ {}", path.display());

            path
        })
}

#[cfg(target_os = "macos")]
pub fn append_homebrew_llvm<P: AsRef<std::path::Path>>(dir: P, out: &mut dyn Write) -> Result<()> {
    use semver::Version;
    use std::os::unix::fs::symlink;
    use std::process::Command;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if file_name == "llvm" || file_name.starts_with("llvm@") {
            let (stdout, _) = Command::new(path.join("bin/llvm-config"))
                .arg("--version")
                .check_output()?;
            let version = Version::parse(&stdout).map_err(|_| Error::invalid_version(&stdout))?;
            let name = format!("homebrew-llvm{}", version.major);

            info!("found {} @ {}", name, path.display());

            let target = data_dir()?.join(&name);
            if !target.exists() {
                symlink(&path, &target)?;
            }

            write!(
                out,
                r#"
[{name}]
name = "{name}"
version = "{version}"
path = "{path}"
"#,
                name = name,
                version = version,
                path = path.display(),
            )?;
        }
    }

    Ok(())
}

/// Initialize configure file
pub fn init_config(force: bool) -> Result<()> {
    let dir = config_dir()?;
    let entry = dir.join(ENTRY_TOML);
    if force || !entry.exists() {
        info!("Create default entry setting: {}", entry.display());
        let mut f = fs::File::create(&entry).with(&entry)?;
        f.write(LLVM_MIRROR.as_bytes()).with(&entry)?;
        #[cfg(target_os = "macos")]
        if let Some(dir) = homebrew_dir() {
            append_homebrew_llvm(dir, &mut f)?;
        }
        Ok(())
    } else {
        Err(Error::ConfigureAlreadyExists { path: entry })
    }
}
