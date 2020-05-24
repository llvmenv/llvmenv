use llvmenv::error::CommandExt;
use llvmenv::*;

use simplelog::*;
use std::{
    env,
    path::PathBuf,
    process::{exit, Command},
};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "llvmenv",
    about = "Manage multiple LLVM/Clang builds",
    setting = structopt::clap::AppSettings::ColoredHelp
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
        #[structopt(short = "c", long = "clean", help = "clean build directory")]
        clean: bool,
        #[structopt(
            short = "G",
            long = "builder",
            help = "Overwrite cmake generator setting"
        )]
        builder: Option<String>,
        #[structopt(
            short = "d",
            long = "discard",
            help = "discard source directory for remote resources"
        )]
        discard: bool,
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
    TermLogger::init(
        LevelFilter::Info,
        ConfigBuilder::new().set_time_to_local(true).build(),
        TerminalMode::Mixed,
    )
    .unwrap();

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

        LLVMEnv::Entries {} => match entry::load_entries() {
            Ok(entries) => {
                for entry in &entries {
                    println!("{}", entry.name());
                }
            }
            Err(e) => {
                panic!("{}", e);
            }
        },
        LLVMEnv::BuildEntry {
            name,
            update,
            clean,
            discard,
            builder,
            nproc,
        } => {
            let mut entry = entry::load_entry(&name)?;
            let nproc = nproc.unwrap_or_else(num_cpus::get);
            if let Some(builder) = builder {
                entry.set_builder(&builder)?;
            }
            if discard {
                if let Err(e) = entry.clean_cache_dir() {
                    println!("{}", e);
                }
            }
            if let Err(e) = entry.checkout() {
                println!("{}", e);
            };
            if update {
                if let Err(e) = entry.update() {
                    println!("{}", e);
                };
            }
            if clean {
                if let Err(e) = entry.clean_build_dir() {
                    println!("{}", e);
                };
            }
            if let Err(e) = entry.build(nproc) {
                println!("{}", e);
            };
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
                println!();
            }
        }

        LLVMEnv::Global { name } => {
            let build = get_existing_build(&name);
            build.set_global()?;
        }
        LLVMEnv::Local { name, path } => {
            let build = get_existing_build(&name);
            let path = path.unwrap_or_else(|| env::current_dir().unwrap());
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
