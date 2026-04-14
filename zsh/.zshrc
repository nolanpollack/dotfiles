source ~/.zsh/zsh-defer/zsh-defer.plugin.zsh

if [[ -v WSL_DISTRO_NAME ]]; then
  eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"
fi

# Enable Powerlevel10k instant prompt. Should stay close to the top of ~/.zshrc.
# Initialization code that may require console input (password prompts, [y/n]
# confirmations, etc.) must go above this block; everything else may go below.
if [[ -r "${XDG_CACHE_HOME:-$HOME/.cache}/p10k-instant-prompt-${(%):-%n}.zsh" ]]; then
  source "${XDG_CACHE_HOME:-$HOME/.cache}/p10k-instant-prompt-${(%):-%n}.zsh"
fi

# Add completion functions and setup deferral of compdef calls
source ~/.zsh/completions.zsh

# Aliases
source ~/.zsh/aliases.zsh

# Environment variables
source ~/.zsh/env.zsh

# Homebrew
source ~/.zsh/brew.zsh

# Powerlevel10k theme
source ~/.zsh/p10k.zsh

# zsh-vi-mode
source ~/.zsh/zsh-vi-mode.zsh

# zsh-autosuggestions
source "$HOMEBREW_PREFIX/share/zsh-autosuggestions/zsh-autosuggestions.zsh"

# Zoxide
eval "$(zoxide init zsh)"

# fzf
source ~/.zsh/fzf.zsh

# Load completions. Should be called _after_ all plugins that add completions
_run_compinit

# fzf-tab (must be after compinit to override _complete function)
source ~/.zsh/fzf-tab.zsh

# fast-syntax-highlighting - Must be sourced last
source ~/.zsh/fast-syntax-highlighting.zsh

# Deferred init (slow, not needed immediately)
zsh-defer -c 'eval "$(thefuck --alias)"'

if [ -f "$HOME/.env" ]; then
    source "$HOME/.env"
fi

# Tmux autostart
if command -v tmux &>/dev/null && [[ -z "$TMUX" ]]; then
  tmux attach -t default 2>/dev/null || tmux new-session -s default
fi
