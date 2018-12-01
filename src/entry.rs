//! Manage entries, i.e. LLVM/Clang source to be built

use failure::bail;
use itertools::*;
use log::{info, warn};
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{fs, process};
use toml;

use crate::config::*;
use crate::error::*;
use crate::resource::Resource;

/// Option for CMake Generators
///
/// - Official document: [CMake Generators](https://cmake.org/cmake/help/latest/manual/cmake-generators.7.html)
#[derive(Deserialize, Debug)]
pub enum CMakeGenerator {
    /// Use platform default generator (without -G option)
    Platform,
    /// Unix Makefile
    Makefile,
    /// Ninja builder
    Ninja,
    /// Visual Studio 15 2017
    VisualStudio,
}

impl CMakeGenerator {
    fn option(&self) -> Vec<String> {
        match self {
            CMakeGenerator::Platform => Vec::new(),
            CMakeGenerator::Makefile => vec!["-G", "Unix Makefiles"],
            CMakeGenerator::Ninja => vec!["-G", "Ninja"],
            CMakeGenerator::VisualStudio => vec!["-G", "Visual Studio 15 2017"],
        }
        .into_iter()
        .map(|s| s.into())
        .collect()
    }

    fn build_option(&self, nproc: usize) -> Vec<String> {
        match self {
            CMakeGenerator::VisualStudio | CMakeGenerator::Platform => Vec::new(),
            CMakeGenerator::Makefile | CMakeGenerator::Ninja => {
                vec!["--".into(), "-j".into(), format!("{}", nproc)]
            }
        }
    }
}

impl Default for CMakeGenerator {
    fn default() -> Self {
        CMakeGenerator::Platform
    }
}

/// CMake build type
#[derive(Deserialize, Debug)]
pub enum BuildType {
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

impl Tool {
    fn rel_path(&self) -> String {
        match self.relative_path {
            Some(ref rel_path) => rel_path.to_string(),
            None => format!("tools/{}", self.name),
        }
    }
}

/// Setting for both Remote and Local entries. TOML setting file will be decoded into this struct.
#[derive(Deserialize, Debug, Default)]
pub struct EntrySetting {
    /// URL of remote LLVM resource, see also [resouce](../resource/index.html) module
    pub url: Option<String>,
    /// Path of local LLVM source dir
    pub path: Option<String>,
    /// Additional LLVM Tools, e.g. clang, openmp, lld, and so on.
    #[serde(default)]
    pub tools: Vec<Tool>,
    /// Target to be build. Empty means all backend
    #[serde(default)]
    pub target: Vec<String>,
    /// Additional LLVM build options
    #[serde(default)]
    pub option: HashMap<String, String>,
    /// CMake Generator option (-G option in cmake)
    #[serde(default)]
    pub builder: CMakeGenerator,
    ///  Option for `CMAKE_BUILD_TYPE`
    #[serde(default)]
    pub build_type: BuildType,
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

impl Entry {
    fn parse_setting(name: &str, setting: EntrySetting) -> Result<Self> {
        if setting.path.is_some() && setting.url.is_some() {
            bail!("One of Path or URL are allowed");
        }
        if let Some(path) = &setting.path {
            if setting.tools.len() > 0 {
                warn!("'tools' must be used with URL, ignored");
            }
            return Ok(Entry::Local {
                name: name.into(),
                path: PathBuf::from(shellexpand::full(&path)?.to_string()),
                setting,
            });
        }
        if let Some(url) = &setting.url {
            return Ok(Entry::Remote {
                name: name.into(),
                url: url.clone(),
                tools: setting.tools.clone(),
                setting,
            });
        }
        bail!("Path nor URL are not found: {}", name);
    }
}

fn load_entry_toml(toml_str: &str) -> Result<Vec<Entry>> {
    let entries: HashMap<String, EntrySetting> = toml::from_str(toml_str)?;
    entries
        .into_iter()
        .map(|(name, setting)| Entry::parse_setting(&name, setting))
        .collect()
}

fn official_releases() -> Result<Vec<Entry>> {
    [
        (7, 0, 0),
        (6, 0, 1),
        (6, 0, 0),
        (5, 0, 2),
        (5, 0, 1),
        (4, 0, 1),
        (4, 0, 0),
        (3, 9, 1),
        (3, 9, 0),
    ]
    .into_iter()
    .map(|(major, minor, patch)| {
        let version = format!("{}.{}.{}", major, minor, patch);
        let mut setting = EntrySetting::default();
        setting.url = Some(format!(
            "http://releases.llvm.org/{version}/llvm-{version}.src.tar.xz",
            version = version
        ));
        let clang = Tool {
            name: "clang".into(),
            url: format!(
                "http://releases.llvm.org/{version}/cfe-{version}.src.tar.xz",
                version = version
            ),
            branch: None,
            relative_path: None,
        };
        let lld = Tool {
            name: "lld".into(),
            url: format!(
                "http://releases.llvm.org/{version}/lld-{version}.src.tar.xz",
                version = version
            ),
            branch: None,
            relative_path: None,
        };
        setting.tools = vec![clang, lld];
        Entry::parse_setting(&version, setting)
    })
    .collect()
}

pub fn load_entries() -> Result<Vec<Entry>> {
    let global_toml = config_dir()?.join(ENTRY_TOML);
    let mut entries = load_entry_toml(&fs::read_to_string(global_toml)?)?;
    let mut official = official_releases()?;
    entries.append(&mut official);
    Ok(entries)
}

pub fn load_entry(name: &str) -> Result<Entry> {
    let entries = load_entries()?;
    for entry in entries {
        if entry.name() == name {
            return Ok(entry);
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

    pub fn checkout(&self) -> Result<()> {
        match self {
            Entry::Remote { url, tools, .. } => {
                if !self.src_dir()?.is_dir() {
                    let src = Resource::from_url(url)?;
                    src.download(&self.src_dir()?)?;
                }
                for tool in tools {
                    let path = self.src_dir()?.join(tool.rel_path());
                    if !path.is_dir() {
                        let src = Resource::from_url(&tool.url)?;
                        src.download(&path)?;
                    }
                }
            }
            Entry::Local { path, .. } => {
                if !path.is_dir() {
                    bail!("Path '{}' is not a directory", path.display())
                }
            }
        }
        Ok(())
    }

    pub fn update(&self) -> Result<()> {
        match self {
            Entry::Remote { url, tools, .. } => {
                let src = Resource::from_url(url)?;
                src.update(&self.src_dir()?)?;
                for tool in tools {
                    let src = Resource::from_url(&tool.url)?;
                    src.update(&self.src_dir()?.join(tool.rel_path()))?;
                }
            }
            Entry::Local { .. } => {}
        }
        Ok(())
    }

    pub fn name(&self) -> &str {
        match self {
            Entry::Remote { name, .. } => name,
            Entry::Local { name, .. } => name,
        }
    }

    pub fn src_dir(&self) -> Result<PathBuf> {
        Ok(match self {
            Entry::Remote { name, .. } => cache_dir()?.join(name),
            Entry::Local { path, .. } => path.into(),
        })
    }

    pub fn build_dir(&self) -> Result<PathBuf> {
        let dir = self.src_dir()?.join("build");
        if !dir.exists() {
            info!("Create build dir: {}", dir.display());
            fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }

    pub fn prefix(&self) -> Result<PathBuf> {
        Ok(data_dir()?.join(self.name()))
    }

    pub fn build(&self, nproc: usize) -> Result<()> {
        self.configure()?;
        process::Command::new("cmake")
            .args(&[
                "--build",
                &format!("{}", self.build_dir()?.display()),
                "--target",
                "install",
            ])
            .args(&self.setting().builder.build_option(nproc))
            .check_run()?;
        Ok(())
    }

    pub fn clean(&self) -> Result<()> {
        fs::remove_dir_all(self.build_dir()?)?;
        Ok(())
    }

    fn configure(&self) -> Result<()> {
        let setting = self.setting();
        let mut opts = setting.builder.option();
        opts.push(format!("{}", self.src_dir()?.display()));
        opts.push(format!(
            "-DCMAKE_INSTALL_PREFIX={}",
            data_dir()?.join(self.prefix()?).display()
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
        process::Command::new("cmake")
            .args(&opts)
            .current_dir(self.build_dir()?)
            .check_run()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_setting() -> Result<()> {
        let setting = EntrySetting {
            url: None,
            path: None,
            tools: Default::default(),
            option: Default::default(),
            builder: Default::default(),
            build_type: Default::default(),
            target: Default::default(),
        };
        assert!(Entry::parse_setting("no_entry", setting).is_err());

        let setting = EntrySetting {
            url: Some("http://llvm.org/svn/llvm-project/llvm/trunk".into()),
            path: Some("~/.config/llvmenv".into()),
            tools: Default::default(),
            option: Default::default(),
            builder: Default::default(),
            build_type: Default::default(),
            target: Default::default(),
        };
        assert!(Entry::parse_setting("duplicated", setting).is_err());

        Ok(())
    }
}
