//! Manage entries, i.e. LLVM/Clang source to be built

use failure::err_msg;
use itertools::Itertools;
use reqwest;
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{fs, process};
use tempfile::NamedTempFile;
use toml;

use config::*;
use error::*;

/// An entry to be built.
#[derive(Debug)]
pub struct Entry {
    name: String,
    llvm: LLVM,
    clang: Clang,
    build: String,
    target: Vec<String>, // empty means all target
    example: u32,
    document: u32,
}

pub type URL = String;
pub type Branch = String;

#[derive(Debug)]
pub enum LLVM {
    SVN(URL, Branch),
    Git(URL, Branch),
    Tar(URL),
}

#[derive(Debug)]
pub enum Clang {
    SVN(URL, Branch),
    Git(URL, Branch),
    Tar(URL),
    None,
}

impl Entry {
    fn default_option(name: String, llvm: LLVM, clang: Clang) -> Self {
        Entry {
            name,
            llvm,
            clang,
            build: "Release".into(),
            target: vec!["X86".into()],
            example: 0,
            document: 0,
        }
    }
    fn src_dir(&self) -> PathBuf {
        cache_dir().join(&self.name)
    }

    fn build_dir(&self) -> PathBuf {
        self.src_dir().join("build")
    }

    pub fn prefix(&self) -> PathBuf {
        data_dir().join(&self.name)
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn checkout(&self) -> Result<()> {
        if !cache_dir().exists() {
            fs::create_dir_all(cache_dir())?;
        }
        let src = self.src_dir();
        if !src.exists() {
            // clone/checkout
            match self.llvm {
                LLVM::SVN(ref url, ref _branch) => {
                    process::Command::new("svn")  // TODO support branch in SVN
                        .args(&["co", url.as_str()])
                        .arg(&self.name)
                        .current_dir(cache_dir())
                        .check_run()?
                }
                LLVM::Git(ref url, ref branch) => {
                    process::Command::new("git")
                        .args(&["clone", url.as_str()])
                        .args(&["-b", branch])
                        .arg(&self.name)
                        .current_dir(cache_dir())
                        .check_run()?;
                }
                LLVM::Tar(ref url) => {
                    let tmp = download_tmp(url)?;
                    expand_tar(&tmp, &cache_dir())?;
                }
            }
        }
        let tools = src.join("tools");
        let clang = tools.join("clang");
        if !clang.exists() {
            match self.clang {
                Clang::SVN(ref url, ref _branch) => {
                    process::Command::new("svn") // TODO support branch in SVN
                        .args(&["co", url.as_str(), "clang"])
                        .current_dir(tools)
                        .check_run()?
                }
                Clang::Git(ref url, ref branch) => {
                    process::Command::new("git")
                        .args(&["clone", url.as_str(), "clang"])
                        .args(&["-b", branch])
                        .current_dir(tools)
                        .check_run()?;
                }
                Clang::Tar(ref url) => {
                    let tmp = download_tmp(url)?;
                    expand_tar(&tmp, &cache_dir())?;
                }
                Clang::None => info!("No clang."),
            }
        }
        Ok(())
    }

    pub fn fetch(&self) -> Result<()> {
        let src = self.src_dir();
        if !src.exists() {
            match self.llvm {
                LLVM::SVN(_, _) => process::Command::new("svn")
                    .arg("update")
                    .current_dir(self.src_dir())
                    .check_run()?,
                LLVM::Git(_, _) => process::Command::new("git")
                    .arg("pull")
                    .current_dir(self.src_dir())
                    .check_run()?,
                LLVM::Tar(_) => {}
            };
        }
        let tools = src.join("tools");
        let clang = tools.join("clang");
        if !clang.exists() {
            match self.clang {
                Clang::SVN(_, _) => process::Command::new("svn")
                    .arg("update")
                    .current_dir(clang)
                    .check_run()?,
                Clang::Git(_, _) => process::Command::new("git")
                    .arg("pull")
                    .current_dir(clang)
                    .check_run()?,
                Clang::Tar(_) | Clang::None => {}
            };
        }
        Ok(())
    }

    pub fn build(&self, nproc: usize) -> Result<()> {
        let build = self.build_dir();
        if !build.exists() {
            fs::create_dir_all(&build)?;
        }
        let mut opts = Vec::new();
        opts.push(format!(
            "-DCMAKE_INSTALL_PREFIX={}",
            self.prefix().display()
        ));
        opts.push(format!("-DCMAKE_BUILD_TYPE={}", self.build));
        if self.target.len() > 0 {
            opts.push(format!(
                "-DLLVM_TARGETS_TO_BUILD={}",
                self.target.iter().join(";")
            ));
        }
        opts.push(format!("-DLLVM_INCLUDE_EXAMPLES={}", self.example));
        opts.push(format!("-DCLANG_INCLUDE_EXAMPLES={}", self.example));
        opts.push(format!("-DLLVM_INCLUDE_DOCS={}", self.document));
        opts.push(format!("-DCLANG_INCLUDE_DOCS={}", self.document));
        opts.push(format!("-DLLVM_INCLUDE_TEST=0"));
        opts.push(format!("-DCLANG_INCLUDE_TEST=0"));
        process::Command::new("cmake")
            .args(&opts)
            .arg(self.src_dir())
            .current_dir(&build)
            .check_run()?;

        process::Command::new("make")
            .arg(format!("-j{}", nproc))
            .current_dir(&build)
            .check_run()?;

        process::Command::new("make")
            .arg("install")
            .current_dir(&build)
            .check_run()?;
        Ok(())
    }
}

fn download_tmp(url: &URL) -> Result<PathBuf> {
    let mut tmp = NamedTempFile::new()?;
    let mut req = reqwest::get(url)?;
    req.copy_to(&mut tmp)?;
    Ok(tmp.path().into())
}

fn expand_tar(tar_path: &Path, out_path: &Path) -> Result<()> {
    process::Command::new("tar")
        .arg("x")
        .arg(tar_path)
        .current_dir(out_path)
        .check_run()?;
    Ok(())
}

const LLVM_RELEASES: [(u32, u32, u32); 8] = [
    (6, 0, 0),
    (5, 0, 2),
    (5, 0, 1),
    (5, 0, 0),
    (4, 0, 1),
    (4, 0, 0),
    (3, 9, 1),
    (3, 9, 0),
]; // XXX should we support more old versions?

pub fn releases() -> Vec<Entry> {
    LLVM_RELEASES
        .iter()
        .map(|(ma, mi, p)| {
            let name = format!("{}.{}.{}", ma, mi, p);
            let llvm_url = format!(
                "http://releases.llvm.org/{name}/llvm-{name}.src.tar.xz",
                name = name
            );
            let clang_url = format!(
                "http://releases.llvm.org/{name}/cfe-{name}.src.tar.xz",
                name = name
            );
            Entry::default_option(name, LLVM::Tar(llvm_url), Clang::Tar(clang_url))
        })
        .collect()
}

#[derive(Deserialize, Debug)]
struct EntryParam {
    llvm_git: Option<String>,
    llvm_svn: Option<String>,
    clang_git: Option<String>,
    clang_svn: Option<String>,
    llvm_branch: Option<String>,
    clang_branch: Option<String>,
    build: Option<String>,
    target: Option<Vec<String>>,
    example: Option<u32>,
    document: Option<u32>,
}

impl EntryParam {
    fn convert(self, name: &str) -> Result<Entry> {
        let name = name.into();
        let llvm = if let Some(svn) = self.llvm_svn {
            if let Some(git) = self.llvm_git {
                return Err(ParseError::DuplicateLLVM { name, svn, git }.into());
            } else {
                LLVM::SVN(svn, self.llvm_branch.unwrap_or("trunk".into()))
            }
        } else {
            if let Some(git) = self.llvm_git {
                LLVM::Git(git, self.llvm_branch.unwrap_or("master".into()))
            } else {
                return Err(ParseError::NoLLVM { name }.into());
            }
        };

        let clang = if let Some(svn) = self.clang_svn {
            if let Some(git) = self.clang_git {
                return Err(ParseError::DuplicateClang { name, svn, git }.into());
            } else {
                Clang::SVN(svn, self.clang_branch.unwrap_or("trunk".into()))
            }
        } else {
            if let Some(git) = self.clang_git {
                Clang::Git(git, self.clang_branch.unwrap_or("master".into()))
            } else {
                Clang::None
            }
        };

        let mut entry = Entry::default_option(name, llvm, clang);
        if let Some(ref build) = self.build {
            entry.build = build.clone();
        }
        if let Some(ref target) = self.target {
            entry.target = target.clone();
        }
        if let Some(ref example) = self.example {
            entry.example = *example;
        }
        if let Some(ref document) = self.document {
            entry.document = *document;
        }
        Ok(entry)
    }
}

#[derive(Debug, Fail)]
pub enum ParseError {
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

pub fn load_entry(name: &str) -> Result<Entry> {
    let mut data = load_toml()?;
    if let Some(param) = data.remove(name) {
        Ok(param.convert(name)?)
    } else {
        Err(err_msg(format!("Not found: {}", name)))
    }
}

pub fn load_entries() -> Result<Vec<Entry>> {
    let data = load_toml()?;
    let mut entries = Vec::new();
    for (k, v) in data.into_iter() {
        entries.push(v.convert(&k)?);
    }
    Ok(entries)
}
