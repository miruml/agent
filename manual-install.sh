#!/bin/sh
set -e

# install the debian package
echo "Installing the Miru Agent"
echo "========================="
curl -fsSL https://raw.githubusercontent.com/miruml/agent/main/manual-deb-install.sh | sh

# install the agent
echo ""
echo ""
echo "Activating the Miru Agent"
echo "========================="
curl -fsSL https://raw.githubusercontent.com/miruml/agent/main/activate.sh | sh