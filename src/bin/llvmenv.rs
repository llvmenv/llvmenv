use llvmenv::error::CommandExt;
use llvmenv::*;

use failure::bail;
use std::env;
use std::path::PathBuf;
use std::process::{exit, Command};
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
        #[structopt(short = "u", long = "update")]
        update: bool,
        #[structopt(short = "c", long = "clean")]
        clean: bool,
        #[structopt(short = "j", long = "nproc")]
        nproc: Option<usize>,
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

    #[structopt(name = "edit", about = "Edit llvmenv configure in your editor")]
    Edit {},

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
            if let Ok(entries) = entry::load_entries() {
                for entry in &entries {
                    println!("{}", entry.name());
                }
            } else {
                bail!("No entries. Please define entries in $XDG_CONFIG_HOME/llvmenv/entry.toml");
            }
        }
        LLVMEnv::BuildEntry {
            name,
            update,
            clean,
            nproc,
        } => {
            let entry = entry::load_entry(&name)?;
            let nproc = nproc.unwrap_or(num_cpus::get());
            entry.checkout().unwrap();
            if update {
                entry.update().unwrap();
            }
            if clean {
                entry.clean().unwrap();
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

        LLVMEnv::Edit {} => {
            let editor = env::var("EDITOR").expect("EDITOR environmental value is not set");
            Command::new(editor)
                .arg(config::config_dir()?.join(config::ENTRY_TOML))
                .check_run()?;
        }

        LLVMEnv::Zsh {} => {
            let src = include_str!("../../llvmenv.zsh");
            println!("{}", src);
        }
    }
    Ok(())
}

fn get_existing_build(name: &str) -> build::Build {
    let build = build::Build::from_name(&name).unwrap();
    if build.exists() {
        build
    } else {
        eprintln!("Build '{}' does not exists", name);
        exit(1)
    }
}
