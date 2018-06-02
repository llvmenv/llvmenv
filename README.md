llvmenv
=========

[![crate](https://img.shields.io/crates/v/llvmenv.svg)](https://crates.io/crates/llvmenv)
[![CircleCI](https://circleci.com/gh/termoshtt/llvmenv.svg?style=shield)](https://circleci.com/gh/termoshtt/llvmenv)

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

zsh integration
-----

```
source <(llvmenv zsh)
```

in your `.zshrc`. Other shell support is WIP.

entry
------
"entry" descrives how to compile LLVM/Clang, and set by `entry.toml` at `$XDG_CONFIG_HOME/llvmenv` (usually `$HOME/.config/llvmenv`).
`llvmenv init` generates default setting:

```toml
[llvm-dev]
llvm_git = "https://github.com/llvm-mirror/llvm"
clang_git = "https://github.com/llvm-mirror/clang"
target   = ["X86"]
example  = 0
document = 0
```

build
------
"build" is a directory where compiled executables (e.g. clang) and libraries are installed.
Builds are compiled by `llvmenv build-entry`, and placed at `$XDG_DATA_HOME/llvmenv` (usually `$HOME/.local/share/llvmenv`).
There is a special build "system", which uses system's executables.

global/local/prefix
--------------------
`llvmenv prefix` returns the PATH for executables in build (e.g. `$XDG_DATA_HOME/llvmenv/llvm-dev`, or `/usr` for system build).
`llvmenv global [name]` sets default build, and `llvmenv local [name]` sets directory-local build by creating `.llvmenv` text file.
You can confirm which `.llvmenv` sets the current prefix by `llvmenv prefix -v`.
