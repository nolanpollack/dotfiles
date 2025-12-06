export HOMEBREW_NO_ENV_HINTS=TRUE

eval $(brew shellenv)

# Required for brew completions to work
export FPATH="$HOMEBREW_PREFIX/share/zsh/site-functions:${FPATH:-}";
