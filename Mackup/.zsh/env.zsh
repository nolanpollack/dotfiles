# Environment variables
# Path to your oh-my-zsh installation.
export ZSH="$HOME/.oh-my-zsh"
export MANPAGER='nvim +Man!'
export EDITOR=nvim
export COLORTERM='truecolor' 
export SUDO_EDITOR="$HOMEBREW_PREFIX/bin/nvim"
path+=("$HOME/.local/bin")
path+=("$HOME/.ebcli-virtual-env/executables")

source "$HOME/.secrets.zsh"
