llvmenv
=========

[![crate](https://img.shields.io/crates/v/llvmenv.svg)](https://crates.io/crates/llvmenv)
[![docs.rs](https://docs.rs/llvmenv/badge.svg)](https://docs.rs/llvmenv)

Manage multiple LLVM/Clang build

Install
-------
0. Install cmake, builder (make/ninja), and C++ compiler (g++/clang++)
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
- **entry** describes how to compile LLVM/Clang
- Two types of entries
  - *Remote*: Download LLVM from Git/SVN repository or Tar archive, and then build
  - *Local*: Build locally cloned LLVM source
- See [the module document](https://docs.rs/llvmenv/*/llvmenv/entry/index.html) for detail

build
------
- **build** is a directory where compiled executables (e.g. clang) and libraries are installed.
- They are compiled by `llvmenv build-entry`, and placed at `$XDG_DATA_HOME/llvmenv` (usually `$HOME/.local/share/llvmenv`).
- There is a special build, "system", which uses system's executables.

global/local prefix
--------------------
- `llvmenv prefix` returns the path of the current build (e.g. `$XDG_DATA_HOME/llvmenv/llvm-dev`, or `/usr` for system build).
- `llvmenv global [name]` sets default build, and `llvmenv local [name]` sets directory-local build by creating `.llvmenv` text file.
- You can confirm which `.llvmenv` sets the current prefix by `llvmenv prefix -v`.

Examples
=========

Intialize llvmenv and create default configuration and directories
---
```shell
$ llvmenv init
```

Viewing available entries to build:
---
```shell
$ llvmenv entries
llvm-project
10.0.0
9.0.1
8.0.1
9.0.0
8.0.0
7.1.0
7.0.1
7.0.0
6.0.1
6.0.0
5.0.2
5.0.1
4.0.1
4.0.0
3.9.1
3.9.0
```

Building an entry
---
```shell
$ llvmenv build-entry 10.0.0
10:14:40 [ INFO] Download Tar file: https://github.com/llvm/llvm-project/archive/llvmorg-10.0.0.tar.gz
10:15:07 [ INFO] Create build dir: /home/alberto/.cache/llvmenv/10.0.0/build
-- The C compiler identification is GNU 9.3.0
-- The CXX compiler identification is GNU 9.3.0
-- The ASM compiler identification is GNU
-- Found assembler: /usr/bin/cc
-- Check for working C compiler: /usr/bin/cc
...
```