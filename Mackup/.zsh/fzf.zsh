export FZF_DEFAULT_OPTS="--color=bg+:#313244,bg:#1e1e2e,spinner:#f5e0dc,hl:#f38ba8 --color=fg:#cdd6f4,header:#f38ba8,info:#cba6f7,pointer:#f5e0dc --color=marker:#f5e0dc,fg+:#cdd6f4,prompt:#cba6f7,hl+:#f38ba8"
source <(fzf --zsh)

export FZF_DEFAULT_COMMAND="fd --type f --strip-cwd-prefix"
export FZF_COMPLETION_DIR_COMMANDS="cd pushd rmdir ls"

export FZF_CTRL_T_OPTS="--preview '[[ -d {} ]] && eza -1 -a --color=always --icons=always {} || bat --color=always {}'"


_fzf_compgen_path() {
    fd --hidden --follow --exclude ".git" . "$1" | sed 's@^\./@@'
}
_fzf_compgen_path() {
    fd --type d --hidden --follow --exclude ".git" . "$1" | sed 's@^\./@@'
}

fzf_ripgrep() {
    rg -i --no-ignore-dot --color=always --line-number --no-heading --smart-case "${*:-}" |
    fzf --ansi \
      --color "hl:-1:underline,hl+:-1:underline:reverse" \
      --delimiter : \
      --preview 'bat --color=always {1} --highlight-line {2}' \
      --preview-window 'up,60%,border-bottom,+{2}+3/3,~3' \
      --bind 'enter:become(vim {1} +{2})'
}
alias rf=fzf_ripgrep
