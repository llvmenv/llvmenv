#!/usr/bin/zsh

function llvmenv_remove_path() {
  path_base=${XDG_DATA_HOME:-$HOME/.local/share/llvmenv}
  path=("${(@)path:#$path_base/*}")
}

function llvmenv_append_path() {
  prefix=$(llvmenv prefix)
  if [[ -n "$prefix" && "$prefix" != "/usr" ]]; then
    # To avoid /usr/bin and /bin become the top of $PATH
    path=($prefix/bin(N-/) $path)
  fi
}

function llvmenv_env_llvm_sys () {
  export LLVM_SYS_$(llvmenv version --major --minor)_PREFIX=$(llvmenv prefix)
}

function llvmenv_update () {
  llvmenv_remove_path
  llvmenv_append_path
  if [[ -n "$LLVMENV_RUST_BINDING" ]]; then
    llvmenv_env_llvm_sys
  fi
}

autoload -Uz add-zsh-hook
add-zsh-hook precmd llvmenv_update
