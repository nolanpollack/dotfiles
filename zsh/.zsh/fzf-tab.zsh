source ~/.zsh/fzf-tab/fzf-tab.plugin.zsh

# Disable alphabetic sort when completing git checkout
zstyle ':completion:*:git-checkout:*' sort false

# Enable group support
zstyle ':completion:*:descriptions' format '%d'

# Preview dirs with eza when completing cd
zstyle ':fzf-tab:complete:cd:*' fzf-preview 'eza -1 -a --color=always --icons=always $realpath'

# Preview diffs with delta when completing git
zstyle ':fzf-tab:complete:git-(add|diff|restore):*' fzf-preview 'git diff $realpath | delta'

# Switch group with < and >
zstyle ':fzf-tab:*' switch-group '<' '>'
