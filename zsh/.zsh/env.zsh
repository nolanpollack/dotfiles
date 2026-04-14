# Environment variables
typeset -U path

export MANPAGER='nvim +Man!'
export EDITOR=nvim
export COLORTERM='truecolor' 
export SUDO_EDITOR="$HOMEBREW_PREFIX/bin/nvim"

# Path
path+=("$HOME/.local/bin")
path+=("$HOME/.ebcli-virtual-env/executables")
path+=("$HOMEBREW_PREFIX/opt/rustup/bin")
path+=("$HOME/.cargo/bin")

# Enable tool search in claude to avoid loading all mcps
ENABLE_TOOL_SEARCH=true
