#!/bin/sh
set -e

# get the branch name
BRANCH=${1:-"main"}
PRERELEASE=${2:-false}
BACKEND_URL=${3:-""}

# install the debian package
echo "Installing the Miru Agent"
echo "========================="
curl -fsSL https://raw.githubusercontent.com/miruml/agent/"$BRANCH"/scripts/install/manual-deb-install.sh | sh -s -- "$PRERELEASE"

# install the agent
echo ""
echo ""
echo "Activating the Miru Agent"
echo "========================="
if [ -n "$BACKEND_URL" ]; then
    curl -fsSL https://raw.githubusercontent.com/miruml/agent/"$BRANCH"/scripts/install/activate.sh | sh -s -- "$PRERELEASE" "$BACKEND_URL"
else
    curl -fsSL https://raw.githubusercontent.com/miruml/agent/"$BRANCH"/scripts/install/activate.sh | sh -s -- "$PRERELEASE"
fi

exit 0
