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

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "llvmenv",
    about = "Manage multi LLVM builds",
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
    Prefix { name: String },
    #[structopt(name = "global", about = "Set the build to use (global)")]
    Global { name: String },
    #[structopt(name = "local", about = "Set the build to use (local)")]
    Local { name: String },
}

fn main() {
    let opt = LLVMEnv::from_args();
    match opt {
        LLVMEnv::Init {} => config::init_config().expect("Failed to initailzie"),

        LLVMEnv::Builds {} => {
            let builds = build::builds().expect("Failed to load builds");
            for b in &builds {
                println!("{}: {}", b.name(), b.prefix().display());
            }
        }

        LLVMEnv::Entries {} => {
            let entries = config::load_entries().expect("Failed to load entries");
            for entry in &entries {
                println!("{}", entry.get_name());
            }
        }
        LLVMEnv::BuildEntry { name, nproc } => {
            let entry = config::load_entry(&name).expect("Failed to load entries");
            let nproc = nproc.unwrap_or(num_cpus::get());
            entry.checkout().unwrap();
            entry.build(nproc).unwrap();
        }

        _ => {
            unimplemented!("opt = {:?}", opt);
        }
    }
}
