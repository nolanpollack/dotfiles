alias vim=nvim
alias prettier="npx prettier"

# eza
alias ls="eza --group-directories-first --icons=auto"
alias l="eza -al --icons=auto --group-directories-first"
alias lt="eza -T --icons=auto"

# zoxide
alias cd="z"

# bat
alias batlog="bat --paging=never -l log"

# Docker compose
alias dc="docker compose"

# Git
alias gdb="git diff master...head"
alias gpv="gh pr view --web"

if [[ $OSTYPE == darwin* ]]; then
  alias copy="pbcopy"
else
  alias copy="xclip -sel clip"
fi

fr() {
    if [ "$#" -ne 2 ]; then
        echo "Usage: fr <search_pattern> <replacement>"
        return 1
    fi

    search_pattern=$1
    replacement=$2

    # TODO: Syntax highlighting
    selected_files=$(rg -l "$search_pattern" | 
        fzf --multi --preview "delta --width \$FZF_PREVIEW_COLUMNS {} <(sd -p '$search_pattern' '$replacement' {})" \
    --preview-window 'down:60%')
    if [ -z "$selected_files" ]; then
        echo "No files selected."
        return 1
    fi

    echo $selected_files | xargs sd "$search_pattern" "$replacement"
}

if [[ -v $WSL_DISTR_NAME ]]; then
  alias open="explorer.exe"
fi

alias cat=bat

alias brew-diff="delta ~/dotfiles/brew/requirements.txt <(brew leaves)"
