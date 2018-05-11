use glob::glob;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{env, fs};

use config::*;
use error::*;

const LLVMENV_FN: &'static str = ".llvmenv";

#[derive(Debug)]
pub struct Build {
    pub name: String,
    pub prefix: PathBuf,
    pub llvmenv: Option<PathBuf>,
}

impl Build {
    fn system() -> Self {
        Build {
            name: "system".into(),
            prefix: PathBuf::from("/usr"),
            llvmenv: None,
        }
    }

    fn from_path(path: &Path) -> Self {
        let name = path.file_name().unwrap().to_str().unwrap();
        Build {
            name: name.into(),
            prefix: path.to_owned(),
            llvmenv: None,
        }
    }

    fn from_name(name: &str) -> Self {
        Build {
            name: name.into(),
            prefix: data_dir().join(name),
            llvmenv: None,
        }
    }

    fn exists(&self) -> bool {
        self.prefix.is_dir()
    }
}

fn local_builds() -> Result<Vec<Build>> {
    Ok(glob(&format!("{}/*/bin", data_dir().display()))?
        .filter_map(|path| {
            if let Ok(path) = path {
                path.parent().map(|path| Build::from_path(path))
            } else {
                None
            }
        })
        .collect())
}

pub fn builds() -> Result<Vec<Build>> {
    let mut bs = local_builds()?;
    bs.sort_by(|a, b| a.name.cmp(&b.name));
    bs.insert(0, Build::system());
    Ok(bs)
}

fn load_llvmenv(path: &Path) -> Result<Option<Build>> {
    let cand = path.join(LLVMENV_FN);
    if !cand.exists() {
        return Ok(None);
    }
    let mut f = fs::File::open(cand)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    let name = s.trim();
    if name == "system" {
        return Ok(Some(Build::system()));
    }
    let mut build = Build::from_name(name);
    if build.exists() {
        build.llvmenv = Some(path.into());
        Ok(Some(build))
    } else {
        Ok(None)
    }
}

pub fn seek_build() -> Result<Build> {
    // Seek .llvmenv from $PWD
    let mut path = env::current_dir()?;
    loop {
        if let Some(mut build) = load_llvmenv(&path)? {
            build.llvmenv = Some(path.join(LLVMENV_FN));
            return Ok(build);
        }
        path = match path.parent() {
            Some(path) => path.into(),
            None => break,
        };
    }
    // check global setting
    if let Some(mut build) = load_llvmenv(&config_dir())? {
        build.llvmenv = Some(config_dir().join(LLVMENV_FN));
        return Ok(build);
    }
    Ok(Build::system())
}
