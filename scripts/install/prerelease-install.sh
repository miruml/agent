#!/bin/sh
set -e

this_repo_root_dir=$(git rev-parse --show-toplevel)
# shellcheck source=../display.sh
. "$this_repo_root_dir/scripts/display.sh"
# shellcheck source=./args.sh
. "$this_repo_root_dir/scripts/install/args.sh"

# CLI args
DEBUG=$(debug_flag --default=false "$@")
BRANCH=$(git_branch --default=main "$@")
if [ "$DEBUG" = true ]; then
    print_git_branch "$BRANCH"
fi
PRERELEASE=$(prerelease_flag --default=false "$@")
if [ "$DEBUG" = true ]; then
    print_prerelease_flag "$PRERELEASE"
fi
BACKEND_URL=$(backend_url --default= "" "$@")
if [ "$DEBUG" = true ]; then
    print_backend_url "$BACKEND_URL"
fi

PRERELEASE=true
curl -fsSL https://raw.githubusercontent.com/miruml/agent/"$BRANCH"/scripts/install/install.sh | sh -s -- --branch="$BRANCH" --prerelease="$PRERELEASE" --backend-url="$BACKEND_URL"