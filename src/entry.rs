//! Manage entries, i.e. LLVM/Clang source to be built

use itertools::*;
use log::*;
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::path::{PathBuf, Path};
use std::{fs, process};
use failure::bail;
use toml;

use crate::config::*;
use crate::error::*;

#[derive(Deserialize, Debug)]
enum Builder {
    Makefile,
    Ninja,
}

impl Default for Builder {
    fn default() -> Self {
        Builder::Ninja
    }
}

#[derive(Deserialize, Debug)]
enum BuildType {
    Debug,
    Release,
}

impl Default for BuildType {
    fn default() -> Self {
        BuildType::Release
    }
}

/// LLVM Tools e.g. clang, compiler-rt, and so on.
#[derive(Deserialize, Debug, Clone)]
pub struct Tool {
    name: String,
    url: String,
    branch: Option<String>,
    relative_path: Option<String>,
}

/// Setting for both Remote and Local entries
#[derive(Deserialize, Debug)]
pub struct EntrySetting {
    url: Option<String>,
    path: Option<PathBuf>,
    #[serde(default)]
    tools: Vec<Tool>,
    /// empty means all backend
    #[serde(default)]
    target: Vec<String>,
    /// other LLVM build options
    #[serde(default)]
    option: HashMap<String, String>,
    #[serde(default)]
    builder: Builder,
    #[serde(default)]
    build_type: BuildType,
}

#[derive(Debug)]
pub enum Entry {
    Remote {
        name: String,
        url: String,
        tools: Vec<Tool>,
        setting: EntrySetting,
    },
    Local {
        name: String,
        path: PathBuf,
        setting: EntrySetting,
    },
}

fn load_entry_toml(toml_filename: &Path) -> Result<Vec<Entry>> {
    type EntryTOML = HashMap<String, EntrySetting>;
    let input: EntryTOML = toml::from_str(&fs::read_to_string(toml_filename)?)?;
    let mut entries = Vec::new();
    for (name, setting) in input {
        if let Some(path) = &setting.path {
            entries.push(Entry::Local {
                name: name.into(),
                path: path.into(),
                setting,
            });
            continue;
        }
        if let Some(url) = &setting.url {
            entries.push(Entry::Remote {
                name: name.into(),
                url: url.clone(),
                tools: setting.tools.clone(),
                setting,
            });
            continue;
        }
        warn!("Ignore entry: {}", name);
    }
    Ok(entries)
}

pub fn load_entries() -> Result<Vec<Entry>> {
    load_entry_toml(&config_dir().join(ENTRY_TOML))
}

pub fn load_entry(name: &str) -> Result<Entry> {
    let entries = load_entries()?;
    for entry in entries {
        if entry.name() == name {
            return Ok(entry)
        }
    }
    bail!("No entries are found: {}", name);
}

impl Entry {
    fn setting(&self) -> &EntrySetting {
        match self {
            Entry::Remote { setting, .. } => setting,
            Entry::Local { setting, .. } => setting,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Entry::Remote { name, .. } => name,
            Entry::Local { name, .. } => name,
        }
    }

    pub fn src_dir(&self) -> PathBuf {
        match self {
            Entry::Remote { name, .. } => cache_dir().join(name),
            Entry::Local { path, .. } => path.into(),
        }
    }

    pub fn build_dir(&self) -> PathBuf {
        self.src_dir().join("build")
    }

    pub fn prefix(&self) -> PathBuf {
        data_dir().join(self.name())
    }

    pub fn build(&self, nproc: usize) -> Result<()> {
        self.configure()?;
        process::Command::new("cmake")
            .args(&[
                "--build",
                &format!("{}", self.build_dir().display()),
                "--target",
                "install",
                "-j",
                &format!("{}", nproc),
            ]).check_run()?;
        Ok(())
    }

    fn configure(&self) -> Result<()> {
        let setting = self.setting();
        let mut opts = Vec::new();
        opts.push(format!("-H{}", self.src_dir().display()));
        opts.push(format!("-B{}", self.build_dir().display()));
        match setting.builder {
            Builder::Ninja => {
                opts.push("-G".into());
                opts.push("Ninja".into());
            }
            _ => {}
        }
        opts.push(format!(
            "-DCMAKE_INSTALL_PREFIX={}",
            data_dir().join(self.prefix()).display()
        ));
        opts.push(format!("-DCMAKE_BUILD_TYPE={:?}", setting.build_type));
        if setting.target.len() > 0 {
            opts.push(format!(
                "-DLLVM_TARGETS_TO_BUILD={}",
                setting.target.iter().join(";")
            ));
        }
        for (k, v) in &setting.option {
            opts.push(format!("-D{}={}", k, v));
        }
        process::Command::new("cmake").args(&opts).check_run()?;
        Ok(())
    }
}
