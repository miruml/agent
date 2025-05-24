#!/bin/sh
set -e

# get the branch name
branch=${1:-"main"}

# install the debian package
echo "Installing the Miru Agent"
echo "========================="
curl -fsSL https://raw.githubusercontent.com/miruml/agent/"$branch"/scripts/install/manual-deb-install.sh | sh

# install the agent
echo ""
echo ""
echo "Activating the Miru Agent"
echo "========================="
BACKEND_URL=${1:-""}
if [ -n "$BACKEND_URL" ]; then
    curl -fsSL https://raw.githubusercontent.com/miruml/agent/"$branch"/scripts/install/activate.sh | sh -s -- "$BACKEND_URL"
else
    curl -fsSL https://raw.githubusercontent.com/miruml/agent/"$branch"/scripts/install/activate.sh | sh
fi

exit 0
