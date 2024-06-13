# Enable Powerlevel10k instant prompt. Should stay close to the top of ~/.zshrc.
# Initialization code that may require console input (password prompts, [y/n]
# confirmations, etc.) must go above this block; everything else may go below.
if [[ -r "${XDG_CACHE_HOME:-$HOME/.cache}/p10k-instant-prompt-${(%):-%n}.zsh" ]]; then
  source "${XDG_CACHE_HOME:-$HOME/.cache}/p10k-instant-prompt-${(%):-%n}.zsh"
fi


# Environment variables
# Path to your oh-my-zsh installation.
export ZSH="$HOME/.oh-my-zsh"
export MANPAGER='nvim +Man!'
export EDITOR=nvim
export COLORTERM='truecolor' 
export SUDO_EDITOR=/home/linuxbrew/.linuxbrew/bin/nvim


# Plugins
plugins=(git)


# Powerlevel10k
ZSH_THEME="powerlevel10k/powerlevel10k"
source $ZSH/oh-my-zsh.sh
# To customize prompt, run `p10k configure` or edit ~/.p10k.zsh.
[[ ! -f ~/.p10k.zsh ]] || source ~/.p10k.zsh


# Homebrew
eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"
# save installed brew packages to requirements
brew_install_and_update_requirements() {
    if command brew install "$@"; then
        brew leaves --installed-on-request > ~/dotfiles/brew_requirements.txt
    fi
}
brew_uninstall_and_update_requirements() {
    if command brew uninstall "$@"; then
        brew leaves --installed-on-request > ~/dotfiles/brew_requirements.txt
    fi
}
alias brew='brew_with_logging'
brew_with_logging() {
    case "$1" in
        install)
            shift
            brew_install_and_update_requirements "$@"
            ;;
        uninstall)
            shift
            brew_uninstall_and_update_requirements "$@"
            ;;
        *)
            command brew "$@"
            ;;
    esac
}


# zsh-autosuggestions
source $(brew --prefix)/share/zsh-autosuggestions/zsh-autosuggestions.zsh


# fzf - catppuccin color scheme
export FZF_DEFAULT_OPTS="--color=bg+:#313244,bg:#1e1e2e,spinner:#f5e0dc,hl:#f38ba8 --color=fg:#cdd6f4,header:#f38ba8,info:#cba6f7,pointer:#f5e0dc --color=marker:#f5e0dc,fg+:#cdd6f4,prompt:#cba6f7,hl+:#f38ba8"
source <(fzf --zsh)


# zsh-syntax-highlighting - Must be sourced last
source $(brew --prefix)/share/zsh-syntax-highlighting/zsh-syntax-highlighting.zsh


# Aliases
alias vim=nvim
alias prettier="npx prettier"
