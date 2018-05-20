use itertools::Itertools;
use reqwest;
use std::path::{Path, PathBuf};
use std::{fs, process};
use tempfile::NamedTempFile;

use config::*;
use error::*;

/// An entry to be built.
#[derive(Debug, new)]
pub struct Entry {
    name: String,
    llvm: LLVM,
    clang: Clang,
    option: Option<CMakeOption>,
}

#[derive(Deserialize, Debug)]
pub struct CMakeOption {
    build: Option<String>,
    target: Option<Vec<String>>,
    example: Option<bool>,
    document: Option<bool>,
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
        if let Some(ref option) = self.option {
            opts.push(format!(
                "-DCMAKE_BUILD_TYPE={}",
                option.build.as_ref().unwrap_or(&"Release".into())
            ));
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
            opts.push(format!("-DLLVM_INCLUDE_TEST=0"));
            opts.push(format!("-DCLANG_INCLUDE_TEST=0"));
        } else {
            opts.push(format!("-DCMAKE_BUILD_TYPE=Release",));
        }
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
    LLVM_RELEASES.iter().map(|(ma,mi,p)| {
        let name= format!("{}.{}.{}", ma, mi, p);
        let llvm_url = format!("http://releases.llvm.org/{name}/llvm-{name}.src.tar.xz", name=name);
        let clang_url = format!("http://releases.llvm.org/{name}/cfe-{name}.src.tar.xz", name=name);
        Entry {
            name,
            llvm: LLVM::Tar(llvm_url),
            clang: Clang::Tar(clang_url),
            option: None
        }
    }).collect()
}
