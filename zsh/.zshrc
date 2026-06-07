source ~/.zsh/zsh-defer/zsh-defer.plugin.zsh

if [[ -v WSL_DISTRO_NAME ]]; then
  eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"
fi

# Add completion functions and setup deferral of compdef calls
source ~/.zsh/completions.zsh

# Aliases
source ~/.zsh/aliases.zsh

# Environment variables
source ~/.zsh/env.zsh

# History
source ~/.zsh/history.zsh

# Homebrew
source ~/.zsh/brew.zsh

# Bun
source ~/.zsh/bun.zsh

# PostgreSQL
source ~/.zsh/postgresql.zsh

# Starship
eval "$(starship init zsh)"

# Terminal/pane title
source ~/.zsh/title.zsh

# zsh-vi-mode
source ~/.zsh/zsh-vi-mode.zsh

# zsh-autosuggestions
source "$HOMEBREW_PREFIX/share/zsh-autosuggestions/zsh-autosuggestions.zsh"

# Zoxide
eval "$(zoxide init zsh)"

# Load completions. Should be called _after_ all plugins that add completions
_run_compinit

# fzf-tab (must be after compinit to override _complete function)
source ~/.zsh/fzf-tab.zsh

# fzf (must be after fzf-tab)
source ~/.zsh/fzf.zsh

# fast-syntax-highlighting - Must be sourced last
source ~/.zsh/fast-syntax-highlighting.zsh

# Deferred init (slow, not needed immediately)
zsh-defer -c 'eval "$(thefuck --alias)"'

if [ -f "$HOME/.env" ]; then
    source "$HOME/.env"
fi

eval "$(zellij setup --generate-auto-start zsh)"
