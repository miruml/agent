#!/bin/sh
set -e

BRANCH=${1:-"dev"}
PRERELEASE=true
BACKEND_URL=https://configs.dev.api.miruml.com/agent/v1
curl -fsSL https://raw.githubusercontent.com/miruml/agent/"$BRANCH"/scripts/install/manual-install.sh | sh -s -- "$BRANCH" "$PRERELEASE" "$BACKEND_URL"