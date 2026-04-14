# compdef isn't defined until we call compinit. But we can't call compinit until
# all plugins have added their completions to fpath. So to allow plugins to call
# compdef to register completions, while also allowing plugins to add to fpath to
# register completions, we defer the compdef calls until we actually call compinit.
typeset -ga _deferred_compdefs=()

compdef() {
  _deferred_compdefs+=("${(j: :)@}")
}

_run_compinit() {
  typeset -gU fpath
  fpath=( ${^fpath}(N/) )

  autoload -Uz compinit && compinit -C

  local def
  for def in "${_deferred_compdefs[@]}"; do
    builtin eval "compdef $def"
  done
  unset _deferred_compdefs
}
