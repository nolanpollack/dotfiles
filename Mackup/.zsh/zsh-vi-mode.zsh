ZVM_VI_INSERT_ESCAPE_BINDKEY=jk
ZVM_VI_HIGHLIGHT_BACKGROUND=#313244
ZVM_VI_HIGHLIGHT_FOREGROUND=white

# Use system clipboard
function zvm_after_lazy_keybindings() {
    source "$HOME/dotfiles/Mackup/.zsh/plugins/zsh-system-clipboard/zsh-system-clipboard.zsh"

    functions -c zvm_yank zvm_yank_orig
    function zvm_yank() {
        zvm_yank_orig "$@"
        printf '%s\n' "$CUTBUFFER" | zsh-system-clipboard-set
    }

    functions -c zvm_replace_selection zvm_replace_selection_orig
    function zvm_replace_selection() {
        zvm_replace_selection_orig "$@"
        printf '%s\n' "$CUTBUFFER" | zsh-system-clipboard-set
    }
    
    functions -c zvm_vi_put_after zvm_vi_put_after_orig
    function zvm_vi_put_after() {
        CUTBUFFER=$(zsh-system-clipboard-get)
        zvm_vi_put_after_orig "$@"
    }
    functions -c zvm_vi_put_before zvm_vi_put_before_orig
    function zvm_vi_put_before() {
        CUTBUFFER=$(zsh-system-clipboard-get)
        zvm_vi_put_before_orig "$@"
    }
}

# Allow fzf to override the default key bindings
zvm_after_init_commands+=('source <(fzf --zsh)')

source $(brew --prefix)/opt/zsh-vi-mode/share/zsh-vi-mode/zsh-vi-mode.plugin.zsh

zvm_bindkey vicmd 'H' beginning-of-line
zvm_bindkey vicmd 'L' end-of-line
