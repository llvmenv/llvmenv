extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
extern crate itertools;
#[macro_use]
extern crate structopt;
extern crate glob;
extern crate num_cpus;

pub mod build;
pub mod config;
pub mod entry;
pub mod error;

use failure::err_msg;
use std::env;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "llvmenv",
    about = "Manage multiple LLVM/Clang builds",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
enum LLVMEnv {
    #[structopt(name = "init", about = "Initialize llvmenv")]
    Init {},

    #[structopt(name = "builds", about = "List usable build")]
    Builds {},

    #[structopt(name = "entries", about = "List entries to be built")]
    Entries {},
    #[structopt(name = "build-entry", about = "Build LLVM/Clang")]
    BuildEntry {
        name: String,
        #[structopt(short = "j")]
        nproc: Option<usize>,
    },

    #[structopt(name = "prefix", about = "Show the prefix of the current build")]
    Prefix {},

    #[structopt(name = "global", about = "Set the build to use (global)")]
    Global { name: String },
    #[structopt(name = "local", about = "Set the build to use (local)")]
    Local {
        name: String,
        #[structopt(short = "p", long = "path", parse(from_os_str))]
        path: Option<PathBuf>,
    },
}

fn main() -> error::Result<()> {
    let opt = LLVMEnv::from_args();
    match opt {
        LLVMEnv::Init {} => config::init_config()?,

        LLVMEnv::Builds {} => {
            let builds = build::builds()?;
            for b in &builds {
                println!("{}: {}", b.name(), b.prefix().display());
            }
        }

        LLVMEnv::Entries {} => {
            let entries = config::load_entries()?;
            for entry in &entries {
                println!("{}", entry.get_name());
            }
        }
        LLVMEnv::BuildEntry { name, nproc } => {
            let entry = config::load_entry(&name)?;
            let nproc = nproc.unwrap_or(num_cpus::get());
            entry.checkout().unwrap();
            entry.build(nproc).unwrap();
        }

        LLVMEnv::Prefix {} => {
            let build = build::seek_build()?;
            println!("{}", build.prefix().display());
            if let Some(env) = build.env_path() {
                info!("Set by {}", env.display());
            }
        }

        LLVMEnv::Global { name } => {
            let build = build::Build::from_name(&name);
            if build.exists() {
                build.set_global()?;
            } else {
                return Err(err_msg(format!("Build '{}' does not exists", name)));
            }
        }
        LLVMEnv::Local { name, path } => {
            let build = build::Build::from_name(&name);
            let path = path.unwrap_or(env::current_dir()?);
            if build.exists() {
                build.set_local(&path)?;
            } else {
                return Err(err_msg(format!("Build '{}' does not exists", name)));
            }
        }
    }
    Ok(())
}
