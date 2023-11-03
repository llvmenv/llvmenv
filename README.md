:sos: This project is only somewhat active due to the low number of active maintainers. If you wish to participate, see [#72](https://github.com/llvmenv/llvmenv/issues/72).

llvmenv
=========

[![crate](https://img.shields.io/crates/v/llvmenv.svg)](https://crates.io/crates/llvmenv)
[![docs.rs](https://docs.rs/llvmenv/badge.svg)](https://docs.rs/llvmenv)

Manage multiple LLVM/Clang build

Install
-------

0. Install cmake, builder (make/ninja), and C++ compiler (g++/clang++)
1. Install Rust using [rustup](https://github.com/rust-lang-nursery/rustup.rs) or any other method.  The minimum supported Rust version is currently **1.48.0**.
2. `cargo install llvmenv`

### Basic Usage

To install a specific version of LLVM after following the installation steps above, run these shell commands ("10.0.0" can be replaced with any other version found with `llvmenv entries`):

```
llvmenv init
llvmenv entries
llvmenv build-entry 10.0.0
```

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

FQA
=========

How can I continue a build when I've got an error?

The builds are placed under `$XDG_DATA_HOME/llvmenv` (usually `$HOME/.local/share/llvmenv`). You can go to the folder `build` and run `make` to retry the build without recompiling from the start.
