autoload -Uz add-zsh-hook

_title_precmd()  { printf '\e]2;%s\a' "${PWD/#$HOME/~}" }
_title_preexec() { printf '\e]2;%s\a' "$1" }

add-zsh-hook precmd  _title_precmd
add-zsh-hook preexec _title_preexec
