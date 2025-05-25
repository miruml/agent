#!/bin/sh
set -e

this_repo_root_dir=$(git rev-parse --show-toplevel)
# shellcheck source=../display.sh
. "$this_repo_root_dir/scripts/display.sh"

default_value() {
    default_value=${1:-}
    for arg in "$@"; do
        case $arg in
        --default=*) default_value="${arg#*=}";;
        esac
    done
    echo "$default_value"
}

# Debug flag
debug_flag() {
    debug_flag=$(default_value false "$@")
    for arg in "$@"; do
        case $arg in
        --debug) debug_flag=true;;
        --debug=*) debug_flag="${arg#*=}";;
        esac
    done
    echo "$debug_flag"
}

# Git branch
git_branch() {
    git_branch=$(default_value "main" "$@")
    for arg in "$@"; do
        case $arg in
        --git-branch=*) git_branch="${arg#*=}";;
        esac
    done
    echo "$git_branch"
}

print_git_branch() {
    git_branch=$1
    debug "Git Branch: '$git_branch'"
}

# Prerelease flag
prerelease_flag() {
    prerelease_flag=$(default_value false "$@")
    for arg in "$@"; do
        case $arg in
            --prerelease) prerelease_flag=true;;
            --prerelease=*) prerelease_flag="${arg#*=}";;
        esac
    done
    echo "$prerelease_flag"
}

print_prerelease_flag() {
    prerelease_flag=$1
    debug "Prerelease flag: '$prerelease_flag' (should be true or false)"
}

# Backend URL
backend_base_url() {
    backend_base_url=$(default_value "" "$@")
    for arg in "$@"; do
        case $arg in
        --backend-base-url=*) backend_base_url="${arg#*=}";;
        esac
    done
    echo "$backend_base_url"
}

print_backend_base_url() {
    backend_base_url=$1
    debug "Backend Base URL: '$backend_base_url'"
}