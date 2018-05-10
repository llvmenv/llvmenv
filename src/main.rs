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

mod build;
mod config;
mod error;

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
    #[structopt(name = "list", about = "List usable builds")]
    List {},
    #[structopt(name = "prefix", about = "Show the prefix of the current build")]
    Prefix { name: String },
    #[structopt(name = "build", about = "Build LLVM/Clang")]
    Build { name: String },
    #[structopt(name = "global", about = "Set the build to use (global)")]
    Global { name: String },
    #[structopt(name = "local", about = "Set the build to use (local)")]
    Local { name: String },
}

fn main() {
    let opt = LLVMEnv::from_args();
    match opt {
        LLVMEnv::Init {} => config::init_config().expect("Failed to initailzie"),
        LLVMEnv::List {} => {
            let entries = config::load_entries().expect("Failed to load entries");
            for entry in &entries {
                println!("{}", entry.get_name());
            }
        }
        LLVMEnv::Build { name } => {
            let entry = config::load_entry(&name).expect("Failed to load entries");
            entry.checkout().unwrap();
            entry.build().unwrap();
        }
        _ => {
            unimplemented!("opt = {:?}", opt);
        }
    }
}
