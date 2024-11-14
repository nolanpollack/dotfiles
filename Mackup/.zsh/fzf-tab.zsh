zstyle ':fzf-tab:complete:z:*' fzf-preview 'eza -1 -a --color=always --icons=always $realpath'

# Disable sort when completing git checkout
zstyle ':completion:*:git-checkout:*' sort false

# Group completions
zstyle ':completion:*:descriptions' format '%d'

zstyle ':fzf-tab:complete:git-(add|diff|restore):*' fzf-preview 'git diff $realpath | delta'

# Switch group with < and >
zstyle ':fzf-tab:*' switch-group '<' '>'
