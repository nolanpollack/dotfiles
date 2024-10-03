ZVM_VI_INSERT_ESCAPE_BINDKEY=jk
ZVM_VI_HIGHLIGHT_BACKGROUND=#313244
ZVM_VI_HIGHLIGHT_FOREGROUND=white

# Allow fzf to override the default key bindings
zvm_after_init_commands+=('source <(fzf --zsh)')

source /home/linuxbrew/.linuxbrew/opt/zsh-vi-mode/share/zsh-vi-mode/zsh-vi-mode.plugin.zsh

zvm_bindkey vicmd 'H' beginning-of-line
zvm_bindkey vicmd 'L' end-of-line
