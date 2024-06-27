ZVM_VI_INSERT_ESCAPE_BINDKEY=jk

# Allow fzf to override the default key bindings
zvm_after_init_commands+=('source <(fzf --zsh)')

source /home/linuxbrew/.linuxbrew/opt/zsh-vi-mode/share/zsh-vi-mode/zsh-vi-mode.plugin.zsh
