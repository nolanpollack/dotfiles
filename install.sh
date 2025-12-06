#!/bin/bash

# Clone submodules
git submodule update --init --recursive

# TODO: I'm thinking mackup might be the wrong approach
ln -s Mackup/.mackup.cfg "$HOME"/.mackup.cfg

mackup restore

# Checks if homebrew is installed, and installs it if not.
if ! command -v brew >/dev/null 2>&1; then
    echo "Installing brew"
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    eval "$(/home/linuxbrew/.linuxbrew/bin/brew shellenv)"
fi

# install any required brew formulae which isn't installed
xargs brew install < ./brew_requirements.txt

chsh -s $(which zsh)

# Allow commands installed with brew to use completions
brew completions link

# Install tmux packages
bash ~/.tmux/plugins/tpm/scripts/install_plugins.sh

# Initialize rust
rustup default stable
rustup component add rust-analyzer

if [[ -v $WSL_DISTRO_NAME ]]; then
    sudo add-apt-repository ppa:wslutilities/wslu
    sudo apt update
    sudo apt install wslu

    # Set up wsl.conf so PATH doesn't include /mnt/c/Program Files/... etc
    # This stops lag in zsh syntax highlighting
    sudo ln -s "${HOME}/dotfiles/wsl.conf" /etc/wsl.conf
fi

# Rebuild bat's cache so it recognizes catpuccin theme
bat cache --build

# Set up catppuccin theme for zsh syntax highlighting
fast-theme ~/dotfiles/Mackup/.zsh/catppuccin-mocha.ini

# Undercurl
# https://dev.to/anurag_pramanik/how-to-enable-undercurl-in-neovim-terminal-and-tmux-setup-guide-2ld7
if [ -z $(infocmp -l -x | rg Smulx) ]; then
    infocmp > /tmp/${TERM}.ti
    sd -p '(smul=\\E\[4m,)' '${1} Smulx=\E[4:%p1%dm,' /tmp/${TERM}.ti
    tic -x /tmp/${TERM}.ti
fi

# Update tldr cache
tldr --update
