llvmenv
=========

[![crate](https://img.shields.io/crates/v/llvmenv.svg)](https://crates.io/crates/llvmenv)
[![docs.rs](https://docs.rs/llvmenv/badge.svg)](https://docs.rs/llvmenv)
[![CircleCI](https://circleci.com/gh/termoshtt/llvmenv.svg?style=shield)](https://circleci.com/gh/termoshtt/llvmenv)
[![Azure Pipeline](https://dev.azure.com/termoshtt2/GitHub%20CI/_apis/build/status/termoshtt.llvmenv)](https://dev.azure.com/termoshtt2/GitHub%20CI/_build/latest?definitionId=1)

Manage multiple LLVM/Clang build

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
"entry" describes how to compile LLVM/Clang, and set by `entry.toml` at `$XDG_CONFIG_HOME/llvmenv` (usually `$HOME/.config/llvmenv`).
`llvmenv init` generates default setting:

```toml
[llvm-mirror]
url    = "https://github.com/llvm-mirror/llvm"
target = ["X86"]

[[llvm-mirror.tools]]
name = "clang"
url = "https://github.com/llvm-mirror/clang"

[[llvm-mirror.tools]]
name = "clang-extra"
url = "https://github.com/llvm-mirror/clang-tools-extra"
relative_path = "tools/clang/tools/extra"
```

(TOML format has been changed largely at version 0.2.0)
`tools` means LLVM tools, e.g. clang, compiler-rt, lld, and so on.
These will be downloaded into `${llvm-top}/tools/${tool-name}` by default,
and `relative_path` property change it.
This toml will be decoded into [llvmenv::entry::EntrySetting][EntrySetting].

[EntrySetting]: https://docs.rs/llvmenv/0.2.1/llvmenv/entry/struct.EntrySetting.html

### Pre-defined entries

There is also pre-defined entries corresponding to the LLVM/Clang releases:

```
$ llvmenv entries
llvm-mirror
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

### Local entries
Different from above "remote" entries, you can build locally cloned LLVM source with "local" entry.

```toml
[my-local-llvm]
path = "/path/to/your/src"
target = ["X86"]
```

Entry is regarded as "local" if there is `path` property, and "remote" if there is `url` property.
Other options are common to "remote" entries.

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
