#!/bin/bash

ln -s Mackup/.mackup.cfg "$HOME"/.mackup.cfg

# Checks if homebrew is installed, and installs it if not. 
if ! command -v brew >/dev/null 2>&1; then
    echo "Installing brew"
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
fi

# install any required brew formulae which isn't installed
xargs brew install < ./brew_requirements.txt 

# Install tmux package manager
git clone https://github.com/tmux-plugins/tpm ~/.tmux/plugins/tpm

# Install tmux packages
bash ~/.tmux/plugins/tpm/scripts/install_plugins.sh

curl --proto '=https' --tlsv1.2 https://sh.rustup.rs -sSf | sh

sudo add-apt-repository ppa:wslutilities/wslu
sudo apt update
sudo apt install wslu
