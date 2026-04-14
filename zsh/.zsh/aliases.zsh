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
git_main_branch () {
	command git rev-parse --git-dir &> /dev/null || return
	local remote ref
	for ref in refs/{heads,remotes/{origin,upstream}}/{main,trunk,mainline,default,stable,master}
	do
		if command git show-ref -q --verify $ref
		then
			echo ${ref:t}
			return 0
		fi
	done
	for remote in origin upstream
	do
		ref=$(command git rev-parse --abbrev-ref $remote/HEAD 2>/dev/null)
		if [[ $ref == $remote/* ]]
		then
			echo ${ref#"$remote/"}
			return 0
		fi
	done
	echo master
	return 1
}

alias gaa="git add --all"

alias gbc="git branch --show-current"

alias gc="git commit"
alias gcn!="git commit --amend --no-edit"

alias gd="git diff"
alias gdm='git diff $(git_main_branch)...HEAD'
alias gds="git diff --staged"

alias glog="git log --oneline --decorate"
alias gloga='git log --oneline --decorate --graph --all'
alias glogm='git log --oneline --decorate --graph $(git_main_branch) origin/$(git_main_branch) HEAD'

alias gp="git push"

alias gsw="git switch"
alias gswc="git switch -c"
alias gswm='git switch $(git_main_branch)'

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

if [[ -v WSL_DISTRO_NAME ]]; then
  alias open="/mnt/c/Windows/explorer.exe"
fi

alias cat=bat

alias brew-diff="delta ~/dotfiles/brew/requirements.txt <(brew leaves)"

alias python="python3"
