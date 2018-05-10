use failure::err_msg;
use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};
use toml;

use build::*;
use error::Result;

pub const APP_NAME: &'static str = "llvmenv";
pub const ENTRY_TOML: &'static str = "entry.toml";

const DEFAULT_ENTRY: &'static [u8] = br#"
[llvm-dev]
llvm_git = "https://github.com/llvm-mirror/llvm"
clang_git = "https://github.com/llvm-mirror/clang"

[llvm-dev.option]
target   = ["X86"]
example  = false
document = false
"#;

pub fn config_dir() -> PathBuf {
    let home = match env::var("XDG_CONFIG_HOME") {
        Ok(path) => path.into(),
        Err(_) => env::home_dir()
            .expect("$HOME does not found")
            .join(".config"), // Use $HOME/.config
    };
    home.join(APP_NAME)
}

pub fn cache_dir() -> PathBuf {
    let home = match env::var("XDG_CACHE_HOME") {
        Ok(path) => path.into(),
        Err(_) => env::home_dir()
            .expect("$HOME does not found")
            .join(".cache"), // Use $HOME/.cache
    };
    home.join(APP_NAME)
}

pub fn data_dir() -> PathBuf {
    let home = match env::var("XDG_DATA_HOME") {
        Ok(path) => path.into(),
        Err(_) => env::home_dir()
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

#[derive(Deserialize, Debug)]
struct EntryParam {
    llvm_git: Option<String>,
    llvm_svn: Option<String>,
    clang_git: Option<String>,
    clang_svn: Option<String>,
    option: Option<BuildOption>,
}

#[derive(Debug, Fail)]
enum ParseError {
    #[fail(display = "Duplicate LLVM in entry '{}': svn={}, git={}", name, svn, git)]
    DuplicateLLVM {
        name: String,
        svn: String,
        git: String,
    },
    #[fail(display = "No LLVM in entry '{}'", name)]
    NoLLVM { name: String },
    #[fail(display = "Duplicate Clang in entry '{}': svn={}, git={}", name, svn, git)]
    DuplicateClang {
        name: String,
        svn: String,
        git: String,
    },
}

type TOMLData = HashMap<String, EntryParam>;

fn load_toml() -> Result<TOMLData> {
    let toml = config_dir().join(ENTRY_TOML);
    let mut f = fs::File::open(toml)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    let data = toml::from_str(&s)?;
    Ok(data)
}

fn convert(name: &str, entry: EntryParam) -> Result<Entry> {
    let name = name.into();
    let llvm = if let Some(svn) = entry.llvm_svn {
        if let Some(git) = entry.llvm_git {
            return Err(ParseError::DuplicateLLVM { name, svn, git }.into());
        } else {
            LLVM::SVN(svn)
        }
    } else {
        if let Some(git) = entry.llvm_git {
            LLVM::Git(git)
        } else {
            return Err(ParseError::NoLLVM { name }.into());
        }
    };

    let clang = if let Some(svn) = entry.clang_svn {
        if let Some(git) = entry.clang_git {
            return Err(ParseError::DuplicateClang { name, svn, git }.into());
        } else {
            Clang::SVN(svn)
        }
    } else {
        if let Some(git) = entry.clang_git {
            Clang::Git(git)
        } else {
            Clang::None
        }
    };

    Ok(Entry::new(name, llvm, clang, entry.option))
}

pub fn load_entry(name: &str) -> Result<Entry> {
    let mut data = load_toml()?;
    if let Some(param) = data.remove(name) {
        Ok(convert(name, param)?)
    } else {
        Err(err_msg(format!("Not found: {}", name)))
    }
}

pub fn load_entries() -> Result<Vec<Entry>> {
    let data = load_toml()?;
    let mut entries = Vec::new();
    for (k, v) in data.into_iter() {
        entries.push(convert(&k, v)?);
    }
    Ok(entries)
}
