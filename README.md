llvmenv
=========

[![crate](https://img.shields.io/crates/v/llvmenv.svg)](https://crates.io/crates/llvmenv)
[![CircleCI](https://circleci.com/gh/termoshtt/llvmenv.svg?style=shield)](https://circleci.com/gh/termoshtt/llvmenv)
[![Azure Pipeline](https://dev.azure.com/termoshtt2/GitHub%20CI/_apis/build/status/termoshtt.llvmenv)](https://dev.azure.com/termoshtt2/GitHub%20CI/_build/latest?definitionId=1)

Manage multiple LLVM/Clang build

```
llvmenv 0.1.6
Manage multiple LLVM/Clang builds

USAGE:
    llvmenv <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    build-entry    Build LLVM/Clang
    builds         List usable build
    current        Show the name of current build
    entries        List entries to be built
    global         Set the build to use (global)
    help           Prints this message or the help of the given subcommand(s)
    init           Initialize llvmenv
    local          Set the build to use (local)
    prefix         Show the prefix of the current build
    zsh            Setup Zsh integration
```

Install
-------
0. Install cmake, make, and g++ (or clang++)
1. Install Rust using [rustup](https://github.com/rust-lang-nursery/rustup.rs)
2. `cargo install llvmenv`

zsh integration
-----
You can swtich LLVM/Clang builds automatically using zsh precmd-hook. Please add a line into your `.zshrc`:

```
source <(llvmenv zsh)
```

If `$LLVMENV_RUST_BINDING` environmental value is non-zero, llvmenv exports `LLVM_SYS_60_PREFIX=$(llvmenv prefix)` in addition to `$PATH`.

```
export LLVMENV_RUST_BINDING=1
source <(llvmenv zsh)
```

This is useful for [llvm-sys.rs](https://github.com/tari/llvm-sys.rs) users. Be sure that this env value will not be unset by llvmenv, only overwrite.

Concepts
=========

entry
------
"entry" descrives how to compile LLVM/Clang, and set by `entry.toml` at `$XDG_CONFIG_HOME/llvmenv` (usually `$HOME/.config/llvmenv`).
`llvmenv init` generates default setting:

```toml
[llvm-dev]
llvm_git  = "https://github.com/llvm-mirror/llvm"
clang_git = "https://github.com/llvm-mirror/clang"
target    = ["X86"]
build     = "Release"
example   = 0
document  = 0
```

There is also pre-defined entries corresponding to the LLVM/Clang releases:

```
$ llvmenv entries
6.0.0
5.0.2
5.0.1
5.0.0
4.0.1
4.0.0
3.9.1
3.9.0
... (your entries)
```

build
------
- "build" is a directory where compiled executables (e.g. clang) and libraries are installed.
- They are compiled by `llvmenv build-entry`, and placed at `$XDG_DATA_HOME/llvmenv` (usually `$HOME/.local/share/llvmenv`).
- There is a special build, "system", which uses system's executables.

global/local/prefix
--------------------
- `llvmenv prefix` returns the path of the current build (e.g. `$XDG_DATA_HOME/llvmenv/llvm-dev`, or `/usr` for system build).
- `llvmenv global [name]` sets default build, and `llvmenv local [name]` sets directory-local build by creating `.llvmenv` text file.
- You can confirm which `.llvmenv` sets the current prefix by `llvmenv prefix -v`.
