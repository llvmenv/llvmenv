use glob::glob;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::{env, fs};

use config::data_dir;
use error::*;

const LLVMENV_FN: &'static str = ".llvmenv";

pub trait Build {
    fn name(&self) -> &str;
    fn prefix(&self) -> &Path;
    fn exists(&self) -> bool {
        self.prefix().join("bin").is_dir()
    }
}

pub struct System {}

impl System {
    fn boxed() -> Box<Build> {
        Box::new(System {}) as _
    }
}

impl Build for System {
    fn name(&self) -> &str {
        "system"
    }

    fn prefix(&self) -> &Path {
        Path::new("/usr")
    }
}

#[derive(new)]
pub struct LocalBuild {
    path: PathBuf,
}

impl Build for LocalBuild {
    fn name(&self) -> &str {
        self.path.file_name().unwrap().to_str().unwrap()
    }

    fn prefix(&self) -> &Path {
        &self.path
    }
}

// Seek .llvmenv from $PWD
fn seek_local() -> Result<Option<String>> {
    let mut path = env::current_dir()?;
    loop {
        let cand = path.join(LLVMENV_FN);
        if cand.exists() {
            let mut f = fs::File::open(cand)?;
            let mut s = String::new();
            f.read_to_string(&mut s)?;
            return Ok(Some(s));
        }
        path = match path.parent() {
            Some(path) => path.into(),
            None => return Ok(None),
        };
    }
}

fn local_builds() -> Result<Vec<LocalBuild>> {
    Ok(glob(&format!("{}/*/bin", data_dir().display()))?
        .filter_map(|path| {
            if let Ok(path) = path {
                path.parent().map(|path| LocalBuild::new(path.to_owned()))
            } else {
                None
            }
        })
        .collect())
}

pub fn builds() -> Result<Vec<Box<Build>>> {
    let mut bs: Vec<_> = local_builds()?
        .into_iter()
        .map(|b| Box::new(b) as Box<Build>)
        .collect();
    bs.sort_by(|a, b| a.name().cmp(b.name()));
    bs.insert(0, System::boxed());
    Ok(bs)
}
