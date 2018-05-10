use itertools::Itertools;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::{fs, process};
use toml;

use config::*;
use error::*;

#[derive(Debug)]
pub struct Entry {
    name: String,
    llvm: LLVM,
    clang: Clang,
    option: Option<BuildOption>,
}

type URL = String;

#[derive(Debug)]
enum LLVM {
    SVN(URL),
    Git(URL),
}

#[derive(Debug)]
enum Clang {
    SVN(URL),
    Git(URL),
    None,
}

impl Entry {
    fn src_dir(&self) -> PathBuf {
        cache_dir().join(&self.name)
    }

    fn build_dir(&self) -> PathBuf {
        self.src_dir().join("build")
    }

    fn prefix(&self) -> PathBuf {
        data_dir().join(&self.name)
    }

    pub fn clone(&self) -> Result<()> {
        let src = self.src_dir();
        if !src.exists() {
            match self.llvm {
                LLVM::SVN(ref url) => process::Command::new("svn")
                    .args(&["co", url.as_str()])
                    .arg(&self.name)
                    .current_dir(cache_dir())
                    .check_run()?,
                LLVM::Git(ref url) => process::Command::new("git")
                    .args(&["clone", url.as_str()])
                    .arg(&self.name)
                    .current_dir(cache_dir())
                    .check_run()?,
            }
        } else {
            warn!("Already exists: {}", src.display());
        }
        let tools = src.join("tools");
        let clang = tools.join("clang");
        if !clang.exists() {
            match self.clang {
                Clang::SVN(ref url) => process::Command::new("svn")
                    .args(&["co", url.as_str(), "clang"])
                    .current_dir(tools)
                    .check_run()?,
                Clang::Git(ref url) => process::Command::new("git")
                    .args(&["clone", url.as_str(), "clang"])
                    .current_dir(tools)
                    .check_run()?,
                Clang::None => info!("No clang."),
            }
        } else {
            warn!("Already exists: {}", self.src_dir().display());
        }
        Ok(())
    }

    pub fn fetch(&self) -> Result<()> {
        Ok(())
    }

    // Prepare build (create dir, run cmake)
    pub fn prebuild(&self) -> Result<()> {
        let build = self.build_dir();
        if !build.exists() {
            fs::create_dir_all(&build)?;
        }
        let mut opts = Vec::new();
        opts.push(format!(
            "-DCMAKE_INSTALL_PREFIX={}",
            self.prefix().display()
        ));
        if let Some(ref option) = self.option {
            if let Some(ref target) = option.target {
                opts.push(format!(
                    "-DLLVM_TARGETS_TO_BUILD={}",
                    target.iter().join(";")
                ));
            }
            if let Some(ref example) = option.example {
                let ex = if *example { 1 } else { 0 };
                opts.push(format!("-DLLVM_INCLUDE_EXAMPLES={}", ex));
                opts.push(format!("-DCLANG_INCLUDE_EXAMPLES={}", ex));
            }
            if let Some(ref document) = option.example {
                let ex = if *document { 1 } else { 0 };
                opts.push(format!("-DLLVM_INCLUDE_DOCS={}", ex));
                opts.push(format!("-DCLANG_INCLUDE_DOCS={}", ex));
            }
        }
        process::Command::new("cmake")
            .args(&opts)
            .arg(self.src_dir())
            .current_dir(build)
            .check_run()?;
        Ok(())
    }
}

#[derive(Deserialize, Debug)]
struct BuildOption {
    subdir: Option<String>,
    build_path: Option<String>,
    target: Option<Vec<String>>,
    example: Option<bool>,
    document: Option<bool>,
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

pub fn load_entries() -> Result<Vec<Entry>> {
    let toml = config_dir().join(ENTRY_TOML);
    let mut f = fs::File::open(toml)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    let data: TOMLData = toml::from_str(&s)?;
    let mut entries = Vec::new();
    for (k, v) in data.into_iter() {
        let name = k.into();
        let llvm = if let Some(svn) = v.llvm_svn {
            if let Some(git) = v.llvm_git {
                return Err(ParseError::DuplicateLLVM { name, svn, git }.into());
            } else {
                LLVM::SVN(svn)
            }
        } else {
            if let Some(git) = v.llvm_git {
                LLVM::Git(git)
            } else {
                return Err(ParseError::NoLLVM { name }.into());
            }
        };

        let clang = if let Some(svn) = v.clang_svn {
            if let Some(git) = v.clang_git {
                return Err(ParseError::DuplicateClang { name, svn, git }.into());
            } else {
                Clang::SVN(svn)
            }
        } else {
            if let Some(git) = v.clang_git {
                Clang::Git(git)
            } else {
                Clang::None
            }
        };

        entries.push(Entry {
            name,
            llvm,
            clang,
            option: v.option,
        });
    }
    Ok(entries)
}
