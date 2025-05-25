#!/bin/sh
set -e

this_repo_root_dir=$(git rev-parse --show-toplevel)
# shellcheck source=../display.sh
. "$this_repo_root_dir/scripts/display.sh"
# shellcheck source=./args.sh
. "$this_repo_root_dir/scripts/install/args.sh"

# CLI args
DEBUG=$(debug_flag --default=true "$@")
BRANCH=$(git_branch --default=dev "$@")
if [ "$DEBUG" = true ]; then
    print_git_branch "$BRANCH"
fi
BACKEND_BASE_URL=$(backend_base_url --default="https://configs.dev.api.miruml.com/agent/v1" "$@")
if [ "$DEBUG" = true ]; then
    print_backend_base_url "$BACKEND_BASE_URL"
fi

curl -fsSL https://raw.githubusercontent.com/miruml/agent/"$BRANCH"/scripts/install/prerelease-install.sh | sh -s -- --git-branch="$BRANCH" --backend-base-url="$BACKEND_BASE_URL"