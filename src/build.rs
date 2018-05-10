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
    git: String,
    option: Option<BuildOption>,
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

    pub fn fetch(&self) -> CommandResult {
        let src = self.src_dir();
        if !src.exists() {
            process::Command::new("git")
                .args(&["clone", &self.git])
                .arg(src)
                .check_run()?;
        }
        process::Command::new("git").arg("pull").check_run()
    }

    // Prepare build (create dir, run cmake)
    pub fn prebuild(&self) {
        //
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
    git: String,
    option: Option<BuildOption>,
}

type TOMLData = HashMap<String, EntryParam>;

pub fn load_entries() -> Result<Vec<Entry>> {
    let toml = config_dir().join(ENTRY_TOML);
    let mut f = fs::File::open(toml)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;
    let data: TOMLData = toml::from_str(&s)?;
    Ok(data.into_iter()
        .map(|(k, v)| Entry {
            name: k.into(),
            git: v.git,
            option: v.option,
        })
        .collect())
}
