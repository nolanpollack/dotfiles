export HOMEBREW_NO_ENV_HINTS=TRUE

# brew_install_and_update_requirements() {
#     if command brew install "$@"; then
#         brew leaves --installed-on-request > ~/dotfiles/brew_requirements.txt
#     fi
# }
# brew_uninstall_and_update_requirements() {
#     if command brew uninstall "$@"; then
#         brew leaves --installed-on-request > ~/dotfiles/brew_requirements.txt
#     fi
# }
# alias brew='brew_with_logging'
# brew_with_logging() {
#     case "$1" in
#         install)
#             shift
#             brew_install_and_update_requirements "$@"
#             ;;
#         uninstall)
#             shift
#             brew_uninstall_and_update_requirements "$@"
#             ;;
#         *)
#             command brew "$@"
#             ;;
#     esac
# }
#
eval $(brew shellenv)

# Required for brew completions to work
export FPATH="$HOMEBREW_PREFIX/share/zsh/site-functions:${FPATH:-}";
