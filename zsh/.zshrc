# Enable Powerlevel10k instant prompt. Should stay close to the top of ~/.zshrc.
# Initialization code that may require console input (password prompts, [y/n]
# confirmations, etc.) must go above this block; everything else may go below.
if [[ -r "${XDG_CACHE_HOME:-$HOME/.cache}/p10k-instant-prompt-${(%):-%n}.zsh" ]]; then
  source "${XDG_CACHE_HOME:-$HOME/.cache}/p10k-instant-prompt-${(%):-%n}.zsh"
fi

# Environment variables
source ~/.zsh/env.zsh

# Homebrew
source ~/.zsh/brew.zsh

# Powerlevel10k theme
source ~/.zsh/p10k.zsh

eval $(thefuck --alias)

# zsh-vi-mode
source ~/.zsh/zsh-vi-mode.zsh

# Oh-My-Zsh. ANYTHING THAT ADDS COMPLETIONS MUST BE SOURCED BEFORE THIS
source ~/.zsh/oh-my-zsh.zsh

# zsh-autosuggestions
source $(brew --prefix)/share/zsh-autosuggestions/zsh-autosuggestions.zsh

eval "$(zoxide init zsh)"

# fzf-tab
source ~/.zsh/fzf-tab.zsh

# fzf
source ~/.zsh/fzf.zsh

# fast-syntax-highlighting - Must be sourced last
source ~/.zsh/fast-syntax-highlighting.zsh

# Aliases
source ~/.zsh/aliases.zsh

export XDG_RUNTIME_DIR=/tmp

if [ -f "$HOME/.env" ]; then
    source "$HOME/.env"
fi
