llvmenv
=========

Manage multiple LLVM/Clang build

[![CircleCI](https://circleci.com/gh/termoshtt/llvmenv.svg?style=shield)](https://circleci.com/gh/termoshtt/llvmenv)

```
llvmenv 0.1.1
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

Usage (zsh integration)
-----

```
eval $(llvmenv zsh)
```

in your `.zshrc`. Other shell support is WIP.
