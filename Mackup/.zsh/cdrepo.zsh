cdrepo() {
    cd_stdout=$(cd "$1" 2>&1)
    exit_status=$?
    if [[ $exit_status -ne 0 ]]; then
        root=$(_find_repo_root)
        if [ $? = 0 ]; then
            path_to_repo=$(repo list | grep "$1")
            if [ $? = 0 ]; then
                path_to_repo="$root/$(echo $path_to_repo | head -n1 | cut -d':' -f1 | xargs)"
                echo $path_to_repo
                cd "$path_to_repo"
                return $?
            fi
        fi
    fi
    echo "$cd_stdout"
    return $exit_status
}

_find_repo_root() {
    local current_dir="$PWD"
    while [[ "$current_dir" != "/" ]]; do
        if [[ -d "$current_dir/.repo" ]]; then
            echo "$current_dir"
            return 0
        fi
        current_dir="$(dirname "$current_dir")"
    done
    return 1
}

alias cdr="cdrepo"
