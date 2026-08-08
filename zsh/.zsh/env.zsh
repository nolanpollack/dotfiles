# Environment variables
typeset -U path

export MANPAGER='nvim +Man!'
export EDITOR=nvim
export COLORTERM='truecolor' 
export SUDO_EDITOR="$HOMEBREW_PREFIX/bin/nvim"
export XDG_RUNTIME_DIR=/tmp

# macOS's default $TMPDIR is a long per-process random path, which pushes
# zellij's IPC socket path (built from it) past the 103-byte AF_UNIX limit
# once the session name is long enough. Force a short, fixed socket dir.
export ZELLIJ_SOCKET_DIR=/tmp/zellij

# Path
path+=("$HOME/.local/bin")
path+=("$HOME/.ebcli-virtual-env/executables")
path+=("$HOMEBREW_PREFIX/opt/rustup/bin")
path+=("$HOME/.cargo/bin")

# Enable tool search in claude to avoid loading all mcps
ENABLE_TOOL_SEARCH=true
