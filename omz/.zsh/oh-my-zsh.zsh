# Path to your oh-my-zsh installation.
export ZSH="$HOME/.oh-my-zsh"
export ZSH_CUSTOM="$HOME/.config/omz"

# Plugins
plugins=(
    git
    docker
    tmux
    fzf-tab
)

# Allows docker commands to autocomplete with option-stacking
zstyle ':completion:*:*:docker:*' option-stacking yes
zstyle ':completion:*:*:docker-*:*' option-stacking yes

# Start tmux by default
ZSH_TMUX_AUTOSTART=true

source $ZSH/oh-my-zsh.sh
