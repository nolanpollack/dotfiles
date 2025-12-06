ZVM_VI_INSERT_ESCAPE_BINDKEY=jk
ZVM_VI_HIGHLIGHT_BACKGROUND=#313244
ZVM_VI_HIGHLIGHT_FOREGROUND=white

# Use system clipboard
ZVM_SYSTEM_CLIPBOARD_ENABLED=true

if [[ -v WSL_DISTRO_NAME ]]; then
    ZVM_CLIPBOARD_COPY_CMD='clip.exe'
    ZVM_CLIPBOARD_PASTE_CMD='powershell.exe -NoProfile -Command Get-Clipboard'
fi

function zvm_after_lazy_keybindings() {
  # Rebind p to paste from system clipboard after cursor
  zvm_bindkey vicmd 'p' zvm_paste_clipboard_after
  
  # Rebind P to paste from system clipboard before cursor
  zvm_bindkey vicmd 'P' zvm_paste_clipboard_before
}

# Allow fzf to override the default key bindings
zvm_after_init_commands+=('source <(fzf --zsh)')

source $(brew --prefix)/opt/zsh-vi-mode/share/zsh-vi-mode/zsh-vi-mode.plugin.zsh

zvm_bindkey vicmd 'H' beginning-of-line
zvm_bindkey vicmd 'L' end-of-line
