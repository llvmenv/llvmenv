//! Describes how to compile LLVM/Clang
//!
//! entry.toml
//! -----------
//! **entry** in llvmenv describes how to compile LLVM/Clang, and set by `$XDG_CONFIG_HOME/llvmenv/entry.toml`.
//! `llvmenv init` generates default setting:
//!
//! ```toml
//! [llvm-mirror]
//! url    = "https://github.com/llvm-mirror/llvm"
//! target = ["X86"]
//!
//! [[llvm-mirror.tools]]
//! name = "clang"
//! url = "https://github.com/llvm-mirror/clang"
//!
//! [[llvm-mirror.tools]]
//! name = "clang-extra"
//! url = "https://github.com/llvm-mirror/clang-tools-extra"
//! relative_path = "tools/clang/tools/extra"
//! ```
//!
//! (TOML format has been changed largely at version 0.2.0)
//!
//! **tools** property means LLVM tools, e.g. clang, compiler-rt, lld, and so on.
//! These will be downloaded into `${llvm-top}/tools/${tool-name}` by default,
//! and `relative_path` property change it.
//! This toml will be decoded into [EntrySetting][EntrySetting] and normalized into [Entry][Entry].
//!
//! [Entry]: ./enum.Entry.html
//! [EntrySetting]: ./struct.EntrySetting.html
//!
//! Local entries (since v0.2.0)
//! -------------
//! Different from above *remote* entries, you can build locally cloned LLVM source with *local* entry.
//!
//! ```toml
//! [my-local-llvm]
//! path = "/path/to/your/src"
//! target = ["X86"]
//! ```
//!
//! Entry is regarded as *local* if there is `path` property, and *remote* if there is `url` property.
//! Other options are common to *remote* entries.
//!
//! Pre-defined entries
//! ------------------
//!
//! There is also pre-defined entries corresponding to the LLVM/Clang releases:
//!
//! ```shell
//! $ llvmenv entries
//! llvm-mirror
//! 7.0.0
//! 6.0.1
//! 6.0.0
//! 5.0.2
//! 5.0.1
//! 4.0.1
//! 4.0.0
//! 3.9.1
//! 3.9.0
//! ```
//!
//! These are compiled with the default setting as shown above. You have to create entry manually
//! if you want to use custom settings.

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
#[derive(Deserialize, PartialEq, Debug)]
pub enum CMakeGenerator {
    /// Use platform default generator (without -G option)
    Platform,
    /// Unix Makefile
    Makefile,
    /// Ninja generator
    Ninja,
    /// Visual Studio 15 2017
    VisualStudio,
    /// Visual Studio 15 2017 Win64
    VisualStudioWin64,
}

impl CMakeGenerator {
    /// ```
    /// # use llvmenv::entry::CMakeGenerator;
    /// assert_eq!(CMakeGenerator::from_str("Makefile").unwrap(), CMakeGenerator::Makefile);
    /// assert_eq!(CMakeGenerator::from_str("Ninja").unwrap(), CMakeGenerator::Ninja);
    /// assert_eq!(CMakeGenerator::from_str("vs").unwrap(), CMakeGenerator::VisualStudio);
    /// assert_eq!(CMakeGenerator::from_str("VisualStudio").unwrap(), CMakeGenerator::VisualStudio);
    /// assert!(CMakeGenerator::from_str("Unknown").is_err());
    /// ```
    pub fn from_str(generator: &str) -> Result<Self> {
        Ok(match generator.to_ascii_lowercase().as_str() {
            "makefile" => CMakeGenerator::Makefile,
            "ninja" => CMakeGenerator::Ninja,
            "visualstudio" | "vs" => CMakeGenerator::VisualStudio,
            _ => {
                return Err(Error::UnsupportedGenerator {
                    generator: generator.into(),
                });
            }
        })
    }

    fn option(&self) -> Vec<String> {
        match self {
            CMakeGenerator::Platform => Vec::new(),
            CMakeGenerator::Makefile => vec!["-G", "Unix Makefiles"],
            CMakeGenerator::Ninja => vec!["-G", "Ninja"],
            CMakeGenerator::VisualStudio => vec!["-G", "Visual Studio 15 2017"],
            CMakeGenerator::VisualStudioWin64 => {
                vec!["-G", "Visual Studio 15 2017 Win64", "-Thost=x64"]
            }
        }
        .into_iter()
        .map(|s| s.into())
        .collect()
    }

    fn build_option(&self, nproc: usize, build_type: BuildType) -> Vec<String> {
        match self {
            CMakeGenerator::VisualStudioWin64 | CMakeGenerator::VisualStudio => {
                vec!["--config".into(), format!("{:?}", build_type)]
            }
            CMakeGenerator::Platform => Vec::new(),
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
#[derive(Deserialize, Debug, Clone, Copy)]
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
    /// Name of tool (will be downloaded into `tools/{name}` by default)
    pub name: String,
    /// URL for tool. Git/SVN repository or Tar archive are allowed.
    pub url: String,
    /// Git branch (not for SVN)
    pub branch: Option<String>,
    /// Relative install Path (see the example of clang-extra in [module level doc](index.html))
    pub relative_path: Option<String>,
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
    pub generator: CMakeGenerator,
    ///  Option for `CMAKE_BUILD_TYPE`
    #[serde(default)]
    pub build_type: BuildType,
}

/// Describes how to compile LLVM/Clang
///
/// See also [module level document](index.html).
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
            return Err(Error::InvalidEntry {
                name: name.into(),
                message: "One of Path or URL are allowed".into(),
            });
        }
        if let Some(path) = &setting.path {
            if setting.tools.len() > 0 {
                warn!("'tools' must be used with URL, ignored");
            }
            return Ok(Entry::Local {
                name: name.into(),
                path: PathBuf::from(shellexpand::full(&path).unwrap().to_string()),
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
        return Err(Error::InvalidEntry {
            name: name.into(),
            message: "Path nor URL are not found".into(),
        });
    }
}

fn load_entry_toml(toml_str: &str) -> Result<Vec<Entry>> {
    let entries: HashMap<String, EntrySetting> = toml::from_str(toml_str)?;
    entries
        .into_iter()
        .map(|(name, setting)| Entry::parse_setting(&name, setting))
        .collect()
}

pub fn official_releases() -> Vec<Entry> {
    let mut github: Vec<Entry> = [(10, 0, 0), (9, 0, 1), (8, 0, 1)]
        .iter()
        .map(|&(major, minor, patch)| {
            let version = format!("{}.{}.{}", major, minor, patch);
            let mut setting = EntrySetting::default();
            setting.url = Some(format!(
                "https://github.com/llvm/llvm-project/releases/download/llvmorg-{version}/llvm-{version}.src.tar.xz",
                version = version
            ));
            let clang = Tool {
                name: "clang".into(),
                url: format!(
                    "https://github.com/llvm/llvm-project/releases/download/llvmorg-{version}/{clang}-{version}.src.tar.xz",
                    version = version,
                    clang = if major > 8 { "clang" } else { "cfe" }
                ),
                branch: None,
                relative_path: None,
            };
            let lld = Tool {
                name: "lld".into(),
                url: format!(
                    "https://github.com/llvm/llvm-project/releases/download/llvmorg-{version}/lld-{version}.src.tar.xz",
                    version = version
                ),
                branch: None,
                relative_path: None,
            };
            setting.tools = vec![clang, lld];
            Entry::parse_setting(&version, setting).unwrap()
        })
        .collect();

    let mut llvm_org: Vec<Entry> = [
        (9, 0, 0),
        (8, 0, 0),
        (7, 1, 0),
        (7, 0, 1),
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
    .iter()
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
        Entry::parse_setting(&version, setting).unwrap()
    })
    .collect();
    github.append(&mut llvm_org);
    github
}

pub fn load_entries() -> Result<Vec<Entry>> {
    let global_toml = config_dir()?.join(ENTRY_TOML);
    let mut entries = load_entry_toml(&fs::read_to_string(&global_toml).with(&global_toml)?)?;
    let mut official = official_releases();
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
    Err(Error::InvalidEntry {
        message: "Entry not found".into(),
        name: name.into(),
    })
}

impl Entry {
    fn setting(&self) -> &EntrySetting {
        match self {
            Entry::Remote { setting, .. } => setting,
            Entry::Local { setting, .. } => setting,
        }
    }

    fn setting_mut(&mut self) -> &mut EntrySetting {
        match self {
            Entry::Remote { setting, .. } => setting,
            Entry::Local { setting, .. } => setting,
        }
    }

    pub fn set_builder(&mut self, generator: &str) -> Result<()> {
        let generator = CMakeGenerator::from_str(generator)?;
        self.setting_mut().generator = generator;
        Ok(())
    }

    pub fn checkout(&self) -> Result<()> {
        match self {
            Entry::Remote { url, tools, .. } => {
                let src = Resource::from_url(url)?;
                src.download(&self.src_dir()?)?;
                for tool in tools {
                    let path = self.src_dir()?.join(tool.rel_path());
                    let src = Resource::from_url(&tool.url)?;
                    src.download(&path)?;
                }
            }
            Entry::Local { .. } => {}
        }
        Ok(())
    }

    pub fn clean_cache_dir(&self) -> Result<()> {
        let path = self.src_dir()?;
        info!("Remove cache dir: {}", path.display());
        fs::remove_dir_all(&path).with(&path)?;
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
            fs::create_dir_all(&dir).with(&dir)?;
        }
        Ok(dir)
    }

    pub fn clean_build_dir(&self) -> Result<()> {
        let path = self.build_dir()?;
        info!("Remove build dir: {}", path.display());
        fs::remove_dir_all(&path).with(&path)?;
        Ok(())
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
            .args(
                &self
                    .setting()
                    .generator
                    .build_option(nproc, self.setting().build_type),
            )
            .check_run()?;
        Ok(())
    }

    fn configure(&self) -> Result<()> {
        let setting = self.setting();
        let mut opts = setting.generator.option();
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
    fn parse_url() {
        let setting = EntrySetting {
            url: Some("http://llvm.org/svn/llvm-project/llvm/trunk".into()),
            ..Default::default()
        };
        let _entry = Entry::parse_setting("url", setting).unwrap();
    }

    #[test]
    fn parse_path() {
        let setting = EntrySetting {
            path: Some("~/.config/llvmenv".into()),
            ..Default::default()
        };
        let _entry = Entry::parse_setting("path", setting).unwrap();
    }

    #[should_panic]
    #[test]
    fn parse_no_entry() {
        let setting = EntrySetting::default();
        let _entry = Entry::parse_setting("no_entry", setting).unwrap();
    }

    #[should_panic]
    #[test]
    fn parse_duplicated() {
        let setting = EntrySetting {
            url: Some("http://llvm.org/svn/llvm-project/llvm/trunk".into()),
            path: Some("~/.config/llvmenv".into()),
            ..Default::default()
        };
        let _entry = Entry::parse_setting("duplicated", setting).unwrap();
    }

    macro_rules! checkout {
        ($index:expr, $major:expr, $minor:expr, $patch: expr) => {
            paste::item! {
                #[ignore]
                #[test]
                fn [< checkout_ $major _ $minor _ $patch >]() {
                    official_releases()[$index].checkout().unwrap();
                }
            }
        };
    }

    checkout!(0, 10, 0, 0);
    checkout!(1, 9, 0, 1);
    checkout!(2, 8, 0, 1);
    checkout!(3, 9, 0, 0);
    checkout!(4, 8, 0, 0);
    checkout!(5, 7, 1, 0);
    checkout!(6, 7, 0, 1);
    checkout!(7, 7, 0, 0);
    checkout!(8, 6, 0, 1);
    checkout!(9, 6, 0, 0);
    checkout!(10, 5, 0, 2);
    checkout!(11, 5, 0, 1);
    checkout!(12, 4, 0, 1);
    checkout!(13, 4, 0, 0);
    checkout!(14, 3, 9, 1);
    checkout!(15, 3, 9, 0);
}
