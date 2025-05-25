#!/bin/sh
set -e

# CLI args
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
BACKEND_BASE_URL=$(backend_base_url --default="" "$@")
if [ "$DEBUG" = true ]; then
    print_backend_base_url "$BACKEND_BASE_URL"
fi

# install the debian package
echo "Installing the Miru Agent"
echo "========================="
curl -fsSL https://raw.githubusercontent.com/miruml/agent/"$BRANCH"/scripts/install/deb-install.sh | sh -s -- --prerelease="$PRERELEASE"

# install the agent
echo ""
echo ""
echo "Activating the Miru Agent"
echo "========================="
if [ -n "$BACKEND_BASE_URL" ]; then
    curl -fsSL https://raw.githubusercontent.com/miruml/agent/"$BRANCH"/scripts/install/activate.sh | sh -s -- --prerelease="$PRERELEASE" --backend-base-url="$BACKEND_BASE_URL"
else
    curl -fsSL https://raw.githubusercontent.com/miruml/agent/"$BRANCH"/scripts/install/activate.sh | sh -s -- --prerelease="$PRERELEASE"
fi

exit 0
