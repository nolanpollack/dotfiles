#!/usr/bin/env bash

# Change to the script's directory
cd "$(dirname "$0")"

# Clone submodules
git submodule update --init --recursive

# Checks if homebrew is installed, and installs it if not.
if ! command -v brew >/dev/null 2>&1; then
    echo "Installing brew"
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    eval "$(brew shellenv)"
fi

# install any required brew formulae which isn't installed
xargs brew install < ./brew/requirements.txt

# Symlink dotfiles using stow script
./stow

if [[ $SHELL != $(which zsh) ]]; then
    if ! grep -qxF "$(which zsh)" /etc/shells; then
        echo "Adding $(which zsh) to /etc/shells (requires sudo)"
        echo "$(which zsh)" | sudo tee -a /etc/shells >/dev/null
        echo "✓ Added $(which zsh) to /etc/shells"
    else
        echo "✓ $ZSH_PATH already in /etc/shells"
    fi
    echo "Changing default shell to zsh"
    chsh -s $(which zsh)
fi

# Allow commands installed with brew to use completions
brew completions link

# Install tmux packages
bash ~/.tmux/plugins/tpm/scripts/install_plugins.sh

# Initialize rust
rustup default stable
rustup component add rust-analyzer

if [[ -v WSL_DISTRO_NAME ]]; then
    sudo add-apt-repository ppa:wslutilities/wslu
    sudo apt update
    sudo apt install wslu
fi

# Rebuild bat's cache so it recognizes catpuccin theme
bat cache --build

# Set up catppuccin theme for zsh syntax highlighting
source ~/.zsh/fast-syntax-highlighting.zsh
fast-theme ~/.zsh/catppuccin-mocha.ini

# Undercurl
# https://dev.to/anurag_pramanik/how-to-enable-undercurl-in-neovim-terminal-and-tmux-setup-guide-2ld7
if [ -z $(infocmp -l -x | rg Smulx) ]; then
    infocmp > /tmp/${TERM}.ti
    sd -p '(smul=\\E\[4m,)' '${1} Smulx=\E[4:%p1%dm,' /tmp/${TERM}.ti
    tic -x /tmp/${TERM}.ti
fi

# Update tldr cache
tldr --update
