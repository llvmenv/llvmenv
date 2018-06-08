extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate toml;
#[macro_use]
extern crate log;
#[macro_use]
extern crate failure;
extern crate itertools;
#[macro_use]
extern crate structopt;
extern crate env_logger;
extern crate glob;
extern crate num_cpus;
extern crate reqwest;
extern crate tempfile;

pub mod build;
pub mod config;
pub mod entry;
pub mod error;

use std::env;
use std::path::PathBuf;
use std::process::exit;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "llvmenv", about = "Manage multiple LLVM/Clang builds",
            raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
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
        #[structopt(short = "u", long = "update")]
        update: bool,
        #[structopt(short = "j", long = "nproc")]
        nproc: Option<usize>,
        #[structopt(long = "prefix", parse(from_os_str), help = "Overwrite prefix")]
        prefix: Option<PathBuf>,
        #[structopt(long = "build", help = "Overwrite cmake build setting (Debug/Release)")]
        build: Option<String>,
    },

    #[structopt(name = "current", about = "Show the name of current build")]
    Current {
        #[structopt(short = "v", long = "verbose")]
        verbose: bool,
    },
    #[structopt(name = "prefix", about = "Show the prefix of the current build")]
    Prefix {
        #[structopt(short = "v", long = "verbose")]
        verbose: bool,
    },
    #[structopt(name = "version", about = "Show the base version of the current build")]
    Version {
        #[structopt(short = "n", long = "name")]
        name: Option<String>,
        #[structopt(long = "major")]
        major: bool,
        #[structopt(long = "minor")]
        minor: bool,
        #[structopt(long = "patch")]
        patch: bool,
    },

    #[structopt(name = "global", about = "Set the build to use (global)")]
    Global { name: String },
    #[structopt(name = "local", about = "Set the build to use (local)")]
    Local {
        name: String,
        #[structopt(short = "p", long = "path", parse(from_os_str))]
        path: Option<PathBuf>,
    },

    #[structopt(name = "archive", about = "archive build into *.tar.xz (require pixz)")]
    Archive {
        name: String,
        #[structopt(short = "v", long = "verbose")]
        verbose: bool,
    },
    #[structopt(name = "expand", about = "expand archive")]
    Expand {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
        #[structopt(short = "v", long = "verbose")]
        verbose: bool,
    },

    #[structopt(name = "zsh", about = "Setup Zsh integration")]
    Zsh {},
}

fn main() -> error::Result<()> {
    env_logger::init();
    let opt = LLVMEnv::from_args();
    match opt {
        LLVMEnv::Init {} => config::init_config()?,

        LLVMEnv::Builds {} => {
            let builds = build::builds()?;
            let max = builds.iter().map(|b| b.name().len()).max().unwrap();
            for b in &builds {
                println!(
                    "{name:<width$}: {prefix}",
                    name = b.name(),
                    prefix = b.prefix().display(),
                    width = max
                );
            }
        }

        LLVMEnv::Entries {} => {
            let entries = entry::load_entries()?;
            for entry in &entries {
                println!("{}", entry.get_name());
            }
        }
        LLVMEnv::BuildEntry {
            name,
            update,
            nproc,
            prefix,
            build,
        } => {
            let mut entry = entry::load_entry(&name)?;
            if let Some(prefix) = prefix {
                entry.overwrite_prefix(&prefix);
            }
            let nproc = nproc.unwrap_or(num_cpus::get());
            entry.checkout().unwrap();
            if update {
                entry.fetch().unwrap();
            }
            if let Some(build) = build {
                entry.overwrite_build(&build);
            }
            entry.build(nproc).unwrap();
        }

        LLVMEnv::Current { verbose } => {
            let build = build::seek_build()?;
            println!("{}", build.name());
            if verbose {
                if let Some(env) = build.env_path() {
                    eprintln!("set by {}", env.display());
                }
            }
        }
        LLVMEnv::Prefix { verbose } => {
            let build = build::seek_build()?;
            println!("{}", build.prefix().display());
            if verbose {
                if let Some(env) = build.env_path() {
                    eprintln!("set by {}", env.display());
                }
            }
        }
        LLVMEnv::Version {
            name,
            major,
            minor,
            patch,
        } => {
            let build = if let Some(name) = name {
                get_existing_build(&name)
            } else {
                build::seek_build()?
            };
            let (ma, mi, pa) = build.version()?;
            if !(major || minor || patch) {
                println!("{}.{}.{}", ma, mi, pa);
            } else {
                if major {
                    print!("{}", ma);
                }
                if minor {
                    print!("{}", mi);
                }
                if patch {
                    print!("{}", pa);
                }
                println!("");
            }
        }

        LLVMEnv::Global { name } => {
            let build = get_existing_build(&name);
            build.set_global()?;
        }
        LLVMEnv::Local { name, path } => {
            let build = get_existing_build(&name);
            let path = path.unwrap_or(env::current_dir()?);
            build.set_local(&path)?;
        }

        LLVMEnv::Archive { name, verbose } => {
            let build = get_existing_build(&name);
            build.archive(verbose)?;
        }
        LLVMEnv::Expand { path, verbose } => {
            build::expand(&path, verbose)?;
        }

        LLVMEnv::Zsh {} => {
            let src = include_str!("../llvmenv.zsh");
            println!("{}", src);
        }
    }
    Ok(())
}

fn get_existing_build(name: &str) -> build::Build {
    let build = build::Build::from_name(&name);
    if build.exists() {
        build
    } else {
        eprintln!("Build '{}' does not exists", name);
        exit(1)
    }
}
