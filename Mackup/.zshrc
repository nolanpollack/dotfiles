# Enable Powerlevel10k instant prompt. Should stay close to the top of ~/.zshrc.
# Initialization code that may require console input (password prompts, [y/n]
# confirmations, etc.) must go above this block; everything else may go below.
if [[ -r "${XDG_CACHE_HOME:-$HOME/.cache}/p10k-instant-prompt-${(%):-%n}.zsh" ]]; then
  source "${XDG_CACHE_HOME:-$HOME/.cache}/p10k-instant-prompt-${(%):-%n}.zsh"
fi

ZSH_DIRECTORY=~/dotfiles/Mackup/.zsh

# Environment variables
source "$ZSH_DIRECTORY/env.zsh"

# Oh-My-Zsh and Powerlevel10k
source "$ZSH_DIRECTORY/oh-my-zsh.zsh"

# Homebrew
source "$ZSH_DIRECTORY/brew.zsh"

# zsh-autosuggestions
source $(brew --prefix)/share/zsh-autosuggestions/zsh-autosuggestions.zsh

# fzf 
source "$ZSH_DIRECTORY/fzf.zsh"

# zsh-vi-mode
source "$ZSH_DIRECTORY/zsh-vi-mode.zsh"

# zsh-syntax-highlighting - Must be sourced last
source "$ZSH_DIRECTORY/zsh-syntax-highlighting.zsh"

# Aliases
source "$ZSH_DIRECTORY/aliases.zsh"
